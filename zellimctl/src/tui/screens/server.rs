//! Server panel screen — shows daemon status + start / stop controls.
//!
//! Keys:
//! - `s`: start daemon (dispatches `StartServer`).
//! - `x`: stop daemon (dispatches `StopServer`).
//! - `r`: manual refresh (dispatches `RefreshStatus`).
//!
//! A live poll is driven by the tick counter in [`crate::app::update`] so the
//! client count and uptime update automatically while this screen is active.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::tui::theme::{palette, styles};

/// Render the Server panel screen.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let srv = &state.server;

    let block = styles::panel(true).title(Span::styled(" Server ", styles::heading()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Rows: status panel / action msg / hints.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // status info
            Constraint::Length(1), // action message / error
            Constraint::Length(2), // key hints
        ])
        .split(inner);

    render_status_panel(frame, state, rows[0]);
    render_action_msg(frame, srv.loading, &srv.action_msg, rows[1]);
    render_hints(frame, rows[2]);
}

/// Render the main status info block.
fn render_status_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let srv = &state.server;

    let block = styles::panel(false).title(Span::styled(" Daemon Status ", styles::muted()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let body: Vec<Line> = if srv.loading && srv.status.is_none() {
        // First load in progress.
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Querying server status…",
                styles::status_warn(),
            )),
        ]
    } else if let Some(ref info) = srv.status {
        // Server is Running.
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status:  ", styles::muted()),
                Span::styled("● Running", styles::status_ok()),
            ]),
            Line::from(vec![
                Span::styled("  Version: ", styles::muted()),
                Span::styled(&info.version, styles::body()),
            ]),
            Line::from(vec![
                Span::styled("  Bind:    ", styles::muted()),
                Span::styled(&info.bind_addr, styles::accent()),
            ]),
            Line::from(vec![
                Span::styled("  PID:     ", styles::muted()),
                Span::styled(info.pid.to_string(), styles::body()),
            ]),
            Line::from(vec![
                Span::styled("  Uptime:  ", styles::muted()),
                Span::styled(format_uptime(info.uptime_secs), styles::body()),
            ]),
            Line::from(vec![
                Span::styled("  Clients: ", styles::muted()),
                Span::styled(
                    info.client_count.to_string(),
                    if info.client_count > 0 {
                        styles::status_ok()
                    } else {
                        styles::muted()
                    },
                ),
            ]),
        ]
    } else {
        // Stopped.
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status:  ", styles::muted()),
                Span::styled("○ Stopped", styles::status_err()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  Press s to start the daemon.",
                styles::muted(),
            )),
        ]
    };

    frame.render_widget(
        Paragraph::new(body).style(Style::default().bg(palette::BG_SURFACE)),
        inner,
    );
}

/// Render the action message / error line.
fn render_action_msg(frame: &mut Frame, loading: bool, msg: &str, area: Rect) {
    if msg.is_empty() {
        return;
    }
    let style = if msg.starts_with("Error") {
        styles::status_err()
    } else if loading {
        styles::status_warn()
    } else {
        styles::status_ok()
    };
    frame.render_widget(
        Paragraph::new(Span::styled(msg, style)).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Render key-binding hints.
fn render_hints(frame: &mut Frame, area: Rect) {
    let hint = Line::from(vec![
        Span::styled("s", styles::accent()),
        Span::styled(" start  ", styles::muted()),
        Span::styled("x", styles::accent()),
        Span::styled(" stop  ", styles::muted()),
        Span::styled("r", styles::accent()),
        Span::styled(" refresh", styles::muted()),
    ]);
    frame.render_widget(Paragraph::new(hint), area);
}

/// Format an uptime in seconds as a human-readable string.
fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        return format!("{secs}s");
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins}m {}s", secs % 60);
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{hours}h {}m", mins % 60);
    }
    let days = hours / 24;
    format!("{days}d {}h", hours % 24)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_uptime_seconds() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(59), "59s");
    }

    #[test]
    fn format_uptime_minutes() {
        assert_eq!(format_uptime(60), "1m 0s");
        assert_eq!(format_uptime(90), "1m 30s");
        assert_eq!(format_uptime(3599), "59m 59s");
    }

    #[test]
    fn format_uptime_hours() {
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(7384), "2h 3m");
    }

    #[test]
    fn format_uptime_days() {
        assert_eq!(format_uptime(86400), "1d 0h");
        assert_eq!(format_uptime(90000), "1d 1h");
    }
}
