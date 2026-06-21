//! spike_w20e — does a BARE SubscribeToPaneRenders viewport reflect a fullscreen
//! toggled via a SEPARATE AttachClient? (Replicates the relay's two-connection model.)
//!
//! Measures pane width (max plain-text line length, ansi:false) on the bare
//! subscription before vs after fullscreening the pane via the primary AttachClient.
//! If width grows ~full → subscription honors fullscreen (FA works). If it stays at
//! the split fraction → the subscription ignores the override and FA must render via
//! the primary AttachClient's normal stream instead.
//!
//! Setup (the dev host): zellij --layout spike_w20d attach --create-background spiked
//!   cargo run --example spike_w20e -- spiked

use anyhow::{Result, bail};
use std::time::Duration;
use zellij_utils::data::{ListPanesResponse, PaneId};
use zellij_utils::input::actions::Action;
use zellij_utils::ipc::ServerToClientMsg;
use zellimserver::ipc::AttachHandle;
use zellimserver::query;

/// max visible line length over the next non-empty PaneRenderUpdate (ansi:false → plain).
fn measure(sub: &mut AttachHandle, max: usize) -> Option<usize> {
    for _ in 0..max {
        match sub.recv() {
            Some(ServerToClientMsg::PaneRenderUpdate { viewport, .. }) => {
                let w = viewport
                    .iter()
                    .map(|l| l.chars().count())
                    .max()
                    .unwrap_or(0);
                if w > 0 {
                    return Some(w);
                }
            }
            Some(_) => {}
            None => return None,
        }
    }
    None
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let session = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spiked".to_owned());
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(40));
        eprintln!("WATCHDOG 40s");
        std::process::exit(2);
    });

    // Primary AttachClient at a wide size (sets session geometry to 160 cols).
    let mut primary = AttachHandle::open(&session, 50, 160)?;
    for _ in 0..30 {
        match primary.recv() {
            Some(ServerToClientMsg::Render { .. }) => break,
            Some(_) => {}
            None => bail!("closed"),
        }
    }
    std::thread::sleep(Duration::from_millis(400));

    // Find the focused split pane.
    let json = query::query_list_panes_json(&session)?;
    let panes: ListPanesResponse =
        serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("{e}"))?;
    let focused = panes
        .iter()
        .find(|e| e.pane_info.is_focused && !e.pane_info.is_plugin)
        .map(|e| (e.pane_info.id, e.pane_info.pane_columns))
        .ok_or_else(|| anyhow::anyhow!("no focused pane"))?;
    let pane = PaneId::Terminal(focused.0);
    println!("focused pane {pane:?}; list-panes cols={}", focused.1);

    // Bare subscription (ansi:false → plain text, so line length == cols).
    let mut sub = AttachHandle::open_pane_subscription(&session, vec![pane], false)?;
    let before = measure(&mut sub, 60);
    println!("subscription viewport width BEFORE fullscreen: {before:?}");

    // Fullscreen the pane via the PRIMARY client (relay's mechanism).
    primary.send_action_as_self(Action::ToggleFocusFullscreenByPaneId { pane_id: pane })?;
    std::thread::sleep(Duration::from_millis(1000));
    // Re-measure on the SAME subscription (it should receive a fresh, wider snapshot).
    let after_same = measure(&mut sub, 80);
    println!("subscription viewport width AFTER fullscreen (same sub): {after_same:?}");

    // Also test a FRESH subscription opened after fullscreen.
    let mut sub2 = AttachHandle::open_pane_subscription(&session, vec![pane], false)?;
    let after_fresh = measure(&mut sub2, 60);
    println!("FRESH subscription viewport width after fullscreen: {after_fresh:?}");

    // list-panes geometry after fullscreen (control).
    let json2 = query::query_list_panes_json(&session)?;
    let panes2: ListPanesResponse =
        serde_json::from_str(&json2).map_err(|e| anyhow::anyhow!("{e}"))?;
    if let Some(e) = panes2.iter().find(|e| e.pane_info.id == focused.0) {
        println!(
            "list-panes cols AFTER fullscreen: {}",
            e.pane_info.pane_columns
        );
    }

    println!("\n=== VERDICT ===");
    println!("before={before:?} after(same)={after_same:?} after(fresh)={after_fresh:?}");
    println!("If after >> before (~full session 160) → subscription HONORS fullscreen.");
    println!(
        "If after ≈ before (split fraction) → subscription IGNORES fullscreen → FA needs the primary render stream."
    );

    primary.send_action_as_self(Action::ToggleFocusFullscreenByPaneId { pane_id: pane })?;
    std::thread::sleep(Duration::from_millis(200));
    Ok(())
}
