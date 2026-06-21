//! Tokio inbound task: drives the gRPC ClientFrame stream → IPC sender.

use futures::StreamExt;
use tonic::Streaming;

use zellij_utils::input::actions::Action;

use tokio::sync::mpsc;

use crate::ipc::AttachSender;
use crate::proto::{ClientFrame, client_frame};

use super::helpers::{fill_floating_pane, hide_floating_panes, toggle_active_fullscreen};
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
    mut sender: AttachSender,
    guard: ShutdownGuard,
    session: String,
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
    // A clone of THIS attach's registered control sender, used only as an
    // ownership token at teardown: we remove the `control` / `view_state` entries
    // only if the registered sender is still ours (a newer attach for the same
    // session may have replaced both — last-attach-wins). Never sent on.
    my_ctrl_tx: mpsc::UnboundedSender<RelayControl>,
    // Held for potential future sole-client gating; not required by the current
    // toggle logic (floating visibility queried live from zellij; tiled uses parity toggle).
    _clients: crate::client_count::SessionClients,
    // FX-QUERY: channel to the render thread carrying in-flight layout queries.
    // The QueryLayout arm hands the query off and returns — it never awaits.
    query_tx: QueryTx,
    // B-FOCUS: per-session relay view state registry.
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
                        if let Err(e) =
                            sender.send_action_as_self(Action::GoToTabById { id: tab_id })
                        {
                            log::warn!("relay inbound [{session}]: SwitchTab send failed: {e:#}");
                        } else {
                            // Update relay view state: active tab is now tab_id.
                            // focused_pane becomes None (we don't know which pane
                            // is focused in the new tab until a FocusPane follows).
                            if let Some(mut vs) = view_state.get_mut(&session) {
                                vs.active_tab = Some(tab_id);
                                vs.focused_pane = None;
                            }
                        }
                    }
                }
                Some(RelayControl::FocusPane(pane)) => {
                    if read_only {
                        log::trace!(
                            "relay inbound [{session}]: dropping FocusPane (read-only token)"
                        );
                    } else if let Err(e) =
                        sender.send_action_as_self(Action::FocusPaneByPaneId { pane_id: pane })
                    {
                        log::warn!("relay inbound [{session}]: FocusPane send failed: {e:#}");
                    } else {
                        // B-FOCUS: track focused pane for this relay client.
                        if let Some(mut vs) = view_state.get_mut(&session) {
                            vs.focused_pane = Some(pane);
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
                        // ephemeral AttachClient on the shared session).
                        //
                        // FALLBACK (no hint — keyboard-driven / hint-less
                        // callers): the original live query, derived fully from
                        // live zellij state so an out-of-band SHOW/HIDE is
                        // reflected immediately (M4 behaviour). HANG FIX
                        // (BE-HANG) preserved: the blocking query runs in
                        // spawn_blocking wrapped in tokio::time::timeout so a
                        // stalled socket can never wedge the loop; on timeout we
                        // skip the toggle rather than hang.
                        let (is_floating, floating_visible, is_focused_floating) = match hint {
                            Some(h) => (
                                h.target_is_floating,
                                h.floating_visible,
                                h.target_is_focused_floating,
                            ),
                            None => {
                                let s = session.clone();
                                let query_fut = tokio::task::spawn_blocking(move || {
                                    crate::query::pane_is_floating_with_visibility(&s, pane)
                                });
                                match tokio::time::timeout(FLOAT_QUERY_TIMEOUT, query_fut).await {
                                    Ok(join_result) => {
                                        let (f, v, focused) = join_result
                                            .unwrap_or(Ok((false, false, None)))
                                            .unwrap_or((false, false, None));
                                        (f, v, focused == Some(pane))
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

                        if is_floating {
                            // Floating path: fill-vs-hide derived fully from
                            // LIVE zellij state.  Hide only when floating panes
                            // are visible AND this specific pane is the focused
                            // floating pane (per live `is_focused` from
                            // ListPanes).  An out-of-band SHOW or HIDE by
                            // another client or the keyboard is reflected in
                            // `focused_floating` immediately, so the decision
                            // never requires a double-tap to re-sync.
                            if floating_visible && is_focused_floating {
                                hide_floating_panes(&mut sender, &session);
                                // B-FOCUS: hiding the floating panes returns focus
                                // to whatever tiled pane was focused underneath —
                                // which we don't track here. Mark unknown (None) so
                                // get_layout falls back to the queried is_focused
                                // rather than asserting this now-hidden floating
                                // pane is still focused.
                                if let Some(mut vs) = view_state.get_mut(&session) {
                                    vs.focused_pane = None;
                                }
                            } else {
                                fill_floating_pane(&mut sender, pane, &session);
                                // B-FOCUS: fill_floating_pane focuses `pane` (it
                                // sends FocusPaneByPaneId), so this relay client's
                                // focused pane is now `pane` — track it, same as
                                // the tiled branch does (folded minor).
                                if let Some(mut vs) = view_state.get_mut(&session) {
                                    vs.focused_pane = Some(pane);
                                }
                            }
                        } else {
                            // Tiled path: Ctrl+p,f keyboard cadence — focus then
                            // active-pane toggle. This is always a clean parity
                            // toggle (enter or exit on the focused pane) — no
                            // FS_SETTLE_MS exit-then-reenter dance needed.
                            if let Err(e) = sender.send_action_as_self(
                                Action::FocusPaneByPaneId { pane_id: pane },
                            ) {
                                log::warn!(
                                    "relay inbound [{session}]: ToggleFullscreen focus failed: {e:#}"
                                );
                            } else {
                                // B-FOCUS: track the pane we just focused.
                                if let Some(mut vs) = view_state.get_mut(&session) {
                                    vs.focused_pane = Some(pane);
                                }
                                toggle_active_fullscreen(&mut sender, &session, "toggle");
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

                    // Send both query actions. The render thread retires the slot
                    // (its receiver cancels) if a send fails to produce a Log; we
                    // don't need to await or own that path here.
                    let tabs_action = Action::ListTabs {
                        show_state: true,
                        show_dimensions: true,
                        show_panes: false,
                        show_layout: false,
                        show_all: true,
                        output_json: true,
                    };
                    if let Err(e) = sender.send_action_as_self(tabs_action) {
                        // The InFlightQuery is already owned by the render thread;
                        // its reply will cancel via RELAY_QUERY_TIMEOUT / close
                        // detection. Just log — we can't reach `reply` anymore.
                        log::warn!(
                            "relay inbound [{session}]: QueryLayout seq={seq}: ListTabs send \
                             failed (render thread will retire the query): {e:#}"
                        );
                        continue;
                    }
                    let panes_action = Action::ListPanes {
                        show_tab: true,
                        show_command: true,
                        show_state: true,
                        show_geometry: true,
                        show_all: true,
                        output_json: true,
                    };
                    if let Err(e) = sender.send_action_as_self(panes_action) {
                        log::warn!(
                            "relay inbound [{session}]: QueryLayout seq={seq}: ListPanes send \
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
                        } else if let Err(e) = forward_input(&mut sender, bytes) {
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
    // GetLayouts stop routing here. ONLY remove if the registered entry is still
    // ours: a newer attach for the same session may have replaced both (last
    // attach wins), and this older task must not clobber the newer one's entries.
    //
    // Ownership is keyed on the control sender's channel identity
    // (`same_channel`). If we still own `control`, we also still own the
    // `view_state` entry (both are inserted together per attach, control last),
    // so we remove both; otherwise we leave both for the newer attach.
    let still_ours = control.remove_if(&session, |_, tx| tx.same_channel(&my_ctrl_tx));
    if still_ours.is_some() {
        view_state.remove(&session);
    } else {
        log::debug!(
            "relay inbound [{session}]: teardown — control entry replaced by a newer \
             attach; leaving control + view_state intact"
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
    match tokio::task::spawn_blocking(move || {
        zellij_utils::web_authentication_tokens::validate_session_token(&token)
    })
    .await
    {
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
/// UTF-8 text goes via `WriteChars` (the A2-proven path); non-UTF-8 byte
/// sequences (e.g. raw ESC) go via `Write`.
fn forward_input(sender: &mut AttachSender, bytes: Vec<u8>) -> anyhow::Result<()> {
    match String::from_utf8(bytes) {
        Ok(text) => sender.send_chars(&text),
        Err(e) => sender.send_bytes(e.into_bytes()),
    }
}
