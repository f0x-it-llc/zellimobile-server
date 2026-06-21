//! spike_w20f — replicate the relay's FocusPane fullscreen sequence on a split
//! pane and check, via list-panes geometry, whether the target pane actually ends
//! fullscreen (and survives a resize). Headless repro of the htop/btop "not
//! fullscreen → bottom gap" bug.
//!
//! Setup (the dev host): zellij --layout spike_w20d attach --create-background spiked
//!   cargo run --example spike_w20f -- spiked

use anyhow::{Result, bail};
use std::time::Duration;
use zellij_utils::data::{ListPanesResponse, PaneId};
use zellij_utils::input::actions::Action;
use zellij_utils::ipc::ServerToClientMsg;
use zellimserver::ipc::AttachHandle;
use zellimserver::query;

/// (id, x, cols, is_focused) for terminal panes.
fn dump(session: &str, label: &str) -> Result<Vec<(u32, u32, u32, bool)>> {
    let json = query::query_list_panes_json(session)?;
    let panes: ListPanesResponse =
        serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut out = vec![];
    print!("  {label}: ");
    for e in &panes {
        let p = &e.pane_info;
        if p.is_plugin {
            continue;
        }
        print!(
            "[id={} x={} cols={} foc={}] ",
            p.id, p.pane_x, p.pane_columns, p.is_focused
        );
        out.push((p.id, p.pane_x as u32, p.pane_columns as u32, p.is_focused));
    }
    println!();
    Ok(out)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let session = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spiked".to_owned());
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(45));
        eprintln!("WATCHDOG");
        std::process::exit(2);
    });

    let mut a = AttachHandle::open(&session, 50, 160)?;
    for _ in 0..30 {
        match a.recv() {
            Some(ServerToClientMsg::Render { .. }) => break,
            Some(_) => {}
            None => bail!("closed"),
        }
    }
    std::thread::sleep(Duration::from_millis(400));

    let base = dump(&session, "baseline (split)")?;
    let p0 = base
        .iter()
        .find(|(_, _, _, f)| *f)
        .map(|t| t.0)
        .ok_or_else(|| anyhow::anyhow!("no focused"))?;
    let p1 = base
        .iter()
        .find(|(id, _, _, _)| *id != p0)
        .map(|t| t.0)
        .ok_or_else(|| anyhow::anyhow!("need 2 panes"))?;
    let full = base.iter().map(|t| t.2).max().unwrap_or(0); // session full cols (sum-ish; use as ref)
    println!("p0(focused)={p0} p1={p1} (a fullscreen pane should show x=0 cols≈session-width)");

    // initial fullscreen p0 (relay attach):
    println!(">> initial: ToggleFS(p0={p0})");
    a.send_action_as_self(Action::ToggleFocusFullscreenByPaneId {
        pane_id: PaneId::Terminal(p0),
    })?;
    std::thread::sleep(Duration::from_millis(600));
    dump(&session, "after initial FS p0")?;

    // relay FocusPane(p1): off-old(p0) + FocusPaneByPaneId(p1) + on-new(p1)
    println!(">> FocusPane(p1={p1}): off-old(p0) + focus(p1) + on-new(p1)");
    a.send_action_as_self(Action::ToggleFocusFullscreenByPaneId {
        pane_id: PaneId::Terminal(p0),
    })?; // off-old
    a.send_action_as_self(Action::FocusPaneByPaneId {
        pane_id: PaneId::Terminal(p1),
    })?;
    a.send_action_as_self(Action::ToggleFocusFullscreenByPaneId {
        pane_id: PaneId::Terminal(p1),
    })?; // on-new
    std::thread::sleep(Duration::from_millis(700));
    let after_focus = dump(&session, "after FocusPane(p1)")?;
    let p1_full_after_focus = after_focus
        .iter()
        .find(|(id, _, _, _)| *id == p1)
        .map(|(_, x, c, _)| *x == 0 && *c >= full)
        .unwrap_or(false);
    println!("   → p1 fullscreen after FocusPane? {p1_full_after_focus}");

    // simulate keyboard resize flurry
    println!(">> resize 40x150 (simulate keyboard)");
    a.send_resize(40, 150)?;
    std::thread::sleep(Duration::from_millis(700));
    let after_resize = dump(&session, "after resize")?;
    let p1_full_after_resize = after_resize
        .iter()
        .find(|(id, _, _, _)| *id == p1)
        .map(|(_, x, c, _)| *x == 0)
        .unwrap_or(false);
    println!("   → p1 still fullscreen (x=0) after resize? {p1_full_after_resize}");

    // restore
    a.send_action_as_self(Action::ToggleFocusFullscreenByPaneId {
        pane_id: PaneId::Terminal(p1),
    })?;
    std::thread::sleep(Duration::from_millis(200));
    println!(
        "\n=== VERDICT: FocusPane→fullscreen works = {p1_full_after_focus}; survives resize = {p1_full_after_resize} ==="
    );
    Ok(())
}
