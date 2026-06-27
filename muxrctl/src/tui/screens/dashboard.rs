//! Dashboard screen rendering.
//!
//! Owns the shared chrome (title bar, left tab list, footer) used by every
//! screen, and provides the Dashboard overview body shown when
//! [`crate::app::state::Screen::Dashboard`] is active.
//!
//! ## Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │ title bar (1 line)                                                      │
//! ├──────────────┬──────────────────────────────────────────────────────────┤
//! │ Screens tab  │ body (per-screen panel)                                  │
//! │ list (18 ch) │                                                          │
//! ├──────────────┴──────────────────────────────────────────────────────────┤
//! │ footer hint (1 line)                                                    │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! The QR overlay (when open) is painted fullscreen over this chrome by
//! [`super::render`] after `dashboard::render` returns.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::AppState;
use crate::app::state::Screen;
use crate::tui::theme::{palette, styles};

/// Render the dashboard chrome and the active-screen body.
///
/// Always uses the standard title / body / footer layout.  The QR overlay
/// (when open) is painted over this chrome by [`super::render`] after this
/// function returns — it does not need to suppress the footer.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    // Paint the base background across the whole area first.
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::BG_BASE)),
        area,
    );

    // Standard title / body / footer layout for all screens.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);
    render_title(frame, rows[0]);
    render_body(frame, state, rows[1]);
    render_footer(frame, rows[2]);
}

/// Top title bar.
fn render_title(frame: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled("muxrctl", styles::accent_bold()),
        Span::styled("  ·  muxrd control panel", styles::muted()),
    ]);
    frame.render_widget(
        Paragraph::new(title).style(Style::default().bg(palette::BG_BASE)),
        area,
    );
}

/// Body: a left tab column + the active-screen body panel.
///
/// The tab column (18 cols) is always shown; the QR overlay paints over the
/// whole frame from [`super::render`] when needed.
fn render_body(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(20)])
        .split(area);

    render_tabs(frame, state, cols[0]);
    // Delegate to the screens module which dispatches per-screen renders.
    crate::tui::screens::render_screen_body(frame, state, cols[1]);
}

/// Left tab list. The active screen is teal+bold; others are muted.
fn render_tabs(frame: &mut Frame, state: &AppState, area: Rect) {
    let lines: Vec<Line> = Screen::ALL
        .iter()
        .map(|screen| {
            let active = *screen == state.screen;
            let marker = if active { "▶ " } else { "  " };
            let style = if active {
                styles::accent_bold()
            } else {
                styles::muted()
            };
            Line::from(Span::styled(format!("{marker}{}", screen.label()), style))
        })
        .collect();

    let block = styles::panel(false).title(Span::styled(" Screens ", styles::heading()));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Dashboard overview body — shows at-a-glance status for all sub-systems.
///
/// Sections:
/// - **Daemon** — running (bind / PID / uptime / clients) or stopped.
/// - **Cert** — short fingerprint excerpt + SAN count.
/// - **Tokens** — count of stored tokens.
/// - **Network** — configured bind addr + reachable IPs.
/// - **Hints** — one-line hints pointing to each action screen.
pub fn render_overview(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = styles::panel(true).title(Span::styled(" Dashboard ", styles::heading()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area vertically: sections stacked top-to-bottom, hints at bottom.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // all overview sections
            Constraint::Length(2), // hints line
        ])
        .split(inner);

    render_overview_sections(frame, state, rows[0]);
    render_overview_hints(frame, rows[1]);
}

/// Render the four overview sections (Daemon / Cert / Tokens / Network).
fn render_overview_sections(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // ── Daemon ───────────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled("  Daemon", styles::heading())));
    let srv = &state.server;
    if crate::tui::screens::server::is_first_load(srv.loading, srv.status.is_none(), srv.stopped) {
        lines.push(Line::from(vec![
            Span::styled("    Status:  ", styles::muted()),
            Span::styled("Querying…", styles::status_warn()),
        ]));
    } else if let Some(ref info) = srv.status {
        lines.push(Line::from(vec![
            Span::styled("    Status:  ", styles::muted()),
            Span::styled("● Running", styles::status_ok()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    Bind:    ", styles::muted()),
            Span::styled(info.bind_addr.clone(), styles::accent()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    PID:     ", styles::muted()),
            Span::styled(info.pid.to_string(), styles::body()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    Uptime:  ", styles::muted()),
            Span::styled(format_uptime(info.uptime_secs), styles::body()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    Clients: ", styles::muted()),
            Span::styled(
                info.client_count.to_string(),
                if info.client_count > 0 {
                    styles::status_ok()
                } else {
                    styles::muted()
                },
            ),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("    Status:  ", styles::muted()),
            Span::styled("○ Stopped", styles::status_err()),
        ]));
    }

    lines.push(Line::from(""));

    // ── Cert ─────────────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled("  Cert", styles::heading())));
    let cert = &state.cert;
    if cert.fingerprint.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("    Fingerprint: ", styles::muted()),
            Span::styled("(none — visit Cert screen)", styles::muted()),
        ]));
    } else {
        // Display first 16 hex chars + "…" as a short fingerprint.
        let short_fp = if cert.fingerprint.len() > 16 {
            format!("{}…", &cert.fingerprint[..16])
        } else {
            cert.fingerprint.clone()
        };
        lines.push(Line::from(vec![
            Span::styled("    Fingerprint: ", styles::muted()),
            Span::styled(short_fp, styles::accent()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    SANs:        ", styles::muted()),
            Span::styled(cert.sans.len().to_string(), styles::body()),
        ]));
    }

    lines.push(Line::from(""));

    // ── Tokens ───────────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled("  Tokens", styles::heading())));
    lines.push(Line::from(vec![
        Span::styled("    Count: ", styles::muted()),
        Span::styled(state.tokens.tokens.len().to_string(), styles::body()),
    ]));

    lines.push(Line::from(""));

    // ── Network ──────────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled("  Network", styles::heading())));
    let cfg = &state.config;
    if cfg.host.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("    Bind: ", styles::muted()),
            Span::styled("(not yet loaded)", styles::muted()),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("    Bind: ", styles::muted()),
            Span::styled(cfg.bind_addr(), styles::accent()),
        ]));
        if cfg.reachable_ips.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("    IPs:  ", styles::muted()),
                Span::styled("(none discovered)", styles::muted()),
            ]));
        } else {
            let ip_list = cfg
                .reachable_ips
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(Line::from(vec![
                Span::styled("    IPs:  ", styles::muted()),
                Span::styled(ip_list, styles::body()),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Render the hints line pointing to each action screen.
fn render_overview_hints(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::styled("Config", styles::accent()),
        Span::styled(" addr  ", styles::muted()),
        Span::styled("Cert", styles::accent()),
        Span::styled(" gen  ", styles::muted()),
        Span::styled("Tokens", styles::accent()),
        Span::styled(" manage + pair  ", styles::muted()),
        Span::styled("Server", styles::accent()),
        Span::styled(" start/stop", styles::muted()),
    ]);
    frame.render_widget(
        Paragraph::new(hints).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Bottom footer with navigation hints.
fn render_footer(frame: &mut Frame, area: Rect) {
    let hint = Line::from(vec![
        Span::styled("Tab", styles::accent()),
        Span::styled("/", styles::muted()),
        Span::styled("←→", styles::accent()),
        Span::styled(" switch  ·  ", styles::muted()),
        Span::styled("q", styles::accent()),
        Span::styled("/", styles::muted()),
        Span::styled("Ctrl-C", styles::accent()),
        Span::styled(" quit", styles::muted()),
    ]);
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().bg(palette::BG_BASE)),
        area,
    );
}

/// Format an uptime duration in seconds as a human-readable string.
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
        assert_eq!(format_uptime(3600), "1h 0m");
    }

    #[test]
    fn format_uptime_days() {
        assert_eq!(format_uptime(86400), "1d 0h");
    }
}
