//! GetLayout RPC implementation with relay-routed query and B-FOCUS override.

use tonic::{Request, Response, Status};

use crate::multiplexer::{LayoutSnapshot, MuxBackend};
use crate::proto::{Layout, PaneMsg, SessionRef, TabMsg};

use super::MuxrService;
use super::helpers::validate_session;

/// Timeout for the oneshot reply when routing a `QueryLayout` through the relay.
///
/// FX-QUERY: this is now the SINGLE timeout bound on a relay-routed layout query.
/// The relay's inbound arm no longer awaits (it hands the query to the render
/// thread and returns), so there is no per-action sub-timeout. When this fires we
/// drop `reply_rx`; the render thread observes the closed receiver and retires
/// the in-flight query so its stray Logs can't be misattributed. 18 s comfortably
/// covers two query round-trips (ListTabs + ListPanes) plus channel overhead.
const RELAY_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(18);

impl MuxrService {
    // ── GetLayout (C1 + BE-LAYOUT) ─────────────────────────────────────────

    pub(super) async fn get_layout_impl(
        &self,
        request: Request<SessionRef>,
    ) -> Result<Response<Layout>, Status> {
        let req = request.into_inner();
        let session = req.session;
        let connection_id = req.connection_id;
        validate_session(&session)?;
        log::info!("GetLayout: session='{session}' connection_id='{connection_id}'");

        // ── B-QUERY: route through relay if one is attached ─────────────────
        // Routing priority:
        //   1. If connection_id non-empty AND entry exists AND session matches →
        //      route to that exact relay (per-connection routing — fixes the
        //      multi-client misroute bug).
        //   2. Otherwise → find any relay registered for the session (session-
        //      scoped fallback; preserves solo-client and legacy-client behavior).
        //   3. No relay → ephemeral backend.query_layout() path.
        //
        // Both paths now hand back a neutral `LayoutSnapshot` directly (P2.00 A-2):
        // the relay branch receives the snapshot over the QueryLayout oneshot — the
        // render thread already parsed the two captured JSON Logs into it via the
        // single backend-owned parse — and the ephemeral branch delegates to
        // `self.backend.query_layout()`, which parses internally. The gRPC layer is
        // backend-agnostic: it never touches the zellij JSON wire format.
        let (snapshot, via_relay, relay_conn_id) = {
            // Try per-connection lookup first, then session-scoped fallback.
            let relay_entry: Option<(
                String,
                tokio::sync::mpsc::UnboundedSender<crate::relay::RelayControl>,
            )> = if !connection_id.is_empty() {
                // Per-connection: validate session match before cloning sender.
                self.control
                    .get(&connection_id)
                    .filter(|entry| entry.session == session)
                    .map(|entry| (connection_id.clone(), entry.sender.clone()))
            } else {
                None
            };

            // If per-connection failed, try session-scoped fallback.
            let relay_entry = relay_entry.or_else(|| {
                self.control
                    .iter()
                    .find(|entry| entry.session == session)
                    .map(|entry| (entry.key().clone(), entry.sender.clone()))
            });

            // Destructure: (conn_id_used, sender_opt)
            let (matched_conn_id, relay_sender) = match relay_entry {
                Some((cid, sender)) => (cid, Some(sender)),
                None => (String::new(), None),
            };

            if let Some(sender) = relay_sender {
                let (reply_tx, reply_rx) =
                    tokio::sync::oneshot::channel::<anyhow::Result<LayoutSnapshot>>();
                let queued =
                    sender.send(crate::relay::RelayControl::QueryLayout { reply: reply_tx });
                // `sender` is an owned clone of the UnboundedSender; the DashMap
                // Ref guard was already released above. Drop is just tidiness.
                drop(sender);

                if queued.is_ok() {
                    match tokio::time::timeout(RELAY_QUERY_TIMEOUT, reply_rx).await {
                        Ok(Ok(Ok(snap))) => {
                            // P2.00 A-2: the render thread already parsed the two
                            // captured JSON Logs into this neutral LayoutSnapshot;
                            // the gRPC layer never sees the zellij JSON wire format.
                            log::debug!(
                                "GetLayout: session='{session}' connection_id='{matched_conn_id}' \
                                 query routed via relay ({} tab(s))",
                                snap.tabs.len()
                            );
                            (snap, true, matched_conn_id)
                        }
                        Ok(Ok(Err(e))) => {
                            // Relay query OR render-thread parse failed (P2.00 A-2
                            // moved the parse into the render thread). Either way
                            // fall back to the ephemeral path, which re-queries and
                            // may succeed. The error detail stays in the server log;
                            // it never leaks to the client (and carries no layout
                            // JSON, only a serde/empty-payload message).
                            log::warn!(
                                "GetLayout: relay query/parse failed for '{session}', \
                                 falling back to ephemeral: {e:#}"
                            );
                            let snap = backend_query_layout(&self.backend, &session).await?;
                            (snap, false, String::new())
                        }
                        Ok(Err(_cancelled)) => {
                            log::warn!(
                                "GetLayout: relay query oneshot cancelled for '{session}', \
                                 falling back to ephemeral"
                            );
                            let snap = backend_query_layout(&self.backend, &session).await?;
                            (snap, false, String::new())
                        }
                        Err(_elapsed) => {
                            log::warn!(
                                "GetLayout: relay query timed out for '{session}' \
                                 after {RELAY_QUERY_TIMEOUT:?}, falling back to ephemeral"
                            );
                            let snap = backend_query_layout(&self.backend, &session).await?;
                            (snap, false, String::new())
                        }
                    }
                } else {
                    // Relay sender was closed (relay tearing down) — fall back.
                    log::debug!(
                        "GetLayout: relay sender closed for '{session}', \
                         falling back to ephemeral"
                    );
                    let snap = backend_query_layout(&self.backend, &session).await?;
                    (snap, false, String::new())
                }
            } else {
                // No relay attached for this session — use the ephemeral backend path.
                log::debug!("GetLayout: no relay for '{session}', using ephemeral query");
                let snap = backend_query_layout(&self.backend, &session).await?;
                (snap, false, String::new())
            }
        };

        log::debug!(
            "GetLayout: {} tab(s) in snapshot, via_relay={via_relay} \
             relay_conn_id='{relay_conn_id}'",
            snapshot.tabs.len()
        );

        // ── B-FOCUS: read relay view state for active_tab / focused_pane ────
        // Only meaningful when a relay is attached AND the relay that served the
        // query is the CALLER'S OWN relay. We snapshot it once and use it for
        // the override pass below so we hold the DashMap guard briefly (never
        // across an .await).
        //
        // Override condition (Issue A fix):
        //   - `connection_id` must be non-empty (caller has a known relay id),
        //   - `relay_conn_id` must equal `connection_id` (the exact-connection
        //     path was taken — the query was served by the caller's own relay).
        //
        // When the fallback path was taken (request connection_id is empty, OR
        // the query fell back to an arbitrary sibling relay whose conn_id differs
        // from the request's), relay_vs is set to None so the override pass is
        // skipped entirely. Raw zellij tab/pane values are returned unchanged,
        // which is always correct because we have no reliable per-caller view
        // state in that case — applying a sibling relay's active_tab/focused_pane
        // would produce an actively wrong indicator (worse than the raw union).
        let relay_vs: Option<crate::relay::RelayViewState> =
            if should_apply_view_state_override(via_relay, &connection_id, &relay_conn_id) {
                // Exact-connection match: the query was served by the caller's own
                // relay. Apply the per-connection view-state override.
                self.view_state
                    .get(&relay_conn_id)
                    .map(|entry| entry.state.clone())
            } else {
                // Fallback path or no relay: skip the override.
                None
            };
        if let Some(ref vs) = relay_vs {
            log::debug!(
                "GetLayout: relay view state override applied (conn={relay_conn_id}): \
                 active_tab={:?} focused_pane={:?}",
                vs.active_tab,
                vs.focused_pane
            );
        } else if via_relay {
            log::debug!(
                "GetLayout: relay view state override SUPPRESSED (request \
                 connection_id='{connection_id}', relay_conn_id='{relay_conn_id}') — \
                 returning raw zellij values"
            );
        }

        // ── Build proto from LayoutSnapshot + B-FOCUS override ────────────────
        // Panes are already nested under their tab in LayoutSnapshot (no HashMap
        // grouping step needed). Plugin panes are filtered here (single chokepoint).
        //
        // B-FOCUS: relay_vs.focused_pane is a neutral `PaneRef` (P1.03), compared
        // against each pane's (id, is_plugin) — byte-identical to the prior
        // `PaneId::Terminal/Plugin` match.
        let tab_msgs: Vec<TabMsg> = snapshot
            .tabs
            .iter()
            .map(|tab| {
                // B-FOCUS: override active with per-relay-client value.
                // When relay_vs.active_tab is Some, the relay knows exactly
                // which tab it switched to. When None (not yet set), fall back
                // to the queried tab.active (best-effort, still better than a
                // union including transient clients since we routed via relay).
                let active = relay_vs
                    .as_ref()
                    .and_then(|vs| vs.active_tab)
                    .map(|at| tab.tab_id == at)
                    .unwrap_or(tab.active);

                // Compute once per tab (all panes in a tab share the same tab_id).
                let in_active_tab = relay_vs
                    .as_ref()
                    .and_then(|vs| vs.active_tab)
                    .map(|at| tab.tab_id == at)
                    .unwrap_or(false);

                let panes: Vec<PaneMsg> = tab
                    .panes
                    .iter()
                    // Plugin panes (background-only plugins like zellij:link, and
                    // tab-bar/status-bar) are never user-facing terminals in the
                    // Muxr model. Exclude them so they don't surface as selectable
                    // panes in the client rail/picker. Single chokepoint: GetLayout
                    // is the only RPC that returns a pane list.
                    .filter(|p| pane_is_client_visible(p.is_plugin))
                    .map(|p| {
                        // B-FOCUS: override is_focused with the per-relay-client
                        // value, but ONLY within the relay's ACTIVE tab.
                        //
                        // The relay tracks a single focused pane — the one focused
                        // in ITS active tab. Each tab, though, has its own
                        // independently-focused pane. If we applied the override
                        // across ALL tabs we'd force is_focused=false on every
                        // legitimately-focused pane of the NON-active tabs (none of
                        // them match the single tracked focused_pane), hiding per-tab
                        // focus from consumers. So we scope the override to the active
                        // tab and leave non-active tabs' queried is_focused untouched.
                        //
                        // When focused_pane is None (unknown, e.g. right after a bare
                        // SwitchTab with no subsequent FocusPane), leave the queried
                        // is_focused as-is everywhere (best-effort).
                        let is_focused = if in_active_tab {
                            relay_vs
                                .as_ref()
                                .and_then(|vs| vs.focused_pane)
                                .map(|fp| fp.is_plugin == p.is_plugin && fp.id == p.id)
                                .unwrap_or(p.is_focused)
                        } else {
                            // Non-active tab (or active_tab unknown): keep the queried
                            // value — each tab carries its own focus.
                            p.is_focused
                        };

                        PaneMsg {
                            id: p.id,
                            title: p.title.clone(),
                            is_focused,
                            is_floating: p.is_floating,
                            exited: p.exited,
                            command: p.command.clone(),
                            cwd: p.cwd.clone(),
                            x: p.x,
                            y: p.y,
                            rows: p.rows,
                            cols: p.cols,
                            is_plugin: p.is_plugin,
                            is_fullscreen: p.is_fullscreen,
                        }
                    })
                    .collect();

                TabMsg {
                    position: tab.position,
                    name: tab.name.clone(),
                    active,
                    has_bell: tab.has_bell,
                    panes_to_hide: tab.panes_to_hide,
                    tab_id: tab.tab_id as u32,
                    panes,
                    fullscreen_active: tab.fullscreen_active,
                    floating_panes_visible: tab.floating_panes_visible,
                }
            })
            .collect();

        log::info!(
            "GetLayout: session='{}' relay_conn='{relay_conn_id}' → {} tab(s), \
             {} total pane group(s), via_relay={via_relay}",
            session,
            tab_msgs.len(),
            tab_msgs.iter().map(|t| t.panes.len()).sum::<usize>()
        );

        Ok(Response::new(Layout { tabs: tab_msgs }))
    }
}

// ─── Private async helper ─────────────────────────────────────────────────────

/// Call `backend.query_layout()` in a blocking task, mapping errors to
/// [`Status`].
///
/// Used by all ephemeral paths in `get_layout_impl` (no relay attached, relay
/// fallback on error/timeout/cancel, relay sender closed). Replaces the old
/// `ephemeral_query` free function that opened two separate IPC connections.
async fn backend_query_layout(
    backend: &std::sync::Arc<dyn MuxBackend>,
    session: &str,
) -> Result<LayoutSnapshot, Status> {
    let backend = backend.clone();
    let session = session.to_owned();
    tokio::task::spawn_blocking(move || backend.query_layout(&session))
        .await
        .map_err(|e| Status::internal(format!("GetLayout query task panicked: {e}")))?
        .map_err(|e| {
            log::warn!("GetLayout ephemeral query failed: {e:#}");
            Status::internal(format!("GetLayout query failed: {e:#}"))
        })
}

// ─── Pure helpers (also used by tests) ───────────────────────────────────────

/// A pane is client-visible iff it is a real terminal pane. Plugin panes
/// (background plugins + tab-bar/status-bar) are excluded from GetLayout.
pub(crate) fn pane_is_client_visible(is_plugin: bool) -> bool {
    !is_plugin
}

/// Decide whether the B-FOCUS view-state override should be applied.
///
/// The override is only correct when the relay that served the query is the
/// CALLER'S OWN relay (exact `connection_id` match). When the fallback path was
/// taken (request `connection_id` is empty, or the resolved `relay_conn_id`
/// differs from the request's `connection_id`), applying a sibling relay's
/// `active_tab`/`focused_pane` would give the caller an actively wrong indicator
/// — worse than returning raw zellij values (Issue A fix).
///
/// Returns `true` only when all three conditions hold:
/// 1. the query was served via a relay (`via_relay`),
/// 2. the request carried a non-empty `connection_id` (caller has a known relay),
/// 3. the relay that served the query is the caller's own relay
///    (`relay_conn_id == request_connection_id`).
pub(crate) fn should_apply_view_state_override(
    via_relay: bool,
    request_connection_id: &str,
    relay_conn_id: &str,
) -> bool {
    via_relay && !request_connection_id.is_empty() && relay_conn_id == request_connection_id
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Plugin-pane visibility filter ───────────────────────────────────────

    #[test]
    fn terminal_pane_is_client_visible() {
        assert!(
            pane_is_client_visible(false),
            "terminal panes (is_plugin=false) must be visible to the client"
        );
    }

    #[test]
    fn plugin_pane_is_not_client_visible() {
        assert!(
            !pane_is_client_visible(true),
            "plugin panes (is_plugin=true) must be excluded from the client pane list"
        );
    }

    // ─── Issue A: view-state override suppression ─────────────────────────────

    #[test]
    fn override_applied_on_exact_connection_id_match() {
        // Exact match: via_relay + non-empty id + relay_conn_id == request id.
        assert!(
            should_apply_view_state_override(true, "conn-1", "conn-1"),
            "override must be applied when relay_conn_id == request connection_id"
        );
    }

    #[test]
    fn override_suppressed_when_request_connection_id_is_empty() {
        // Empty request connection_id → fallback path; no reliable caller identity.
        assert!(
            !should_apply_view_state_override(true, "", "conn-2"),
            "override must be suppressed when request connection_id is empty"
        );
    }

    #[test]
    fn override_suppressed_when_relay_conn_id_differs() {
        // relay_conn_id is a sibling relay's id — applying its view-state would
        // give the caller an actively wrong active_tab / focused_pane.
        assert!(
            !should_apply_view_state_override(true, "conn-A", "conn-B"),
            "override must be suppressed when relay_conn_id != request connection_id"
        );
    }

    #[test]
    fn override_suppressed_when_not_via_relay() {
        // Ephemeral query path: no relay view state at all.
        assert!(
            !should_apply_view_state_override(false, "conn-1", "conn-1"),
            "override must be suppressed when query was not served via relay"
        );
    }

    #[test]
    fn override_suppressed_when_all_empty() {
        // No connection_id and no relay: definitely no override.
        assert!(
            !should_apply_view_state_override(false, "", ""),
            "override must be suppressed with no relay and no connection_id"
        );
    }
}
