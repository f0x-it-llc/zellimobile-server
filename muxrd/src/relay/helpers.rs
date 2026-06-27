//! Private relay helper: view-state initialisation.
//!
//! The fullscreen-toggle and floating-pane fill/hide action sequences that used
//! to live here are **zellij-specific**, so as of P1.03 they moved behind
//! [`crate::multiplexer::MuxSender::toggle_fullscreen`] (the zellij impl is in
//! `multiplexer::zellij`). The relay now drives them through the neutral sender.

use std::sync::Arc;

use crate::multiplexer::{MuxBackend, PaneRef};

use super::types::RelayViewState;

// ─── View-state init helper ────────────────────────────────────────────────────

/// Query live state to build the initial [`RelayViewState`] for a freshly-attached
/// relay.  Blocking — call via `spawn_blocking`.
///
/// Goes through the neutral [`MuxBackend::query_layout`] (the ephemeral path):
/// safe here because the relay's control sender is not registered until AFTER
/// this returns, so no `QueryLayout` RPC — and thus no B-QUERY race against the
/// render thread — can occur at attach time. From the neutral snapshot it derives
/// this client's active tab and the focused pane within it (a neutral [`PaneRef`],
/// plugin panes included, matching the pre-P1.03 behaviour).
pub(super) fn init_relay_view_state(
    backend: &Arc<dyn MuxBackend>,
    session: &str,
) -> anyhow::Result<RelayViewState> {
    let snapshot = backend.query_layout(session)?;

    let active = snapshot.tabs.iter().find(|t| t.active);
    let active_tab = active.map(|t| t.tab_id);

    // Focused pane within the active tab (panes are already grouped under their
    // tab in the snapshot). Plugin panes are NOT filtered here — the relay tracks
    // whichever pane zellij reports focused, same as before.
    let focused_pane = active.and_then(|t| {
        t.panes.iter().find(|p| p.is_focused).map(|p| PaneRef {
            id: p.id,
            is_plugin: p.is_plugin,
        })
    });

    Ok(RelayViewState {
        active_tab,
        focused_pane,
    })
}
