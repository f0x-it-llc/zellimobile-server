//! Cert screen — show TLS certificate fingerprint + SANs; (re)generate with `g`.
//!
//! Keys:
//! - `g`: ensure / regenerate the cert using SANs built from the Config host.
//! - `r`: same as `g` (refresh).
//!
//! The fingerprint is displayed prominently (it is what the pairing QR pins).

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::tui::theme::{palette, styles};

/// Render the Cert screen.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let cert = &state.cert;

    let block = styles::panel(true).title(Span::styled(" Cert ", styles::heading()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Rows: fingerprint section / SANs list / status / hints.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // fingerprint panel
            Constraint::Min(3),    // SANs panel
            Constraint::Length(1), // status line
            Constraint::Length(2), // key hints
        ])
        .split(inner);

    render_fingerprint(frame, cert.loading, &cert.fingerprint, rows[0]);
    render_sans(frame, &cert.sans, rows[1]);
    render_status(frame, cert.loading, &cert.status, rows[2]);
    render_hints(frame, rows[3]);
}

/// Render the fingerprint prominently inside a bordered sub-panel.
fn render_fingerprint(frame: &mut Frame, loading: bool, fingerprint: &str, area: Rect) {
    let block = styles::panel(false).title(Span::styled(" SHA-256 Fingerprint ", styles::muted()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let body = if loading {
        vec![
            Line::from(""),
            Line::from(Span::styled("Generating…", styles::status_warn())),
        ]
    } else if fingerprint.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "No cert generated yet. Press g to generate.",
                styles::muted(),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(fingerprint, styles::accent_bold())),
        ]
    };

    frame.render_widget(
        Paragraph::new(body).style(Style::default().bg(palette::BG_SURFACE)),
        inner,
    );
}

/// Render the list of active SANs.
fn render_sans(frame: &mut Frame, sans: &[String], area: Rect) {
    let block =
        styles::panel(false).title(Span::styled(" Subject Alternative Names ", styles::muted()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let body: Vec<Line> = if sans.is_empty() {
        vec![Line::from(Span::styled(
            "  (none — generate cert to populate)",
            styles::muted(),
        ))]
    } else {
        sans.iter()
            .map(|s| Line::from(Span::styled(format!("  • {s}"), styles::body())))
            .collect()
    };

    frame.render_widget(
        Paragraph::new(body).style(Style::default().bg(palette::BG_SURFACE)),
        inner,
    );
}

/// Render the status line.
fn render_status(frame: &mut Frame, loading: bool, status: &str, area: Rect) {
    let style = if status.starts_with("Error") {
        styles::status_err()
    } else if loading {
        styles::status_warn()
    } else {
        styles::status_ok()
    };
    let text = if loading && status.is_empty() {
        "Working…"
    } else {
        status
    };
    frame.render_widget(
        Paragraph::new(Span::styled(text, style)).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Render key-binding hints.
fn render_hints(frame: &mut Frame, area: Rect) {
    let hint = Line::from(vec![
        Span::styled("g", styles::accent()),
        Span::styled(" generate/ensure cert  ", styles::muted()),
        Span::styled("r", styles::accent()),
        Span::styled(" refresh", styles::muted()),
    ]);
    frame.render_widget(Paragraph::new(hint), area);
}
