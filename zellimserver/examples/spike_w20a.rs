//! spike_w20a — W2.0a gate probe: does an `is_cli_client:false` action sent
//! over the *persistent* attach switch **this** client's focused tab?
//!
//! Root-cause recap (RESEARCH-nav-bug.md): the relay sends everything as
//! `is_cli_client:true` Actions, which zellij routes to `get_last_active_client()`
//! (the client that last sent a `Key`) — never the relay's own client. So the
//! mobile client's tab never moves. The proposed fix is to send the focus action
//! as `is_cli_client:false` over the relay connection so it targets the relay's
//! own client_id.
//!
//! This probe proves the fix and characterises the bug, headlessly.
//!
//! Setup (the dev host, no PTY needed):
//!   export XDG_RUNTIME_DIR=/tmp/zrun-spike
//!   zellij --layout spike_w20a attach --create-background spikenav
//!   cargo run --example spike_w20a -- spikenav
//!
//! Layout `spike_w20a` must have tab 1 showing ZZZ_ALPHA_ZZZ and tab 2 showing
//! ZZZ_BRAVO_ZZZ (see docker/spike_w20a.kdl), each redrawing ~1/s.
//!
//! PASS (gate): after `GoToTab{index:1}` via send_action_as_self the render shows
//! ALPHA; after `GoToTab{index:2}` it shows BRAVO.

use std::time::Duration;

use anyhow::{Result, bail};
use zellij_utils::input::actions::Action;
use zellij_utils::ipc::ServerToClientMsg;
use zellimserver::ipc::AttachHandle;

const ALPHA: &str = "ZZZ_ALPHA_ZZZ";
const BRAVO: &str = "ZZZ_BRAVO_ZZZ";

/// Read up to `max_msgs` server messages, returning the first render content
/// that contains `marker`, or None if not seen within the budget.
fn wait_for_marker(handle: &mut AttachHandle, marker: &str, max_msgs: usize) -> Option<String> {
    for _ in 0..max_msgs {
        match handle.recv() {
            Some(ServerToClientMsg::Render { content }) => {
                if content.contains(marker) {
                    return Some(content);
                }
            }
            Some(_) => {}
            None => return None,
        }
    }
    None
}

/// True if `marker` is seen in any render within `max_msgs` (best-effort,
/// used for the negative/differential observation).
fn marker_seen(handle: &mut AttachHandle, marker: &str, max_msgs: usize) -> bool {
    wait_for_marker(handle, marker, max_msgs).is_some()
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let session = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spikenav".to_owned());

    // Global watchdog: if any blocking recv stalls (e.g. the switch produced no
    // render because it failed), don't hang the probe.
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(40));
        eprintln!("WATCHDOG: 40s elapsed — exiting (treat as inconclusive/FAIL)");
        std::process::exit(2);
    });

    log::info!("=== W2.0a probe against session '{session}' ===");

    // ── Client A = the 'mobile relay' client ────────────────────────────────
    let mut a = AttachHandle::open(&session, 40, 120)?;
    log::info!("[A] attached; draining initial render (registers the client before we act)…");

    // CRITICAL: drain to the first render before sending any action — an action
    // sent before the client is fully registered is dropped. Initial tab is
    // 'alpha' (focus=true in the layout).
    if wait_for_marker(&mut a, ALPHA, 40).is_none() {
        bail!("[A] never saw initial ALPHA render — layout wrong or attach failed");
    }
    log::info!("[A] registered, on ALPHA ✓");

    // ── GATE: the fix — self-action switches A's own tab ─────────────────────
    log::info!("[A] sending GoToTab{{2}} as SELF (is_cli_client:false) …");
    a.send_action_as_self(Action::GoToTab { index: 2 })?;
    if wait_for_marker(&mut a, BRAVO, 40).is_none() {
        println!("\n❌ W2.0a GATE FAILED: self-action did NOT switch A from ALPHA to BRAVO.");
        std::process::exit(1);
    }
    log::info!("[A] switched ALPHA → BRAVO via self-action ✓");

    log::info!("[A] sending GoToTab{{1}} as SELF …");
    a.send_action_as_self(Action::GoToTab { index: 1 })?;
    if wait_for_marker(&mut a, ALPHA, 40).is_none() {
        println!("\n❌ W2.0a GATE FAILED: self-action did NOT switch A back to ALPHA.");
        std::process::exit(1);
    }
    log::info!("[A] switched BRAVO → ALPHA via self-action ✓");

    println!(
        "\n✅ W2.0a GATE PASSED: is_cli_client:false over the persistent attach deterministically switches THIS client's tab."
    );

    // ── Differential: characterise the current (broken) is_cli_client:true path ─
    // Open a competing client B and make it the last-active client by sending a
    // keystroke. Then A's is_cli_client:true action should be routed to B, not A.
    log::info!("--- differential: opening competing client B + making it last-active ---");
    match AttachHandle::open(&session, 40, 120) {
        Ok(mut b) => {
            // Drain B's initial render so it's registered, then B types → B
            // becomes get_last_active_client().
            let _ = wait_for_marker(&mut b, "ZZZ_", 40);
            let _ = b.send_chars(" ");
            std::thread::sleep(Duration::from_millis(400));

            // A is currently on ALPHA. Try to switch A → BRAVO the OLD way
            // (is_cli_client:true) while B is the last-active client.
            log::info!(
                "[A] sending GoToTab{{2}} as CLI (is_cli_client:true) with B as last-active …"
            );
            a.send_action(Action::GoToTab { index: 2 })?;
            let a_switched_via_cli = marker_seen(&mut a, BRAVO, 30);
            if a_switched_via_cli {
                println!(
                    "ℹ️  differential: is_cli_client:true ALSO switched A (no competing-client steal observed in this run)."
                );
            } else {
                println!(
                    "ℹ️  differential: is_cli_client:true did NOT switch A (routed to competing client B) — this is the live bug."
                );
            }

            // Now the fix again, with B still last-active: self-action must still switch A → BRAVO.
            a.send_action_as_self(Action::GoToTab { index: 2 })?;
            if wait_for_marker(&mut a, BRAVO, 40).is_some() {
                println!(
                    "✅ differential: with B as last-active, is_cli_client:false STILL switches A → fix is robust to competing clients."
                );
            } else {
                println!(
                    "⚠️  differential: self-action failed to switch A while B was last-active — investigate."
                );
            }
        }
        Err(e) => log::warn!("could not open competing client B ({e:#}); skipping differential"),
    }

    log::info!("=== probe complete ===");
    Ok(())
}
