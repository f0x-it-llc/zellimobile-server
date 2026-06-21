//! spike_w20c — does RE-subscribing on the same bare connection re-point the
//! pane render stream? (W2.0b switching path.)
//!
//! The relay re-points the single-pane display by sending another
//! SubscribeToPaneRenders (new pane_ids) on the same connection when the user
//! switches pane/tab. If the server ignores re-subscription, switching won't
//! update the display and we'd need a teardown+reopen per switch instead.
//!
//! Setup (the dev host): same 2-tab spike_w20a layout (alpha/bravo counters).
//!   cargo run --example spike_w20c -- spikenav

use std::time::Duration;

use anyhow::{Result, bail};
use zellij_utils::data::{ListPanesResponse, PaneId};
use zellij_utils::ipc::ServerToClientMsg;
use zellimserver::ipc::AttachHandle;
use zellimserver::query;

/// Read up to `max` updates; return the marker (ALPHA/BRAVO) of the first
/// non-empty viewport seen, or None.
fn first_marker(handle: &mut AttachHandle, max: usize) -> Option<&'static str> {
    for _ in 0..max {
        match handle.recv() {
            Some(ServerToClientMsg::PaneRenderUpdate { viewport, .. }) => {
                let joined = viewport.join("\n");
                if joined.contains("ZZZ_ALPHA_ZZZ") {
                    return Some("ALPHA");
                }
                if joined.contains("ZZZ_BRAVO_ZZZ") {
                    return Some("BRAVO");
                }
            }
            Some(_) => {}
            None => return None,
        }
    }
    None
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let session = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spikenav".to_owned());

    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(40));
        eprintln!("WATCHDOG: 40s — exiting (re-subscription inconclusive/FAIL)");
        std::process::exit(2);
    });

    // Collect the two terminal panes (one per tab).
    let json = query::query_list_panes_json(&session)?;
    let panes: ListPanesResponse =
        serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("parse list-panes: {e}"))?;
    let ids: Vec<PaneId> = panes
        .into_iter()
        .filter(|e| !e.pane_info.is_plugin)
        .map(|e| PaneId::Terminal(e.pane_info.id))
        .collect();
    if ids.len() < 2 {
        bail!(
            "need ≥2 terminal panes (got {}); is the spike_w20a layout loaded?",
            ids.len()
        );
    }
    let (p0, p1) = (ids[0], ids[1]);
    log::info!("subscribing to {p0:?} first, then re-pointing to {p1:?}");

    let mut h = AttachHandle::open_pane_subscription(&session, vec![p0], true)?;
    let m0 = first_marker(&mut h, 60).unwrap_or("(none)");
    log::info!("initial subscription marker: {m0}");

    // Re-subscribe to the other pane on the SAME connection.
    h.subscribe_to_panes(vec![p1], true)?;
    let m1 = first_marker(&mut h, 60).unwrap_or("(none)");
    log::info!("after re-subscription marker: {m1}");

    println!("\n--- W2.0c re-subscription result ---");
    println!("pane {p0:?} → marker {m0}");
    println!("pane {p1:?} → marker {m1}");
    if m0 != "(none)" && m1 != "(none)" && m0 != m1 {
        println!(
            "✅ RE-SUBSCRIPTION WORKS: same-connection re-subscribe re-points the stream (switching display is viable)."
        );
    } else if m0 == m1 && m0 != "(none)" {
        println!(
            "❌ RE-SUBSCRIPTION IGNORED: stream stayed on the first pane → relay must teardown+reopen the bare connection per switch."
        );
        std::process::exit(1);
    } else {
        println!("⚠️  inconclusive (a marker was empty) — rerun.");
        std::process::exit(2);
    }
    Ok(())
}
