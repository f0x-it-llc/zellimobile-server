//! Blocking render-loop thread and associated query-fulfillment helpers.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use tokio::sync::mpsc;

use tonic::Status;

use crate::ipc::{self, AttachSender};
use crate::proto::ServerFrame;
use crate::proto::{ControlEvent, server_frame};

use super::types::InFlightQuery;

// ─── render_loop ──────────────────────────────────────────────────────────────

/// Blocking render loop — runs on a dedicated std thread.
///
/// Returns the number of render frames forwarded (for logging on join).
///
/// ## FX-QUERY: Log capture (render-thread-owned reply-fulfillment)
///
/// This thread is the SOLE owner of `recv()`, so it is the only place a `Log`
/// (the reply to a `ListTabs`/`ListPanes` query) can be observed. It therefore
/// also owns the in-flight layout query end-to-end:
///
/// - It drains `query_rx` (non-blocking `try_recv`) once per loop iteration. A
///   newly arrived query REPLACES any held one (replace-on-new): the old
///   `reply` is dropped, cancelling its receiver so `get_layout` falls back.
/// - It retires a held query whose `reply.is_closed()` (i.e. `get_layout`'s
///   `RELAY_QUERY_TIMEOUT` fired and dropped the receiver) — so the timed-out
///   query's stray Logs are DROPPED, never misattributed to it.
/// - On a `Log` WITH a held query: the first fills `tabs`, the second fills
///   `panes`; once both are present it `reply.send(Ok((tabs, panes)))` and
///   clears the slot. On a `Log` with NO held query: it is dropped (it is a
///   stale reply from an already-retired query).
/// - Every `Render` is forwarded to `tx` regardless of query state.
///
/// **Invariants** (see also the module-level FX-QUERY notes):
/// 1. The inbound `select!` arm never blocks on the query — it only hands the
///    `InFlightQuery` over this channel and returns; ALL awaiting/fulfillment
///    happens here.
/// 2. A timed-out / failed / replaced query can NEVER cause a *later* query's
///    Logs to be misattributed: the slot is retired on close/replace before any
///    Log is matched, and a Log with no held query is dropped.
/// 3. Render frames are never dropped (`Log` is a distinct variant from
///    `Render`).
pub(super) fn render_loop(
    receiver: &mut ipc::AttachReceiver,
    tx: mpsc::Sender<Result<ServerFrame, Status>>,
    stop: Arc<AtomicBool>,
    session: &str,
    query_rx: std::sync::mpsc::Receiver<InFlightQuery>,
) -> u64 {
    use zellij_utils::ipc::ServerToClientMsg;

    let mut renders: u64 = 0;
    // The single in-flight layout query, owned exclusively by this thread.
    // At most one is outstanding; a newer one replaces it (replace-on-new).
    let mut in_flight: Option<InFlightQuery> = None;

    loop {
        if stop.load(Ordering::Relaxed) {
            log::debug!("relay reader [{session}]: stop-flag set, exiting");
            break;
        }

        // Pre-recv: drain newly handed-off queries (replace-on-new) and retire
        // any held query whose receiver has gone away (grpc timed out).
        drain_queries(&query_rx, &mut in_flight, session);

        let Some(msg) = receiver.recv() else {
            log::info!("relay reader [{session}]: IPC stream closed (recv None)");
            drop(in_flight.take()); // dropping the reply cancels grpc's receiver
            break;
        };

        // Re-check after the (blocking) recv returned — shutdown may have been
        // requested while we were parked.
        if stop.load(Ordering::Relaxed) {
            log::debug!("relay reader [{session}]: stop-flag set after recv, exiting");
            drop(in_flight.take());
            break;
        }

        // Post-recv: a query may have been handed off while we were parked on
        // recv(); pick it up before matching this message so a Log that arrives
        // in the same wakeup is attributed to it.
        drain_queries(&query_rx, &mut in_flight, session);

        match msg {
            ServerToClientMsg::Render { content } => {
                let frame = ServerFrame {
                    kind: Some(server_frame::Kind::Render(content.into_bytes())),
                };
                // blocking_send applies backpressure; Err == receiver dropped
                // (gRPC client disconnected) → tear down.
                if tx.blocking_send(Ok(frame)).is_err() {
                    log::info!("relay reader [{session}]: outbound dropped (client gone), exiting");
                    drop(in_flight.take());
                    break;
                }
                renders += 1;
            }

            // FX-QUERY: Log is the reply to a ListTabs/ListPanes query action.
            // First Log → tabs, second Log → panes; on the second, fulfill the
            // reply and clear the slot. A Log with no in-flight query is a stale
            // reply from an already-retired query — drop it.
            ServerToClientMsg::Log { lines } => {
                capture_query_log(&mut in_flight, lines, session);
            }

            // ── Phase C: control events ──────────────────────────────────────
            //
            // Map pushed ServerToClientMsg variants → ServerFrame::control so
            // the gRPC client can react to session lifecycle changes without
            // polling.  Render bytes and control events interleave naturally on
            // the same bounded mpsc channel.
            ServerToClientMsg::Exit { exit_reason } => {
                log::info!("relay reader [{session}]: session Exit: {exit_reason:?}");
                // Drop any in-flight query (its receiver cancels → grpc falls
                // back / errors out rather than hanging to the outer timeout).
                drop(in_flight.take());
                // Surface the exit reason to the client before closing.
                let payload = exit_reason.to_string();
                let frame = ServerFrame {
                    kind: Some(server_frame::Kind::Control(ControlEvent {
                        kind: "exit".to_owned(),
                        payload,
                    })),
                };
                // Best-effort send; if the channel is already gone we just exit.
                let _ = tx.blocking_send(Ok(frame));
                break;
            }

            ServerToClientMsg::RenamedSession { name } => {
                log::info!("relay reader [{session}]: RenamedSession → '{name}'");
                let frame = ServerFrame {
                    kind: Some(server_frame::Kind::Control(ControlEvent {
                        kind: "renamed_session".to_owned(),
                        payload: name,
                    })),
                };
                if tx.blocking_send(Ok(frame)).is_err() {
                    log::info!("relay reader [{session}]: outbound dropped (client gone), exiting");
                    break;
                }
            }

            ServerToClientMsg::ConfigFileUpdated => {
                log::info!("relay reader [{session}]: ConfigFileUpdated");
                let frame = ServerFrame {
                    kind: Some(server_frame::Kind::Control(ControlEvent {
                        kind: "config_updated".to_owned(),
                        payload: String::new(),
                    })),
                };
                if tx.blocking_send(Ok(frame)).is_err() {
                    log::info!("relay reader [{session}]: outbound dropped (client gone), exiting");
                    break;
                }
            }

            ServerToClientMsg::SwitchSession { connect_to_session } => {
                let target = connect_to_session.name.unwrap_or_default();
                log::info!("relay reader [{session}]: SwitchSession → '{target}'");
                let frame = ServerFrame {
                    kind: Some(server_frame::Kind::Control(ControlEvent {
                        kind: "switch_session".to_owned(),
                        payload: target,
                    })),
                };
                if tx.blocking_send(Ok(frame)).is_err() {
                    log::info!("relay reader [{session}]: outbound dropped (client gone), exiting");
                    break;
                }
            }

            // Remaining variants we intentionally drop:
            //
            // • UnblockInputThread / Connected / QueryTerminalSize — internal
            //   plumbing; no semantic value to remote clients.
            // • LogError — only delivered to cli-client query connections; should
            //   not arrive here but we drain it if it does.
            // • PaneRenderUpdate / SubscribedPaneClosed — require an explicit
            //   SubscribeToPaneRenders subscription which we never send; these
            //   messages will not arrive in practice.
            // • UnblockCliPipeInput / CliPipeOutput — pipe plumbing.
            // • StartWebServer — internal.
            // • ForwardQueryToHost — terminal-query passthrough; not applicable.
            //
            // Known 0.44.3 limitation: no per-change push for tab/pane
            // structure changes or bell notifications.  Clients that need
            // up-to-date layout/bell state should poll GetLayout.
            other => {
                log::trace!("relay reader [{session}]: ignoring {other:?}");
            }
        }
    }
    log::info!("relay reader [{session}]: exited after {renders} renders");
    renders
}

// ─── render_loop query helpers (FX-QUERY) ──────────────────────────────────────

/// Drain newly handed-off layout queries and retire dead ones.
///
/// Called once before and once after each blocking `recv()` in [`render_loop`].
/// Two anti-cross-talk responsibilities, in order:
///
/// 1. **Retire-on-close.** If the held query's `reply.is_closed()` (grpc's
///    `RELAY_QUERY_TIMEOUT` fired and dropped the receiver), drop the slot so a
///    later Log can't be misattributed to the timed-out query. This is the
///    crucial idle-session fix: with no Renders flowing, the only way a stale
///    query is cleared is here (and on replace, below).
/// 2. **Replace-on-new.** Drain `query_rx` non-blocking; the LAST entry wins.
///    Replacing drops the previous `reply`, cancelling its receiver so grpc
///    falls back. (In practice get_layout polls serially, so at most one is
///    queued; the loop is defensive.)
fn drain_queries(
    query_rx: &std::sync::mpsc::Receiver<InFlightQuery>,
    in_flight: &mut Option<InFlightQuery>,
    session: &str,
) {
    // 1. Retire a held query whose consumer has gone away.
    if in_flight.as_ref().is_some_and(|q| q.reply.is_closed()) {
        let seq = in_flight.as_ref().map(|q| q.seq).unwrap_or(0);
        log::debug!(
            "relay reader [{session}]: in-flight query seq={seq} receiver closed \
             (grpc timed out) — retiring slot"
        );
        *in_flight = None;
    }

    // 2. Pick up any newly handed-off queries; a newer one replaces the older.
    while let Ok(q) = query_rx.try_recv() {
        if let Some(prev) = in_flight.take() {
            log::debug!(
                "relay reader [{session}]: query seq={} replaces in-flight seq={} \
                 (dropping old reply → grpc falls back)",
                q.seq,
                prev.seq
            );
            // `prev.reply` drops here → its receiver cancels.
        } else {
            log::trace!("relay reader [{session}]: query seq={} armed", q.seq);
        }
        *in_flight = Some(q);
    }
}

/// Apply a captured `Log` to the in-flight query (FX-QUERY).
///
/// First Log → tabs, second Log → panes. When both are present, fulfill the
/// `reply` with `(tabs_json, panes_json)` and clear the slot. A Log with no
/// in-flight query is a stale reply from an already-retired query — drop it.
///
/// RESIDUAL RISK (Log-ordering assumption): Logs are NOT tagged with the query
/// they answer. In the common timeout case the stalled session emits NO further
/// Logs, so the timed-out query is retired (reply closed) and the next query
/// fills cleanly from its OWN Logs. The one narrow window that remains: a query
/// that timed out AFTER its tabs Log but whose panes Log arrives LATE — if a new
/// query is already armed by then, that straggler panes Log lands in the new
/// query's tabs slot. This is inherent to untagged Logs (see the module NOTE);
/// the alternative is issuing the two actions one-at-a-time, which would
/// reintroduce an await on the inbound loop and is therefore rejected here.
fn capture_query_log(in_flight: &mut Option<InFlightQuery>, lines: Vec<String>, session: &str) {
    let Some(q) = in_flight.as_mut() else {
        log::debug!(
            "relay reader [{session}]: Log ({} lines) with no in-flight query — \
             dropping (stale reply)",
            lines.len()
        );
        return;
    };

    // If grpc already dropped the receiver, don't bother filling — retire it so
    // the *next* Log (e.g. the panes reply chasing a timed-out tabs query) is
    // also dropped rather than landing in a fresh query's slot.
    if q.reply.is_closed() {
        log::debug!(
            "relay reader [{session}]: in-flight query seq={} receiver closed mid-capture \
             — discarding Log + retiring slot",
            q.seq
        );
        *in_flight = None;
        return;
    }

    let json = lines.join("\n");
    if q.tabs.is_none() {
        log::trace!(
            "relay reader [{session}]: captured tabs Log ({}B) for query seq={}",
            json.len(),
            q.seq
        );
        q.tabs = Some(json);
        return;
    }

    // Second Log → panes; both halves present, fulfill and clear.
    let seq = q.seq;
    log::trace!(
        "relay reader [{session}]: captured panes Log ({}B) for query seq={seq} — fulfilling",
        json.len()
    );
    let panes_json = json;
    let q = in_flight.take().expect("checked Some above");
    let tabs_json = q.tabs.unwrap_or_default();

    let result = if tabs_json.is_empty() {
        Err(anyhow::anyhow!(
            "empty tabs JSON from relay query (seq={seq})"
        ))
    } else if panes_json.is_empty() {
        Err(anyhow::anyhow!(
            "empty panes JSON from relay query (seq={seq})"
        ))
    } else {
        Ok((tabs_json, panes_json))
    };
    // Receiver may have been dropped between the is_closed() check and now; the
    // send just no-ops in that case.
    let _ = q.reply.send(result);
}

// ─── ShutdownGuard ────────────────────────────────────────────────────────────

/// Tears down the std reader thread on drop.
pub(super) struct ShutdownGuard {
    pub(super) stop: Arc<AtomicBool>,
    pub(super) nudge: AttachSender,
    pub(super) reader: Option<JoinHandle<u64>>,
    pub(super) rows: u16,
    pub(super) cols: u16,
    /// When set, this attach is read-only and must NOT emit the teardown resize
    /// nudge (review round-2 Major A): the nudge is a `TerminalResize`, which
    /// re-runs zellij's `min_client_terminal_size` and could disturb a writer's
    /// geometry. For RO we rely solely on the IPC-close path to wake the reader.
    pub(super) read_only: bool,
    pub(super) session: String,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        log::info!("relay [{}]: shutting down", self.session);
        // 1. Ask the reader to stop.
        self.stop.store(true, Ordering::Relaxed);
        // 2. Wake the reader so its parked blocking recv() returns and observes
        //    the stop-flag. Two paths, depending on read-only:
        //
        //    • RW (unchanged from before): send a no-op TerminalResize to the
        //      same dims — enough to provoke a redraw. RW already drives the
        //      session's geometry, so re-asserting its own size is a no-op.
        //
        //    • RO (round-2 Major A): a TerminalResize would make zellij
        //      recompute the shared min terminal size and could shrink a
        //      writer's geometry, so we must NOT resize here. Instead send
        //      `ClientExited`: the server removes this client and closes our
        //      connection, which wakes the parked recv() (returns None) — and
        //      *removing* a client can only raise/keep the min, never lower it,
        //      so a read-only detach can never perturb a writer's geometry.
        //
        //    Errors are ignored — the socket may already be gone.
        if self.read_only {
            let _ = self.nudge.send_client_exited();
        } else {
            let _ = self.nudge.send_resize(self.rows, self.cols);
        }
        // 3. Join — the relay future does not complete until the thread is gone.
        if let Some(handle) = self.reader.take() {
            match handle.join() {
                Ok(renders) => log::info!(
                    "relay [{}]: reader thread joined ({renders} renders total)",
                    self.session
                ),
                Err(_) => log::error!("relay [{}]: reader thread panicked", self.session),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Rx = tokio::sync::oneshot::Receiver<anyhow::Result<(String, String)>>;

    fn mk_query(seq: u64) -> (InFlightQuery, Rx) {
        let (reply, rx) = tokio::sync::oneshot::channel();
        (
            InFlightQuery {
                seq,
                reply,
                tabs: None,
            },
            rx,
        )
    }

    #[test]
    fn capture_fills_tabs_then_panes_then_fulfills() {
        let (q, mut rx) = mk_query(1);
        let mut in_flight = Some(q);

        capture_query_log(&mut in_flight, vec!["TABS".into()], "s");
        assert!(in_flight.is_some(), "after first Log the slot stays armed");

        capture_query_log(&mut in_flight, vec!["PANES".into()], "s");
        assert!(in_flight.is_none(), "after second Log the slot is cleared");

        let got = rx.try_recv().expect("reply sent").expect("ok result");
        assert_eq!(got, ("TABS".to_string(), "PANES".to_string()));
    }

    #[test]
    fn capture_with_no_in_flight_query_is_a_noop() {
        let mut in_flight: Option<InFlightQuery> = None;
        capture_query_log(&mut in_flight, vec!["stray".into()], "s");
        assert!(in_flight.is_none());
    }

    #[test]
    fn capture_empty_tabs_fulfills_with_error() {
        let (q, mut rx) = mk_query(2);
        let mut in_flight = Some(q);

        capture_query_log(&mut in_flight, vec![], "s"); // tabs Log → ""
        capture_query_log(&mut in_flight, vec!["PANES".into()], "s");

        let got = rx.try_recv().expect("reply sent");
        assert!(got.is_err(), "empty tabs JSON yields an error result");
    }

    #[test]
    fn capture_retires_slot_when_receiver_closed_mid_capture() {
        let (q, rx) = mk_query(3);
        drop(rx); // grpc timed out and dropped the receiver
        let mut in_flight = Some(q);

        capture_query_log(&mut in_flight, vec!["TABS".into()], "s");
        assert!(
            in_flight.is_none(),
            "closed receiver → slot retired, Log discarded"
        );
    }

    #[test]
    fn drain_retires_query_with_closed_receiver() {
        let (q, rx) = mk_query(4);
        drop(rx);
        let mut in_flight = Some(q);
        let (_tx, query_rx) = std::sync::mpsc::channel::<InFlightQuery>();

        drain_queries(&query_rx, &mut in_flight, "s");
        assert!(
            in_flight.is_none(),
            "in-flight query with a dead receiver is retired"
        );
    }

    #[test]
    fn drain_replaces_in_flight_with_newer_query() {
        let (q1, mut rx1) = mk_query(1);
        let mut in_flight = Some(q1);
        let (tx, query_rx) = std::sync::mpsc::channel::<InFlightQuery>();
        let (q2, _rx2) = mk_query(2);
        tx.send(q2).unwrap();

        drain_queries(&query_rx, &mut in_flight, "s");

        assert_eq!(in_flight.as_ref().unwrap().seq, 2, "newer query armed");
        // The replaced query's reply was dropped → its receiver is cancelled.
        assert!(
            matches!(
                rx1.try_recv(),
                Err(tokio::sync::oneshot::error::TryRecvError::Closed)
            ),
            "replaced query's receiver is cancelled so grpc falls back"
        );
    }
}
