//! GetLayout RPC implementation with relay-routed query and B-FOCUS override.

use tonic::{Request, Response, Status};

use crate::proto::{Layout, PaneMsg, SessionRef, TabMsg};

use super::ZelliService;
use super::helpers::{ephemeral_query, validate_session};

/// Timeout for the oneshot reply when routing a `QueryLayout` through the relay.
///
/// FX-QUERY: this is now the SINGLE timeout bound on a relay-routed layout query.
/// The relay's inbound arm no longer awaits (it hands the query to the render
/// thread and returns), so there is no per-action sub-timeout. When this fires we
/// drop `reply_rx`; the render thread observes the closed receiver and retires
/// the in-flight query so its stray Logs can't be misattributed. 18 s comfortably
/// covers two query round-trips (ListTabs + ListPanes) plus channel overhead.
const RELAY_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(18);

impl ZelliService {
    // ── GetLayout (C1 + BE-LAYOUT) ─────────────────────────────────────────

    pub(super) async fn get_layout_impl(
        &self,
        request: Request<SessionRef>,
    ) -> Result<Response<Layout>, Status> {
        let session = request.into_inner().session;
        validate_session(&session)?;
        log::info!("GetLayout: session='{session}'");

        // ── B-QUERY: route through relay if one is attached ─────────────────
        // Look up the relay control sender for this session. If found, send a
        // QueryLayout command and await the (tabs_json, panes_json) result via
        // a oneshot, bounded by RELAY_QUERY_TIMEOUT. Fall back to the ephemeral
        // path on timeout or relay absence.
        let (tabs_json, panes_json, via_relay) = {
            // Try to get the relay sender. Clone the UnboundedSender (cheap:
            // it's just a channel handle) so the DashMap Ref guard is dropped
            // immediately — we must not hold it across awaits.
            let relay_sender = self.control.get(&session).map(|r| r.value().clone());

            if let Some(sender) = relay_sender {
                let (reply_tx, reply_rx) =
                    tokio::sync::oneshot::channel::<anyhow::Result<(String, String)>>();
                let queued =
                    sender.send(crate::relay::RelayControl::QueryLayout { reply: reply_tx });
                // `sender` is an owned clone of the UnboundedSender (the DashMap
                // Ref was already released by `.map(|r| r.value().clone())`
                // above), so this drop is just tidiness — releasing our handle to
                // the relay's control channel now that the query is queued. It is
                // NOT releasing a DashMap guard.
                drop(sender);

                if queued.is_ok() {
                    match tokio::time::timeout(RELAY_QUERY_TIMEOUT, reply_rx).await {
                        Ok(Ok(Ok((t, p)))) => {
                            log::debug!(
                                "GetLayout: session='{session}' query routed via relay \
                                 (tabs={}B panes={}B)",
                                t.len(),
                                p.len()
                            );
                            (t, p, true)
                        }
                        Ok(Ok(Err(e))) => {
                            log::warn!(
                                "GetLayout: relay query failed for '{session}', \
                                 falling back to ephemeral: {e:#}"
                            );
                            ephemeral_query(&session).await?
                        }
                        Ok(Err(_cancelled)) => {
                            log::warn!(
                                "GetLayout: relay query oneshot cancelled for '{session}', \
                                 falling back to ephemeral"
                            );
                            ephemeral_query(&session).await?
                        }
                        Err(_elapsed) => {
                            log::warn!(
                                "GetLayout: relay query timed out for '{session}' \
                                 after {RELAY_QUERY_TIMEOUT:?}, falling back to ephemeral"
                            );
                            ephemeral_query(&session).await?
                        }
                    }
                } else {
                    // Relay sender was closed (relay tearing down) — fall back.
                    log::debug!(
                        "GetLayout: relay sender closed for '{session}', \
                         falling back to ephemeral"
                    );
                    ephemeral_query(&session).await?
                }
            } else {
                // No relay attached for this session — use the original ephemeral path.
                log::debug!("GetLayout: no relay for '{session}', using ephemeral query");
                ephemeral_query(&session).await?
            }
        };

        log::debug!(
            "GetLayout: tabs JSON ({} bytes), panes JSON ({} bytes), via_relay={via_relay}",
            tabs_json.len(),
            panes_json.len()
        );

        // ── Deserialise ─────────────────────────────────────────────────────
        use zellij_utils::data::{ListPanesResponse, ListTabsResponse, PaneId};

        // On parse failure, keep the serde detail + payload size in the server
        // log only; return a generic Status so neither the internal error nor the
        // (potentially large, cwd/command-bearing) layout JSON leaks to the client.
        let tabs: ListTabsResponse = serde_json::from_str(&tabs_json).map_err(|e| {
            log::warn!(
                "GetLayout: failed to parse ListTabs JSON ({}B): {e}",
                tabs_json.len()
            );
            Status::internal("failed to parse layout response from session")
        })?;

        let panes: ListPanesResponse = serde_json::from_str(&panes_json).map_err(|e| {
            log::warn!(
                "GetLayout: failed to parse ListPanes JSON ({}B): {e}",
                panes_json.len()
            );
            Status::internal("failed to parse layout response from session")
        })?;

        // ── B-FOCUS: read relay view state for active_tab / focused_pane ────
        // Only meaningful when a relay is attached. We snapshot it once and use
        // it for the override pass below so we hold the DashMap guard briefly.
        let relay_vs: Option<crate::relay::RelayViewState> = if via_relay {
            self.view_state.get(&session).map(|vs| vs.value().clone())
        } else {
            None
        };
        if let Some(ref vs) = relay_vs {
            log::debug!(
                "GetLayout: relay view state: active_tab={:?} focused_pane={:?}",
                vs.active_tab,
                vs.focused_pane
            );
        }

        // ── Group panes by tab_id ────────────────────────────────────────────
        let mut panes_by_tab: std::collections::HashMap<usize, Vec<PaneMsg>> =
            std::collections::HashMap::new();
        for entry in panes {
            // B-FOCUS: override is_focused with the per-relay-client value, but
            // ONLY within the relay's ACTIVE tab.
            //
            // The relay tracks a single focused pane — the one focused in ITS
            // active tab. Each tab, though, has its own independently-focused
            // pane. If we applied the override across ALL tabs we'd force
            // is_focused=false on every legitimately-focused pane of the
            // NON-active tabs (none of them match the single tracked focused_pane),
            // hiding per-tab focus from consumers. So we scope the override to the
            // active tab and leave non-active tabs' queried is_focused untouched.
            //
            // When focused_pane is None (unknown, e.g. right after a bare SwitchTab
            // with no subsequent FocusPane), leave the queried is_focused as-is
            // everywhere (best-effort).
            let in_active_tab = relay_vs
                .as_ref()
                .and_then(|vs| vs.active_tab)
                .map(|at| entry.tab_id as u64 == at)
                .unwrap_or(false);

            let is_focused = if in_active_tab {
                relay_vs
                    .as_ref()
                    .and_then(|vs| vs.focused_pane)
                    .map(|fp| match fp {
                        PaneId::Terminal(fid) => {
                            !entry.pane_info.is_plugin && entry.pane_info.id == fid
                        }
                        PaneId::Plugin(fid) => {
                            entry.pane_info.is_plugin && entry.pane_info.id == fid
                        }
                    })
                    .unwrap_or(entry.pane_info.is_focused)
            } else {
                // Non-active tab (or active_tab unknown): keep the queried value —
                // each tab carries its own focus.
                entry.pane_info.is_focused
            };

            let pane_msg = PaneMsg {
                id: entry.pane_info.id,
                title: entry.pane_info.title.clone(),
                is_focused,
                is_floating: entry.pane_info.is_floating,
                exited: entry.pane_info.exited,
                command: entry.pane_command.unwrap_or_default(),
                cwd: entry.pane_cwd.unwrap_or_default(),
                x: entry.pane_info.pane_x as u32,
                y: entry.pane_info.pane_y as u32,
                rows: entry.pane_info.pane_rows as u32,
                cols: entry.pane_info.pane_columns as u32,
                is_plugin: entry.pane_info.is_plugin,
                is_fullscreen: entry.pane_info.is_fullscreen,
            };
            panes_by_tab.entry(entry.tab_id).or_default().push(pane_msg);
        }

        // ── Build Layout ────────────────────────────────────────────────────
        let tab_msgs: Vec<TabMsg> = tabs
            .into_iter()
            .map(|tab| {
                let panes = panes_by_tab.remove(&tab.tab_id).unwrap_or_default();

                // B-FOCUS: override active with per-relay-client value.
                // When relay_vs.active_tab is Some, the relay knows exactly
                // which tab it switched to. When None (not yet set), fall back
                // to the queried tab.active (best-effort, still better than a
                // union including transient clients since we routed via relay).
                let active = relay_vs
                    .as_ref()
                    .and_then(|vs| vs.active_tab)
                    .map(|at| tab.tab_id as u64 == at)
                    .unwrap_or(tab.active);

                TabMsg {
                    position: tab.position as u32,
                    name: tab.name.clone(),
                    active,
                    has_bell: tab.has_bell_notification,
                    panes_to_hide: tab.panes_to_hide as u32,
                    tab_id: tab.tab_id as u32,
                    panes,
                    fullscreen_active: tab.is_fullscreen_active,
                    floating_panes_visible: tab.are_floating_panes_visible,
                }
            })
            .collect();

        log::info!(
            "GetLayout: session='{}' → {} tab(s), {} total pane group(s), via_relay={via_relay}",
            session,
            tab_msgs.len(),
            tab_msgs.iter().map(|t| t.panes.len()).sum::<usize>()
        );

        Ok(Response::new(Layout { tabs: tab_msgs }))
    }
}
