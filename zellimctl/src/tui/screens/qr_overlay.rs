//! QR overlay — renders the fullscreen pairing QR for an already-minted token.
//!
//! ## Phase machine
//!
//! ```text
//! Generating ──(TokenQrReady)──► Showing ──(client count rises)──► Connected
//!             └──(TokenQrFailed)──► Failed
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
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::app::state::{QrOverlay, QrOverlayPhase};
use crate::tui::theme::{palette, styles};
use crate::tui::widgets::qr::QrWidget;

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
    // Wipe the screen drawn underneath FIRST. `Clear` resets each cell's symbol
    // (a plain `Block`/`set_style` only recolors the background and leaves the
    // dashboard's text glyphs in place — they then bleed through the overlay
    // margins and read as a "transparent" background). After clearing, paint a
    // solid opaque base so the whole overlay is one flat colour.
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::BG_BASE)),
        area,
    );

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
            "  Reading cert fingerprint, building QR…",
            styles::muted(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(palette::BG_SURFACE)),
        area,
    );
}

/// Showing phase: render the QR code centered, with the connection metadata as a
/// caption below it.
///
/// The overlay is fullscreen, so the QR gets a full-width region and
/// [`QrWidget`] centers the block horizontally (and vertically) within it; the
/// info caption sits below, center-aligned. On a terminal too small for the
/// matrix, `QrWidget` falls back to printing the raw URI itself.
fn render_showing(
    frame: &mut Frame,
    uri: &str,
    host: &str,
    port: u16,
    fingerprint_short: &str,
    area: Rect,
) {
    let qr = QrWidget::new(uri);

    // QR region (centered, full width) on top; a fixed info caption below.
    let info_rows = 5u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(13), Constraint::Length(info_rows)])
        .split(area);

    qr.render(frame, chunks[0]);
    render_info_panel(frame, host, port, fingerprint_short, chunks[1]);
}

/// Info caption (centered, below the QR): host:port, cert fingerprint, scan prompt.
fn render_info_panel(
    frame: &mut Frame,
    host: &str,
    port: u16,
    fingerprint_short: &str,
    area: Rect,
) {
    let info_lines = vec![
        Line::from(vec![
            Span::styled("Server: ", styles::muted()),
            Span::styled(format!("{host}:{port}"), styles::accent()),
            Span::styled("   Cert: ", styles::muted()),
            Span::styled(fingerprint_short, styles::body()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Scan with the Zelli app to connect…",
            styles::muted(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(info_lines)
            .alignment(Alignment::Center)
            .style(Style::default().bg(palette::BG_BASE)),
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
        Line::from(Span::styled(
            "  QR generation failed:",
            styles::status_err(),
        )),
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
