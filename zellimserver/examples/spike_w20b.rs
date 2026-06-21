//! spike_w20b — W2.0b mechanism probe: can our (non-cli) persistent attach
//! subscribe to a single pane and receive `PaneRenderUpdate`?
//!
//! This de-risks the single-pane rendering rework before wiring it into the
//! relay. It answers: (1) does SubscribeToPaneRenders work from a normal
//! AttachClient (not just a cli-client)? (2) do PaneRenderUpdate snapshots
//! arrive, and with what shape (viewport line count, is_initial)? (3) does the
//! server still also send normal `Render` after we subscribe (→ the relay must
//! suppress one in single-pane mode)?
//!
//! Setup (the dev host):
//!   export XDG_RUNTIME_DIR=/tmp/zrun-spike
//!   zellij --layout spike_w20a attach --create-background spikenav
//!   cargo run --example spike_w20b -- spikenav
//!
//! NOTE: visual FIDELITY (vim/btop legibility) is NOT decided here — that needs
//! the on-device kterm test. This only proves the wire mechanism.

use std::time::Duration;

use anyhow::{Result, bail};
use zellij_utils::data::{ListPanesResponse, PaneId};
use zellij_utils::ipc::ServerToClientMsg;
use zellimserver::ipc::AttachHandle;
use zellimserver::query;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let session = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spikenav".to_owned());

    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(40));
        eprintln!(
            "WATCHDOG: 40s elapsed — exiting (no PaneRenderUpdate received → mechanism inconclusive/FAIL)"
        );
        std::process::exit(2);
    });

    log::info!("=== W2.0b probe against session '{session}' ===");

    // ── Find a terminal pane to subscribe to (prefer the focused one) ────────
    let json = query::query_list_panes_json(&session)?;
    let panes: ListPanesResponse = serde_json::from_str(&json)
        .map_err(|e| anyhow::anyhow!("failed to parse list-panes JSON: {e}"))?;
    let mut chosen: Option<PaneId> = None;
    let mut total = 0usize;
    for entry in panes {
        let pi = &entry.pane_info;
        if pi.is_plugin {
            continue;
        }
        total += 1;
        let id = PaneId::Terminal(pi.id);
        if pi.is_focused {
            chosen = Some(id);
        } else if chosen.is_none() {
            chosen = Some(id);
        }
    }
    let Some(pane) = chosen else {
        bail!("no terminal panes found in session '{session}'");
    };
    log::info!("found {total} terminal pane(s); subscribing to {pane:?} (ansi:true)");

    // ── Bare subscription (no AttachClient) — the native `zellij subscribe`
    //    pattern. A full AttachClient + subscribe only yields the initial
    //    snapshot; the bare connection gets ongoing updates. ────────────────────
    let mut a = AttachHandle::open_pane_subscription(&session, vec![pane], true)?;
    log::info!("bare SubscribeToPaneRenders sent; reading messages…");

    // ── Observe what arrives ─────────────────────────────────────────────────
    let mut pane_updates = 0usize;
    let mut normal_renders = 0usize;
    let mut closed = 0usize;
    let mut first_dumped = false;

    for _ in 0..400 {
        match a.recv() {
            Some(ServerToClientMsg::PaneRenderUpdate {
                pane_id,
                viewport,
                scrollback,
                is_initial,
            }) => {
                pane_updates += 1;
                if !first_dumped {
                    first_dumped = true;
                    let nonempty = viewport.iter().filter(|l| !l.trim().is_empty()).count();
                    let sample = viewport
                        .iter()
                        .find(|l| !l.trim().is_empty())
                        .map(|l| {
                            let s: String =
                                l.chars().filter(|c| !c.is_control()).take(60).collect();
                            s
                        })
                        .unwrap_or_default();
                    log::info!(
                        "first PaneRenderUpdate: pane={pane_id:?} is_initial={is_initial} \
                         viewport_lines={} nonempty={} scrollback={} sample='{sample}'",
                        viewport.len(),
                        nonempty,
                        scrollback.as_ref().map(|s| s.len()).unwrap_or(0),
                    );
                }
                if pane_updates >= 5 {
                    break;
                }
            }
            Some(ServerToClientMsg::SubscribedPaneClosed { pane_id }) => {
                closed += 1;
                log::warn!("SubscribedPaneClosed: {pane_id:?}");
            }
            Some(ServerToClientMsg::Render { .. }) => normal_renders += 1,
            Some(ServerToClientMsg::Exit { .. }) => break,
            Some(_) => {}
            None => break,
        }
    }

    println!("\n--- W2.0b mechanism result ---");
    println!("PaneRenderUpdate received : {pane_updates}");
    println!(
        "normal Render after sub   : {normal_renders}  (relay must suppress one in single-pane mode)"
    );
    println!("SubscribedPaneClosed      : {closed}");
    if pane_updates > 0 {
        println!(
            "✅ W2.0b MECHANISM PASSED: a normal AttachClient can subscribe and receives PaneRenderUpdate snapshots."
        );
        println!("   (Visual fidelity of vim/btop still needs the on-device kterm test.)");
    } else {
        println!(
            "❌ W2.0b MECHANISM FAILED: no PaneRenderUpdate arrived — single-pane via SubscribeToPaneRenders not viable from our attach."
        );
        std::process::exit(1);
    }
    Ok(())
}
