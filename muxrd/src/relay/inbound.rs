//! Tokio inbound task: drives the gRPC ClientFrame stream → IPC sender.

use std::sync::Arc;

use futures::StreamExt;
use tonic::Streaming;

use tokio::sync::mpsc;

use crate::multiplexer::{FullscreenHint, MuxBackend, MuxSender};
use crate::proto::{ClientFrame, client_frame};

use super::reader::ShutdownGuard;
use super::types::{
    ControlRegistry, FLOAT_QUERY_TIMEOUT, InFlightQuery, MAX_INPUT_FRAME_BYTES, QueryTx,
    RelayControl, TOKEN_RECHECK_INTERVAL, ViewStateRegistry,
};

// ─── inbound_loop ─────────────────────────────────────────────────────────────

/// Inbound loop — runs as a tokio task; owns the [`ShutdownGuard`] so the
/// reader thread is torn down when this returns (stream end or error).
///
/// Enforces two security invariants while the stream is live:
///
/// - **Major A (read-only gate):** when `read_only` is set, every inbound
///   input/resize frame is dropped (render-only) so a read-only token cannot
///   inject keystrokes or resize the session.  The gate also covers the two
///   geometry side-channels handled in [`attach_relay`] / [`ShutdownGuard`]:
///   the **attach-handshake size** (RO attaches use the session's current size,
///   never the client's) and the **teardown resize nudge** (suppressed for RO),
///   so a small read-only client can never shrink a writer's shared session.
/// - **Major H (token re-validation):** the bearer token is re-checked every
///   [`TOKEN_RECHECK_INTERVAL`]; on revocation/expiry/error the loop breaks,
///   dropping the guard and tearing the whole attach down.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn inbound_loop(
    mut inbound: Streaming<ClientFrame>,
    mut sender: Box<dyn MuxSender>,
    // The multiplexer backend — used for the hint-less `ToggleFullscreen`
    // fallback query (`pane_is_floating_with_visibility`). Cheap to clone (`Arc`).
    backend: Arc<dyn MuxBackend>,
    guard: ShutdownGuard,
    session: String,
    // Process-unique id minted at attach time; used as the registry key for
    // both `control` and `view_state` so concurrent relays on the same session
    // each own a distinct slot (fixes the multi-client misroute bug).
    connection_id: String,
    read_only: bool,
    token: Option<String>,
    // Decrements the session's attached-client count when this task ends
    // (any exit path). Held only for its Drop; never read.
    _client_guard: crate::client_count::ClientGuard,
    // Control commands from the unary GoToTab/FocusPane RPCs, routed through
    // this rendering client (is_cli_client:false). Registry used for teardown
    // deregistration.
    mut control_rx: mpsc::UnboundedReceiver<RelayControl>,
    control: ControlRegistry,
    // Held for potential future sole-client gating; not required by the current
    // toggle logic (floating visibility queried live from zellij; tiled uses parity toggle).
    _clients: crate::client_count::SessionClients,
    // FX-QUERY: channel to the render thread carrying in-flight layout queries.
    // The QueryLayout arm hands the query off and returns — it never awaits.
    query_tx: QueryTx,
    // B-FOCUS: per-connection relay view state registry.
    view_state: ViewStateRegistry,
) {
    // The guard lives for the body of this task; on any exit path its Drop
    // signals + joins the reader thread.
    let _guard = guard;

    // FX-QUERY: monotonic sequence id stamped on each layout query so the render
    // thread can order/replace and so logs are correlatable.
    let mut next_query_seq: u64 = 0;

    // Note: the floating fill-vs-hide decision is derived fully from live zellij
    // state (`focused_floating` + `floating_visible` from the ListPanes/ListTabs
    // query below) — there is no in-process fullscreen/fill tracker (M4 fix).

    let mut recheck = tokio::time::interval(TOKEN_RECHECK_INTERVAL);
    // The first tick fires immediately; skip re-validating right after the
    // layer already validated at open.
    recheck.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    recheck.tick().await;

    loop {
        tokio::select! {
            // ── Major H: periodic bearer re-validation ───────────────────────
            _ = recheck.tick() => {
                if !revalidate_token(token.as_deref(), &session).await {
                    log::warn!(
                        "relay inbound [{session}]: token no longer valid — tearing down stream"
                    );
                    break;
                }
            }

            // ── Control commands routed through this rendering client ──────────
            // Each arm forwards one action as this client (is_cli_client:false)
            // so the tab/pane switch / fullscreen toggle targets the rendering
            // client deterministically.
            cmd = control_rx.recv() => { match cmd {
                Some(RelayControl::SwitchTab(tab_id)) => {
                    log::trace!("relay inbound [{session}]: SwitchTab({tab_id})");
                    // Read-only: no action is sent AND view_state is left
                    // untouched. That is correct — a RO client never moves this
                    // relay client's active tab, so its tracked active_tab must
                    // not change (overriding it would make get_layout report a tab
                    // switch that never happened). B-FOCUS state is only updated
                    // on the RW path below, after the action is actually sent.
                    if !read_only {
                        if let Err(e) = sender.go_to_tab(tab_id) {
                            log::warn!("relay inbound [{session}]: SwitchTab send failed: {e:#}");
                        } else {
                            // Update relay view state: active tab is now tab_id.
                            // focused_pane becomes None (we don't know which pane
                            // is focused in the new tab until a FocusPane follows).
                            // Key by connection_id (unique per relay) so concurrent
                            // relays on the same session each update their own slot.
                            if let Some(mut entry) = view_state.get_mut(&connection_id) {
                                entry.state.active_tab = Some(tab_id);
                                entry.state.focused_pane = None;
                            }
                        }
                    }
                }
                Some(RelayControl::FocusPane(pane)) => {
                    if read_only {
                        log::trace!(
                            "relay inbound [{session}]: dropping FocusPane (read-only token)"
                        );
                    } else if let Err(e) = sender.focus_pane(pane) {
                        log::warn!("relay inbound [{session}]: FocusPane send failed: {e:#}");
                    } else {
                        // B-FOCUS: track focused pane for this relay client.
                        // Key by connection_id so concurrent relays each update their own slot.
                        if let Some(mut entry) = view_state.get_mut(&connection_id) {
                            entry.state.focused_pane = Some(pane);
                        }
                    }
                }
                Some(RelayControl::ToggleFullscreen { pane, hint }) => {
                    if read_only {
                        log::trace!(
                            "relay inbound [{session}]: dropping ToggleFullscreen (read-only token)"
                        );
                    } else {
                        // Resolve the floating context: (is_floating,
                        // floating_visible, is_focused_floating).
                        //
                        // Bug 2c: prefer the CLIENT HINT — the mobile client
                        // already polls all three, so a hint lets us skip a
                        // synchronous IPC query on this select-loop hot path
                        // (the query stalled input forwarding + the bearer
                        // recheck for a whole round trip, and spawned yet another
                        // ephemeral client on the shared session).
                        //
                        // FALLBACK (no hint — keyboard-driven / hint-less
                        // callers): a live query through the backend so an
                        // out-of-band SHOW/HIDE is reflected immediately (M4
                        // behaviour). HANG FIX (BE-HANG) preserved: the blocking
                        // query runs in spawn_blocking wrapped in
                        // tokio::time::timeout so a stalled socket can never wedge
                        // the loop; on timeout we skip the toggle rather than hang.
                        let resolved = match hint {
                            Some(h) => FullscreenHint {
                                is_floating: h.target_is_floating,
                                floating_visible: h.floating_visible,
                                is_focused_floating: h.target_is_focused_floating,
                            },
                            None => {
                                let backend = backend.clone();
                                let s = session.clone();
                                let query_fut = tokio::task::spawn_blocking(move || {
                                    backend.pane_is_floating_with_visibility(&s, pane)
                                });
                                match tokio::time::timeout(FLOAT_QUERY_TIMEOUT, query_fut).await {
                                    Ok(join_result) => {
                                        let (f, v, focused) = join_result
                                            .unwrap_or(Ok((false, false, None)))
                                            .unwrap_or((false, false, None));
                                        FullscreenHint {
                                            is_floating: f,
                                            floating_visible: v,
                                            is_focused_floating: focused == Some(pane),
                                        }
                                    }
                                    Err(_elapsed) => {
                                        log::warn!(
                                            "relay inbound [{session}]: floating-pane query \
                                             timed out after {FLOAT_QUERY_TIMEOUT:?} — \
                                             skipping ToggleFullscreen to avoid wedging the loop"
                                        );
                                        // Degrade: skip the toggle rather than hang.
                                        // The user can retry; the session is not frozen.
                                        continue;
                                    }
                                }
                            }
                        };

                        // The fill-vs-hide-vs-tiled action sequence is
                        // backend-specific and lives behind `toggle_fullscreen`
                        // (the zellij impl in `multiplexer::zellij`). The relay
                        // updates its OWN view state from the SAME resolved hint:
                        //   - hide path (floating, visible, focused) → focus is
                        //     handed back to an untracked tiled pane → None;
                        //   - fill path / tiled path → this client now focuses
                        //     `pane` → Some(pane).
                        let is_hide = resolved.is_floating
                            && resolved.floating_visible
                            && resolved.is_focused_floating;
                        if let Err(e) = sender.toggle_fullscreen(pane, resolved) {
                            log::warn!(
                                "relay inbound [{session}]: ToggleFullscreen failed: {e:#}"
                            );
                        } else {
                            // Key by connection_id so concurrent relays each update their own slot.
                            if let Some(mut entry) = view_state.get_mut(&connection_id) {
                                entry.state.focused_pane = if is_hide { None } else { Some(pane) };
                            }
                        }
                    }
                }

                // B-QUERY (BE-LAYOUT; FX-QUERY redesign): route a layout query
                // over this relay's existing persistent connection. This
                // eliminates the ephemeral AttachClient that query_session opens
                // for each GetLayout poll, stopping both the per-client focus/tab
                // union pollution and the pane-frame flicker caused by
                // attach/detach churn.
                //
                // CRITICAL (FX-QUERY): this arm NEVER awaits. The render thread
                // exclusively owns recv() and so is the only place a Log can be
                // seen — so it also owns reply-fulfillment. We:
                //   1. stamp a monotonic seq,
                //   2. hand InFlightQuery { seq, reply, … } to the render thread,
                //   3. send ListTabs THEN ListPanes,
                //   4. return immediately.
                // The render thread captures the two Logs (tabs then panes) and
                // fulfills `reply`. The single timeout bound is RELAY_QUERY_TIMEOUT
                // in grpc.rs; on timeout it drops the receiver and the render
                // thread retires the slot. Awaiting here would block input
                // forwarding + the bearer recheck for up to the full query budget
                // — exactly the select-loop block this redesign removes.
                Some(RelayControl::QueryLayout { reply }) => {
                    let seq = next_query_seq;
                    next_query_seq = next_query_seq.wrapping_add(1);
                    log::debug!("relay inbound [{session}]: QueryLayout seq={seq} requested");

                    // Hand the query to the render thread BEFORE sending the
                    // actions, so the first Log can't arrive before the render
                    // thread has the slot armed. (Even if it momentarily does,
                    // the post-recv drain in render_loop picks it up; arming
                    // first is the simpler ordering.)
                    let in_flight = InFlightQuery { seq, reply, tabs: None };
                    if let Err(returned) = query_tx.send(in_flight) {
                        // Render thread is gone; reply via the sender we get back
                        // so grpc falls back to the ephemeral path.
                        log::warn!(
                            "relay inbound [{session}]: QueryLayout seq={seq}: render thread \
                             gone (query_tx send failed)"
                        );
                        let _ = returned
                            .0
                            .reply
                            .send(Err(anyhow::anyhow!("render thread not available")));
                        continue;
                    }

                    // Fire the layout query (ListTabs THEN ListPanes) over the
                    // neutral sender. The InFlightQuery is already owned by the
                    // render thread; if a send fails to produce a Log, its reply
                    // cancels via RELAY_QUERY_TIMEOUT / close detection. We just
                    // log — we can't reach `reply` from here anymore.
                    if let Err(e) = sender.query_layout() {
                        log::warn!(
                            "relay inbound [{session}]: QueryLayout seq={seq}: query_layout send \
                             failed (render thread will retire the query): {e:#}"
                        );
                        continue;
                    }
                    log::trace!(
                        "relay inbound [{session}]: QueryLayout seq={seq} dispatched \
                         (render thread will fulfill)"
                    );
                }

                None => {
                    // All senders dropped — registry entry already gone or being
                    // replaced; nothing to route. (Loop continues on other arms.)
                }
            } }

            // ── Inbound client frames ────────────────────────────────────────
            next = inbound.next() => match next {
                Some(Ok(frame)) => match frame.kind {
                    Some(client_frame::Kind::Input(bytes)) => {
                        // Major A: read-only tokens may observe but not inject.
                        if read_only {
                            log::trace!(
                                "relay inbound [{session}]: dropping input frame (read-only token)"
                            );
                        } else if bytes.len() > MAX_INPUT_FRAME_BYTES {
                            // Round-2 minor: cap per-frame input size (matches the
                            // WriteToPane cap) so one frame can't push an
                            // unbounded write into the session IPC channel.
                            log::warn!(
                                "relay inbound [{session}]: dropping oversized input frame \
                                 ({} bytes > {MAX_INPUT_FRAME_BYTES} byte limit)",
                                bytes.len()
                            );
                        } else if let Err(e) = forward_input(&mut *sender, bytes) {
                            log::warn!("relay inbound [{session}]: input send failed: {e:#}");
                        }
                    }
                    Some(client_frame::Kind::Resize(r)) => {
                        // Major A: resize is a mutating control too — drop for RO.
                        if read_only {
                            log::trace!(
                                "relay inbound [{session}]: dropping resize frame (read-only token)"
                            );
                        } else {
                            let rows = super::clamp_dim(r.rows, 24);
                            let cols = super::clamp_dim(r.cols, 80);
                            if let Err(e) = sender.send_resize(rows, cols) {
                                log::warn!("relay inbound [{session}]: resize send failed: {e:#}");
                            }
                        }
                    }
                    Some(client_frame::Kind::Attach(_)) => {
                        log::warn!(
                            "relay inbound [{session}]: unexpected second AttachReq — ignoring"
                        );
                    }
                    None => {
                        log::warn!("relay inbound [{session}]: ClientFrame with no kind — ignoring");
                    }
                },
                Some(Err(e)) => {
                    log::info!("relay inbound [{session}]: stream error (client gone): {e}");
                    break;
                }
                None => {
                    log::info!("relay inbound [{session}]: stream ended (client detached)");
                    break;
                }
            }
        }
    }
    // Deregister this relay's control channel + view state so stale unary RPCs /
    // GetLayouts stop routing here.
    //
    // Because entries are keyed by connection_id (process-unique per relay),
    // removing by connection_id is always safe: we can ONLY ever remove our OWN
    // entry — a newer attach for the same session has a DIFFERENT connection_id
    // and therefore a different key. The old `same_channel` guard (which was
    // needed when keys were session-keyed and last-attach-wins could overwrite) is
    // no longer necessary for correctness, but we keep a remove_if for the view
    // state as an extra safety belt: if for any reason the entry was already
    // removed (e.g. by an explicit deregistration path in the future), the remove
    // is a harmless no-op.
    let ctrl_removed = control.remove(&connection_id);
    let vs_removed = view_state.remove(&connection_id);
    if ctrl_removed.is_some() || vs_removed.is_some() {
        log::debug!(
            "relay inbound [{session}] connection_id={connection_id}: teardown — \
             removed registry entries"
        );
    } else {
        log::debug!(
            "relay inbound [{session}] connection_id={connection_id}: teardown — \
             registry entries already absent (no-op)"
        );
    }
    // _guard drops here → reader thread shutdown.
}

// ─── Token re-validation ──────────────────────────────────────────────────────

/// Re-validate the attach's bearer token (Major H).  Runs the blocking SQLite
/// check on the blocking pool.  Returns `true` only if the token is still
/// present and unexpired; `false` (revoke the stream) on absence, invalidity,
/// or any error — fail closed.
async fn revalidate_token(token: Option<&str>, session: &str) -> bool {
    let Some(token) = token else {
        log::warn!("relay inbound [{session}]: no token to re-validate → failing closed");
        return false;
    };
    let token = token.to_owned();
    match tokio::task::spawn_blocking(move || crate::ipc::validate_session_token(&token)).await {
        Ok(Ok(valid)) => valid,
        Ok(Err(e)) => {
            log::warn!("relay [{session}]: token re-validation DB error (failing closed): {e}");
            false
        }
        Err(e) => {
            log::warn!(
                "relay [{session}]: token re-validation task panicked (failing closed): {e}"
            );
            false
        }
    }
}

// ─── Input forwarding ────────────────────────────────────────────────────────

/// Forward raw input bytes to the focused pane.
///
/// UTF-8 text goes via `send_input_chars` (the A2-proven `WriteChars` path);
/// non-UTF-8 byte sequences (e.g. raw ESC) go via `send_input_bytes` (`Write`).
fn forward_input(sender: &mut dyn MuxSender, bytes: Vec<u8>) -> anyhow::Result<()> {
    match String::from_utf8(bytes) {
        Ok(text) => sender.send_input_chars(&text),
        Err(e) => sender.send_input_bytes(e.into_bytes()),
    }
}
