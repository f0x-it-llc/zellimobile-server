//! Private helper functions: fullscreen toggling, floating-pane manipulation,
//! and relay view-state initialisation.

use zellij_utils::data::PaneId;
use zellij_utils::input::actions::Action;

use crate::ipc::AttachSender;

use super::types::RelayViewState;

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Toggle fullscreen on the relay client's CURRENTLY-ACTIVE pane — the exact
/// action the keyboard `Ctrl+p f` emits (`Action::ToggleFocusFullscreen`, no
/// pane id; server resolves the *active* pane of the *active* tab for this
/// client).
///
/// Always called right after `FocusPaneByPaneId(target)`, which sets this
/// client's active pane + active tab to `target`. Because the UI only ever
/// toggles the focused pane, this is always a clean parity toggle (enter or
/// exit) — no FS_SETTLE_MS dance needed.
pub(super) fn toggle_active_fullscreen(sender: &mut AttachSender, session: &str, why: &str) {
    log::info!("relay [{session}]: toggle-active-fullscreen ({why})");
    if let Err(e) = sender.send_action_as_self(Action::ToggleFocusFullscreen) {
        log::warn!("relay [{session}]: toggle-active-fullscreen failed: {e:#}");
    }
}

/// Make a FLOATING `pane` fill the device screen. Floating panes can't be
/// fullscreened (zellij's fullscreen toggle no-ops while floating panes are
/// visible), but they can be freely positioned/sized — so we show floating panes,
/// focus the target, and set its coordinates to the whole display area via
/// `Action::ChangeFloatingPaneCoordinates`. `borderless:true` is required so the
/// content fills (otherwise zellij keeps a 1-cell frame and shrinks the PTY).
pub(super) fn fill_floating_pane(sender: &mut AttachSender, pane: PaneId, session: &str) {
    use zellij_utils::data::FloatingPaneCoordinates;
    use zellij_utils::input::layout::PercentOrFixed;

    log::info!("relay [{session}]: fill-floating {pane:?}");
    let _ = sender.send_action_as_self(Action::ShowFloatingPanes { tab_id: None });
    let _ = sender.send_action_as_self(Action::FocusPaneByPaneId { pane_id: pane });
    let coordinates = FloatingPaneCoordinates {
        x: Some(PercentOrFixed::Percent(0)),
        y: Some(PercentOrFixed::Percent(0)),
        width: Some(PercentOrFixed::Percent(100)),
        height: Some(PercentOrFixed::Percent(100)),
        pinned: None,
        borderless: Some(true),
    };
    if let Err(e) = sender.send_action_as_self(Action::ChangeFloatingPaneCoordinates {
        pane_id: pane,
        coordinates,
    }) {
        log::warn!("relay [{session}]: fill-floating failed: {e:#}");
    }
}

/// Hide all floating panes in the active tab (so a tiled pane can be shown
/// fullscreen again after we were displaying a floating pane).
pub(super) fn hide_floating_panes(sender: &mut AttachSender, session: &str) {
    log::info!("relay [{session}]: hide-floating");
    let _ = sender.send_action_as_self(Action::HideFloatingPanes { tab_id: None });
}

// ─── View-state init helper ────────────────────────────────────────────────────

/// Query live zellij state to build the initial [`RelayViewState`] for a
/// freshly-attached relay.  Blocking — call via `spawn_blocking`.
///
/// Uses the ephemeral query path (safe here: the relay's control sender is not
/// registered until AFTER this returns, so no `QueryLayout` RPC — and thus no
/// B-QUERY race against the render thread — can occur at attach time).
pub(super) fn init_relay_view_state(session: &str) -> anyhow::Result<RelayViewState> {
    use zellij_utils::data::{ListPanesResponse, ListTabsResponse, PaneId};

    let tabs_json = crate::query::query_list_tabs_json(session)?;
    let tabs: ListTabsResponse = serde_json::from_str(&tabs_json)
        .map_err(|e| anyhow::anyhow!("init_relay_view_state: parse ListTabs JSON: {e}"))?;

    let panes_json = crate::query::query_list_panes_json(session)?;
    let panes: ListPanesResponse = serde_json::from_str(&panes_json)
        .map_err(|e| anyhow::anyhow!("init_relay_view_state: parse ListPanes JSON: {e}"))?;

    let active_tab = tabs.iter().find(|t| t.active).map(|t| t.tab_id as u64);
    let active_tab_pos = tabs.iter().find(|t| t.active).map(|t| t.position);

    // Find the focused pane in the active tab.
    let focused_pane = active_tab_pos.and_then(|pos| {
        panes
            .iter()
            .find(|e| e.tab_position == pos && e.pane_info.is_focused)
            .map(|e| {
                if e.pane_info.is_plugin {
                    PaneId::Plugin(e.pane_info.id)
                } else {
                    PaneId::Terminal(e.pane_info.id)
                }
            })
    });

    Ok(RelayViewState {
        active_tab,
        focused_pane,
    })
}
