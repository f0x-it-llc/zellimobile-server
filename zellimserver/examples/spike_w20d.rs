//! spike_w20d — FA: zellij focus-while-fullscreen behaviour.
//!
//! Q: after fullscreening pane A (ToggleFocusFullscreenByPaneId) via our own client,
//! does focusing pane B (FocusPaneByPaneId) keep fullscreen on B, or exit fullscreen?
//! Read via list-panes geometry (x offset + columns): fullscreen pane has x=0 and
//! columns≈full width; a split pane is offset/narrower.
//!
//! Setup (the dev host): zellij --layout spike_w20d attach --create-background spiked
//!   cargo run --example spike_w20d -- spiked

use anyhow::{Result, bail};
use std::time::Duration;
use zellij_utils::data::{ListPanesResponse, PaneId};
use zellij_utils::input::actions::Action;
use zellimserver::ipc::AttachHandle;
use zellimserver::query;

fn dump(session: &str, label: &str) -> Result<Vec<(u32, u32, u32, bool)>> {
    let json = query::query_list_panes_json(session)?;
    let panes: ListPanesResponse =
        serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("parse: {e}"))?;
    let mut out = vec![];
    println!("--- {label} ---");
    for e in &panes {
        let p = &e.pane_info;
        if p.is_plugin {
            continue;
        }
        println!(
            "  pane id={} x={} cols={} rows={} focused={}",
            p.id, p.pane_x, p.pane_columns, p.pane_rows, p.is_focused
        );
        out.push((p.id, p.pane_x as u32, p.pane_columns as u32, p.is_focused));
    }
    Ok(out)
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

    // Persistent client at a wide size so the split is clearly measurable.
    let mut h = AttachHandle::open(&session, 50, 160)?;
    // drain a few renders so the client registers + session sizes to ours
    use zellij_utils::ipc::ServerToClientMsg;
    for _ in 0..30 {
        match h.recv() {
            Some(ServerToClientMsg::Render { .. }) => break,
            Some(_) => {}
            None => bail!("closed"),
        }
    }
    std::thread::sleep(Duration::from_millis(500));

    let base = dump(&session, "BASELINE (split)")?;
    let focused = base
        .iter()
        .find(|(_, _, _, f)| *f)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("no focused pane"))?;
    let a = focused.0;
    let b = base
        .iter()
        .find(|(id, _, _, _)| *id != a)
        .map(|(id, _, _, _)| *id)
        .ok_or_else(|| anyhow::anyhow!("need a 2nd pane (is the split layout loaded?)"))?;
    println!("focused pane A = {a}; other pane B = {b}");

    println!("\n>> fullscreen A ({a})");
    h.send_action_as_self(Action::ToggleFocusFullscreenByPaneId {
        pane_id: PaneId::Terminal(a),
    })?;
    std::thread::sleep(Duration::from_millis(800));
    let after_fs = dump(&session, "AFTER fullscreen A")?;

    println!("\n>> focus B ({b}) while fullscreen");
    h.send_action_as_self(Action::FocusPaneByPaneId {
        pane_id: PaneId::Terminal(b),
    })?;
    std::thread::sleep(Duration::from_millis(800));
    let after_focus_b = dump(&session, "AFTER focus B (still fullscreen?)")?;

    // Verdict: is B now the wide/x=0 pane (fullscreen followed focus), or did A stay full
    // (focus ignored under fullscreen), or did both return to split (fullscreen exited)?
    let wide = |v: &Vec<(u32, u32, u32, bool)>, id: u32| {
        v.iter()
            .find(|(i, _, _, _)| *i == id)
            .map(|(_, x, c, _)| (*x, *c))
    };
    println!("\n=== VERDICT ===");
    println!("baseline A {:?}  B {:?}", wide(&base, a), wide(&base, b));
    println!(
        "after FS A {:?}  B {:?}",
        wide(&after_fs, a),
        wide(&after_fs, b)
    );
    println!(
        "after focusB A {:?}  B {:?}",
        wide(&after_focus_b, a),
        wide(&after_focus_b, b)
    );
    println!("(fullscreen pane = x:0 + cols≈full; split panes = offset/narrower)");

    // restore
    println!("\n>> toggling fullscreen off (restore)");
    h.send_action_as_self(Action::ToggleFocusFullscreenByPaneId {
        pane_id: PaneId::Terminal(b),
    })?;
    std::thread::sleep(Duration::from_millis(300));
    Ok(())
}
