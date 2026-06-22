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

/// Minimum width of the info panel (right-hand side in side-by-side layout).
const INFO_MIN_WIDTH: u16 = 28;

/// Render the Pair screen.
///
/// Layout (borderless — no `Borders::ALL` cost — so the QR body gets the
/// full height):
///
/// ```text
/// ┌── phase body (Min 3) ──────────────────────────────────────────────┐
/// │  QR / idle / generating / connected / failed                        │
/// ├── combined strip (1) ──────────────────────────────────────────────┤
/// │  ro=on|off  phase  │  p/Enter generate  r regen  Space toggle-ro   │
/// └────────────────────────────────────────────────────────────────────┘
/// ```
///
/// At 80×24 the phase body receives 21 rows (≥ `QrWidget::MIN_HEIGHT` 19).
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    // Borderless — dropping Borders::ALL saves 2 rows, letting the QR body
    // fill the full area height at 80×24 (21 rows ≥ MIN_HEIGHT 19).
    // A header label is rendered inline in the combined strip instead.

    // Vertical layout: phase body / combined status+hints strip.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // phase body
            Constraint::Length(1), // combined status/toggle + hints
        ])
        .split(area);

    render_phase_body(frame, state, rows[0]);
    render_combined_strip(frame, state, rows[1]);
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
///
/// Layout strategy:
/// - **Side-by-side** (preferred): QR on the left (sized to its block width),
///   info panel on the right.  This keeps the info rows from consuming height
///   budget, so a standard 80×24 terminal fits.
/// - **Vertical fallback**: when the available width is too narrow for a
///   side-by-side split (QR block width + `INFO_MIN_WIDTH`), the info is
///   rendered below the QR as before.
fn render_showing(
    frame: &mut Frame,
    uri: &str,
    host: &str,
    port: u16,
    fingerprint_short: &str,
    area: Rect,
) {
    let qr = QrWidget::new(uri);

    // Determine the QR block width so we can decide on layout.
    let qr_block_w = qr.block_width().unwrap_or(41);

    let side_by_side = area.width >= qr_block_w.saturating_add(INFO_MIN_WIDTH);

    if side_by_side {
        // Horizontal split: QR left (fixed cols), info right (remainder).
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(qr_block_w), Constraint::Min(0)])
            .split(area);

        // Give the QR widget the full available height.
        qr.render(frame, cols[0]);
        render_info_panel(frame, host, port, fingerprint_short, cols[1]);
    } else {
        // Vertical fallback: QR above, info below.
        let info_rows = 5u16;
        let qr_rows = area.height.saturating_sub(info_rows).max(13);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(qr_rows), Constraint::Min(0)])
            .split(area);

        qr.render(frame, chunks[0]);
        render_info_panel(frame, host, port, fingerprint_short, chunks[1]);
    }
}

/// Info panel: host:port, certificate fingerprint, scan prompt.
fn render_info_panel(
    frame: &mut Frame,
    host: &str,
    port: u16,
    fingerprint_short: &str,
    area: Rect,
) {
    let info_lines = vec![
        Line::from(""),
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
            "  Scan with the Zelli app",
            styles::muted(),
        )),
        Line::from(Span::styled(
            "  to connect…",
            styles::muted(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(info_lines)
            .alignment(Alignment::Left)
            .style(Style::default().bg(palette::BG_SURFACE)),
        area,
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

/// Combined status-strip + key-binding hints (one row).
///
/// Merging the former two-row status-strip + hints block into a single line
/// reclaims 2 rows for the QR body.  The read-only toggle state and the key
/// hints are all short enough to fit on one 80-column line.
fn render_combined_strip(frame: &mut Frame, state: &AppState, area: Rect) {
    let ro_span = if state.pairing.read_only {
        Span::styled("ro=on", styles::status_warn())
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
        Span::styled(" Pair ", styles::heading()),
        Span::styled("·  ro=", styles::muted()),
        ro_span,
        Span::styled("  phase:", styles::muted()),
        Span::styled(phase_tag, styles::accent()),
        Span::styled("  │  ", styles::muted()),
        Span::styled("p/Enter", styles::accent()),
        Span::styled(" gen  ", styles::muted()),
        Span::styled("r", styles::accent()),
        Span::styled(" regen  ", styles::muted()),
        Span::styled("Space", styles::accent()),
        Span::styled(" toggle ro", styles::muted()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
