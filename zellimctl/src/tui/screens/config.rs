//! Config screen — bind host / port form + reachable-IP picker + SAN list.
//!
//! Keys:
//! - Tab / Down / Right: advance focus (Host → Port → IpPicker → Host).
//! - BackTab / Up: retreat focus.
//! - In IpPicker: Up / Down scroll list; Enter selects IP into Host field.
//! - While editing Host/Port: `Enter` or `Ctrl-S` saves; `Ctrl-R` reloads
//!   (so `s`/`r`/`q` are literal characters and hostnames can be typed).
//! - In IpPicker (a list, not a text field): bare `s` saves, `r` reloads.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::AppState;
use crate::app::state::ConfigField;
use crate::tui::theme::{palette, styles};

/// Render the Config screen.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let cfg = &state.config;

    // Outer bordered panel.
    let block = styles::panel(true).title(Span::styled(" Config ", styles::heading()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split: top form / bottom status line.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(inner);

    render_form(frame, state, rows[0]);
    render_status(frame, cfg.loading, &cfg.status, rows[1]);
}

/// Render the bind-address form and IP picker.
fn render_form(frame: &mut Frame, state: &AppState, area: Rect) {
    let cfg = &state.config;

    // Vertical sections: Host / Port inputs, then IP picker.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Host input
            Constraint::Length(3), // Port input
            Constraint::Min(4),    // IP picker
            Constraint::Length(2), // key hints
        ])
        .split(area);

    // ── Host input ────────────────────────────────────────────────────────────
    {
        let active = cfg.focused == ConfigField::Host;
        let block = styles::panel(active).title(Span::styled(" Bind Host ", styles::muted()));
        let value = Span::styled(
            format!(
                " {} ",
                if cfg.host.is_empty() {
                    "…"
                } else {
                    &cfg.host
                }
            ),
            if active {
                styles::accent()
            } else {
                styles::body()
            },
        );
        frame.render_widget(Paragraph::new(Line::from(value)).block(block), rows[0]);
    }

    // ── Port input ────────────────────────────────────────────────────────────
    {
        let active = cfg.focused == ConfigField::Port;
        let block = styles::panel(active).title(Span::styled(" Port ", styles::muted()));
        let value = Span::styled(
            format!(
                " {} ",
                if cfg.port.is_empty() {
                    "…"
                } else {
                    &cfg.port
                }
            ),
            if active {
                styles::accent()
            } else {
                styles::body()
            },
        );
        frame.render_widget(Paragraph::new(Line::from(value)).block(block), rows[1]);
    }

    // ── Reachable-IP picker ───────────────────────────────────────────────────
    {
        let active = cfg.focused == ConfigField::IpPicker;
        let picker_block = styles::panel(active).title(Span::styled(
            " Reachable IPs (Enter to select) ",
            styles::muted(),
        ));

        if cfg.reachable_ips.is_empty() {
            let body = Paragraph::new(Span::styled(
                "  No reachable IPs discovered.",
                styles::muted(),
            ))
            .block(picker_block);
            frame.render_widget(body, rows[2]);
        } else {
            let items: Vec<ListItem> = cfg
                .reachable_ips
                .iter()
                .map(|ip| {
                    ListItem::new(Line::from(Span::styled(format!("  {ip}"), styles::body())))
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(cfg.ip_cursor));

            let list = List::new(items)
                .block(picker_block)
                .highlight_style(Style::default().bg(palette::BG_RAISED).fg(palette::TEAL))
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(list, rows[2], &mut list_state);
        }
    }

    // ── Key hints ─────────────────────────────────────────────────────────────
    {
        let hint = Line::from(vec![
            Span::styled("Tab", styles::accent()),
            Span::styled(" focus  ", styles::muted()),
            Span::styled("Enter", styles::accent()),
            Span::styled(" pick IP / save  ", styles::muted()),
            Span::styled("Ctrl-S", styles::accent()),
            Span::styled(" save  ", styles::muted()),
            Span::styled("Ctrl-R", styles::accent()),
            Span::styled(" reload", styles::muted()),
        ]);
        frame.render_widget(Paragraph::new(hint), rows[3]);
    }
}

/// Render the single-line status bar at the bottom.
fn render_status(frame: &mut Frame, loading: bool, status: &str, area: Rect) {
    let style = if status.starts_with("Error") {
        styles::status_err()
    } else if loading {
        styles::status_warn()
    } else {
        styles::status_ok()
    };
    let text = if loading && status.is_empty() {
        "Loading config…"
    } else {
        status
    };
    frame.render_widget(
        Paragraph::new(Span::styled(text, style)).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}
