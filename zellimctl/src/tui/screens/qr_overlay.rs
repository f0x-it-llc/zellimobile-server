//! QR overlay — renders the fullscreen pairing QR for an already-minted token.
//!
//! ## Phase machine
//!
//! ```text
//! Generating ──(QrOverlayReady)──► Showing ──(client count rises)──► Connected
//!             └──(QrOverlayFailed)──► Failed
//! ```
//!
//! This module only knows about [`QrOverlay`] — it never reads `AppState`
//! directly and has no knowledge of `Screen`.  It is invoked as a fullscreen
//! overlay painted over the active screen body by the top-level `render` in
//! [`super`] after `dashboard::render` has drawn the underlying chrome.
//!
//! ## Key bindings (handled by the update layer, shown here for reference)
//! - `Esc`: close the overlay (token is **not** revoked).

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::state::{QrOverlay, QrOverlayPhase};
use crate::tui::theme::{palette, styles};
use crate::tui::widgets::qr::QrWidget;

/// Minimum width of the info panel (right-hand side in side-by-side layout).
const INFO_MIN_WIDTH: u16 = 28;

/// Render the QR overlay fullscreen over `area`.
///
/// Layout (borderless — no `Borders::ALL` cost — so the QR body gets the
/// full height):
///
/// ```text
/// ┌── phase body (Min 3) ──────────────────────────────────────────────┐
/// │  QR / generating / connected / failed                               │
/// ├── bottom strip (1) ─────────────────────────────────────────────────┤
/// │  Esc close · ro=<on|off>                                            │
/// └────────────────────────────────────────────────────────────────────┘
/// ```
pub fn render(frame: &mut Frame, overlay: &QrOverlay, area: Rect) {
    // Vertical layout: phase body / bottom strip.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // phase body
            Constraint::Length(1), // bottom strip (Esc + ro)
        ])
        .split(area);

    render_phase_body(frame, overlay, rows[0]);
    render_bottom_strip(frame, overlay, rows[1]);
}

/// Render the main body depending on the current overlay phase.
fn render_phase_body(frame: &mut Frame, overlay: &QrOverlay, area: Rect) {
    match &overlay.phase {
        QrOverlayPhase::Generating => render_generating(frame, area),
        QrOverlayPhase::Showing {
            uri,
            host,
            port,
            fingerprint_short,
        } => render_showing(frame, uri, host, *port, fingerprint_short, area),
        QrOverlayPhase::Connected => render_connected(frame, area),
        QrOverlayPhase::Failed { err } => render_failed(frame, err, area),
    }
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
            "  Ensuring TLS cert, building QR…",
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
            "  Press Esc to close the overlay.",
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
        Line::from(Span::styled("  QR generation failed:", styles::status_err())),
        Line::from(Span::styled(format!("  {err}"), styles::body())),
        Line::from(""),
        Line::from(Span::styled("  Press Esc to close.", styles::muted())),
    ];
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Bottom strip: `Esc close · ro=<on|off> · <token_name>`.
fn render_bottom_strip(frame: &mut Frame, overlay: &QrOverlay, area: Rect) {
    let ro_span = if overlay.read_only {
        Span::styled("on", styles::status_warn())
    } else {
        Span::styled("off", styles::status_ok())
    };

    let line = Line::from(vec![
        Span::styled(" Esc", styles::accent()),
        Span::styled(" close  ·  ro=", styles::muted()),
        ro_span,
        Span::styled("  ·  ", styles::muted()),
        Span::styled(overlay.token_name.as_str(), styles::accent()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
