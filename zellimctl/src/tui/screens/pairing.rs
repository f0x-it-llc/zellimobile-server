//! Pairing screen — generates a QR code for scanning from the mobile app.
//!
//! ## Phase machine
//!
//! ```text
//! Idle ──(p/Enter)──► Generating ──(PairingReady)──► Showing ──(client count rises)──► Connected
//!                                  └──(PairingFailed)──► Failed
//!
//! From any phase: r / p / g regenerates (bumps seq, goes back to Generating).
//! ```
//!
//! ## Key bindings
//! - `p`/`Enter`/`g`: start or regenerate pairing.
//! - `r`: regenerate (same as `p`).
//! - `Space`: toggle read-only for the next generated token.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::app::state::PairingPhase;
use crate::tui::theme::{palette, styles};
use crate::tui::widgets::qr::QrWidget;

/// Render the Pair screen.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = styles::panel(true).title(Span::styled(" Pair ", styles::heading()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Vertical layout: body / status strip / hints.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // phase body
            Constraint::Length(1), // status/toggle strip
            Constraint::Length(2), // hints
        ])
        .split(inner);

    render_phase_body(frame, state, rows[0]);
    render_status_strip(frame, state, rows[1]);
    render_hints(frame, rows[2]);
}

/// Render the main body depending on the current phase.
fn render_phase_body(frame: &mut Frame, state: &AppState, area: Rect) {
    match &state.pairing.phase {
        PairingPhase::Idle => render_idle(frame, area),
        PairingPhase::Generating => render_generating(frame, area),
        PairingPhase::Showing {
            uri,
            host,
            port,
            fingerprint_short,
            ..
        } => render_showing(frame, uri, host, *port, fingerprint_short, area),
        PairingPhase::Connected => render_connected(frame, area),
        PairingPhase::Failed { err } => render_failed(frame, err, area),
    }
}

/// Idle phase: prompt the user to press p.
fn render_idle(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Press p or Enter to generate a pairing QR code.",
            styles::muted(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  The code encodes a one-time token and the server address.",
            styles::muted(),
        )),
        Line::from(Span::styled(
            "  Scan it from the Zelli mobile app to connect automatically.",
            styles::muted(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Generating phase: show a spinner/progress message.
fn render_generating(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Generating pairing code…",
            styles::status_warn(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Ensuring TLS cert, minting token, building QR…",
            styles::muted(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Showing phase: render the QR code and connection metadata.
fn render_showing(
    frame: &mut Frame,
    uri: &str,
    host: &str,
    port: u16,
    fingerprint_short: &str,
    area: Rect,
) {
    // Split: QR block (most of the height) + info text below.
    // Reserve 4 rows for info (host:port, fingerprint, scanning prompt).
    let info_rows = 5u16;
    let qr_rows = area.height.saturating_sub(info_rows).max(13);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(qr_rows), Constraint::Min(0)])
        .split(area);

    // Render the QR widget.
    QrWidget::new(uri).render(frame, chunks[0]);

    // Render the info strip below the QR.
    let info_lines = vec![
        Line::from(vec![
            Span::styled("  Server: ", styles::muted()),
            Span::styled(format!("{host}:{port}"), styles::accent()),
        ]),
        Line::from(vec![
            Span::styled("  Cert:   ", styles::muted()),
            Span::styled(fingerprint_short, styles::body()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Scan with the Zelli app to connect…",
            styles::muted(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(info_lines).style(Style::default().bg(palette::BG_SURFACE)),
        chunks[1],
    );
}

/// Connected phase: show a heuristic "a client connected" message.
///
/// NOTE: this is inferred from a rise in the attached-client count, not from
/// verified per-token authentication — the copy is deliberately honest about it.
fn render_connected(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ✓ A client connected (attached-client count rose).",
            Style::default().fg(palette::TEAL),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Verify it's your phone, then continue.",
            styles::muted(),
        )),
        Line::from(Span::styled(
            "  Press r to generate a new pairing code.",
            styles::muted(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Left)
            .style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Failed phase: show the error.
fn render_failed(frame: &mut Frame, err: &str, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Pairing failed:", styles::status_err())),
        Line::from(Span::styled(format!("  {err}"), styles::body())),
        Line::from(""),
        Line::from(Span::styled("  Press r to retry.", styles::muted())),
    ];
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Status strip: shows the current read-only toggle + phase tag.
fn render_status_strip(frame: &mut Frame, state: &AppState, area: Rect) {
    let ro_span = if state.pairing.read_only {
        Span::styled("ro=on ", styles::status_warn())
    } else {
        Span::styled("ro=off", styles::status_ok())
    };

    let phase_tag = match &state.pairing.phase {
        PairingPhase::Idle => "idle",
        PairingPhase::Generating => "generating",
        PairingPhase::Showing { .. } => "showing",
        PairingPhase::Connected => "connected",
        PairingPhase::Failed { .. } => "failed",
    };

    let line = Line::from(vec![
        Span::styled("  Token: ", styles::muted()),
        ro_span,
        Span::styled("  [Space toggle]   phase: ", styles::muted()),
        Span::styled(phase_tag, styles::accent()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

/// Key-binding hints.
fn render_hints(frame: &mut Frame, area: Rect) {
    let hint = Line::from(vec![
        Span::styled("p/Enter", styles::accent()),
        Span::styled(" generate  ", styles::muted()),
        Span::styled("r", styles::accent()),
        Span::styled(" regenerate  ", styles::muted()),
        Span::styled("Space", styles::accent()),
        Span::styled(" toggle read-only", styles::muted()),
    ]);
    frame.render_widget(Paragraph::new(hint), area);
}
