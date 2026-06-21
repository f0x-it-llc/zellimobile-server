//! spike_relay_query — Gate probe: can the relay route a layout query
//! (`Action::ListTabs` / `Action::ListPanes`) over its EXISTING persistent
//! `AttachClient` connection and receive the `ServerToClientMsg::Log { lines }`
//! reply back on the SAME connection?
//!
//! **The question this answers:**
//! `query.rs::query_session` opens a *fresh* ephemeral `AttachClient` per call —
//! that registers a transient extra client, causing pane-frame flicker + per-client
//! focus/tab union pollution. If routing over the persistent (relay-style) client
//! connection also returns the Log, we can eliminate the ephemeral attach entirely.
//!
//! **What the spike does:**
//! 1. Opens a single `AttachHandle` (persistent relay-style client, NEUTRAL size).
//! 2. Drains until the first Render (so the client is fully registered).
//! 3. Sends `Action::ListTabs { output_json:true, … }` via `send_action_as_self`.
//! 4. Loops on `recv()` up to 200 messages / 5 seconds, counting Render frames
//!    interleaved before the Log arrives.
//! 5. Parses the Log JSON into `ListTabsResponse`, prints focused tab info.
//! 6. Repeats for `Action::ListPanes { output_json:true, … }`, parses into
//!    `ListPanesResponse`, prints focused-pane info.
//! 7. Prints a final `SPIKE RESULT: PASS` or `SPIKE RESULT: FAIL: <why>`.
//!
//! **Setup (the dev host):**
//! ```text
//! export XDG_RUNTIME_DIR=/tmp/zrun-spikeq
//! mkdir -p $XDG_RUNTIME_DIR
//! cat > /tmp/spikeq.kdl <<'EOF'
//! layout {
//!     tab name="editor" { pane; pane; }
//!     tab name="shell"  { pane }
//! }
//! EOF
//! zellij --layout /tmp/spikeq.kdl attach --create-background spikequery
//! cargo run --example spike_relay_query -- spikequery
//! ```

use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use zellij_utils::data::{ListPanesResponse, ListTabsResponse};
use zellij_utils::input::actions::Action;
use zellij_utils::ipc::ServerToClientMsg;
use zellimserver::ipc::AttachHandle;
use zellimserver::query::{NEUTRAL_ATTACH_COLS, NEUTRAL_ATTACH_ROWS};

/// How long to wait for a Log reply after sending each query action.
const LOG_WAIT_SECS: u64 = 5;
/// Maximum messages to drain while waiting for the Log.
const LOG_DRAIN_LIMIT: usize = 200;

// ─── Query-over-persistent-connection helper ──────────────────────────────────

/// Send `action` via `send_action_as_self` over an already-attached handle and
/// wait for `ServerToClientMsg::Log { lines }` to arrive on the same connection.
///
/// Returns `(lines, render_count, total_msg_count)`:
/// - `lines`: the raw Log lines.
/// - `render_count`: how many Render frames arrived before the Log.
/// - `total_msg_count`: total messages drained (Render + other + Log).
///
/// Returns `Err` if the Log does not arrive within the budget.
fn query_via_persistent(
    handle: &mut AttachHandle,
    action: Action,
) -> Result<(Vec<String>, usize, usize)> {
    handle.send_action_as_self(action)?;

    let deadline = Instant::now() + Duration::from_secs(LOG_WAIT_SECS);
    let mut render_count = 0usize;
    let mut total = 0usize;

    for _ in 0..LOG_DRAIN_LIMIT {
        if Instant::now() > deadline {
            bail!("timed out waiting for Log after {total} messages ({render_count} Renders)");
        }

        let Some(msg) = handle.recv() else {
            bail!("IPC stream closed before Log arrived (after {total} messages)");
        };
        total += 1;

        match msg {
            ServerToClientMsg::Log { lines } => {
                return Ok((lines, render_count, total));
            }
            ServerToClientMsg::LogError { lines } => {
                bail!("server returned LogError: {lines:?}");
            }
            ServerToClientMsg::Exit { exit_reason } => {
                bail!("session exited during query: {exit_reason:?}");
            }
            ServerToClientMsg::Render { .. } => {
                render_count += 1;
            }
            other => {
                // Connected, QueryTerminalSize, etc. — drain silently.
                eprintln!("  [drain] non-Render/Log msg: {other:?}");
            }
        }
    }

    bail!(
        "exceeded {LOG_DRAIN_LIMIT} messages without a Log reply \
         ({render_count} Renders drained)"
    )
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let session = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spikequery".to_owned());

    // Global watchdog so we never hang the dev host CI pipeline.
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(60));
        eprintln!("WATCHDOG: 60s elapsed — exiting (treat as FAIL)");
        std::process::exit(2);
    });

    println!("=== spike_relay_query: session '{session}' ===");
    println!(
        "Opening ONE persistent AttachHandle (NEUTRAL size {NEUTRAL_ATTACH_ROWS}×{NEUTRAL_ATTACH_COLS})…"
    );

    let mut handle = AttachHandle::open(&session, NEUTRAL_ATTACH_ROWS, NEUTRAL_ATTACH_COLS)?;

    // Drain until the first Render so the client is fully registered with the
    // server before we send any query action.  An action sent before the attach
    // handshake is fully processed can be dropped.
    println!("Waiting for first Render (client-registration confirmation)…");
    let mut pre_renders = 0usize;
    loop {
        match handle.recv() {
            Some(ServerToClientMsg::Render { .. }) => {
                pre_renders += 1;
                println!(
                    "  -> first Render received after {pre_renders} message(s). Client registered."
                );
                break;
            }
            Some(_) => {
                pre_renders += 1;
                if pre_renders > 100 {
                    bail!("100 non-Render messages before first Render — attach may have failed");
                }
            }
            None => bail!("IPC stream closed before first Render arrived"),
        }
    }

    // ── Test 1: ListTabs ──────────────────────────────────────────────────────

    println!("\n── Test 1: ListTabs over persistent connection ──");
    let tabs_action = Action::ListTabs {
        show_state: true,
        show_dimensions: true,
        show_panes: false,
        show_layout: false,
        show_all: true,
        output_json: true,
    };

    let tabs_result = query_via_persistent(&mut handle, tabs_action);

    let tabs_pass = match &tabs_result {
        Ok((lines, renders, total)) => {
            println!(
                "  Log ARRIVED: {total} total msgs, {renders} Render frames before Log, {} Log lines",
                lines.len()
            );
            let json = lines.join("\n");
            let truncated = if json.len() > 300 {
                format!("{}…", &json[..300])
            } else {
                json.clone()
            };
            println!("  Log JSON (truncated): {truncated}");

            // Parse
            match serde_json::from_str::<ListTabsResponse>(&json) {
                Ok(tabs) => {
                    println!("  Parsed {} tab(s):", tabs.len());
                    for t in &tabs {
                        println!(
                            "    tab[{}] name={:?} active={} display={}×{}",
                            t.position,
                            t.name,
                            t.active,
                            t.display_area_rows,
                            t.display_area_columns
                        );
                    }
                    let active_count = tabs.iter().filter(|t| t.active).count();
                    println!("  Active tab count: {active_count}");
                    true
                }
                Err(e) => {
                    println!("  PARSE FAILED: {e}");
                    false
                }
            }
        }
        Err(e) => {
            println!("  FAIL: Log did NOT arrive — {e}");
            false
        }
    };

    // ── Test 2: ListPanes ──────────────────────────────────────────────────────

    println!("\n── Test 2: ListPanes over persistent connection ──");
    let panes_action = Action::ListPanes {
        show_tab: true,
        show_command: true,
        show_state: true,
        show_geometry: true,
        show_all: true,
        output_json: true,
    };

    let panes_result = query_via_persistent(&mut handle, panes_action);

    let panes_pass = match &panes_result {
        Ok((lines, renders, total)) => {
            println!(
                "  Log ARRIVED: {total} total msgs, {renders} Render frames before Log, {} Log lines",
                lines.len()
            );
            let json = lines.join("\n");
            let truncated = if json.len() > 300 {
                format!("{}…", &json[..300])
            } else {
                json.clone()
            };
            println!("  Log JSON (truncated): {truncated}");

            // Parse
            match serde_json::from_str::<ListPanesResponse>(&json) {
                Ok(panes) => {
                    println!("  Parsed {} pane entry(ies):", panes.len());
                    for entry in &panes {
                        let p = &entry.pane_info;
                        println!(
                            "    pane id={} tab={} plugin={} floating={} focused={} x={} y={} cols={} rows={}",
                            p.id,
                            entry.tab_position,
                            p.is_plugin,
                            p.is_floating,
                            p.is_focused,
                            p.pane_x,
                            p.pane_y,
                            p.pane_columns,
                            p.pane_rows
                        );
                    }
                    let focused_count = panes.iter().filter(|e| e.pane_info.is_focused).count();
                    println!("  Focused pane count: {focused_count}");
                    true
                }
                Err(e) => {
                    println!("  PARSE FAILED: {e}");
                    false
                }
            }
        }
        Err(e) => {
            println!("  FAIL: Log did NOT arrive — {e}");
            false
        }
    };

    // ── Summary ───────────────────────────────────────────────────────────────

    println!("\n=== SUMMARY ===");
    println!(
        "ListTabs  Log received via persistent conn: {}",
        if tabs_pass { "YES" } else { "NO" }
    );
    println!(
        "ListPanes Log received via persistent conn: {}",
        if panes_pass { "YES" } else { "NO" }
    );
    println!(
        "ListTabs  interleaved Renders before Log: {}",
        tabs_result.as_ref().map(|(_, r, _)| *r).unwrap_or(0)
    );
    println!(
        "ListPanes interleaved Renders before Log: {}",
        panes_result.as_ref().map(|(_, r, _)| *r).unwrap_or(0)
    );
    println!("Persistent handle opened: 1 (single AttachHandle, no ephemeral query client)");

    if tabs_pass && panes_pass {
        println!("\nSPIKE RESULT: PASS");
    } else {
        let why = match (tabs_pass, panes_pass) {
            (false, false) => "Log did not arrive for ListTabs OR ListPanes",
            (false, true) => "Log did not arrive for ListTabs",
            (true, false) => "Log did not arrive for ListPanes",
            _ => unreachable!(),
        };
        println!("\nSPIKE RESULT: FAIL: {why}");
        std::process::exit(1);
    }

    Ok(())
}
