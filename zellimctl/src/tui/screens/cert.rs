//! Cert screen — show TLS certificate fingerprint + SANs; (re)generate with `g`.
//!
//! Keys:
//! - `g`: ensure / regenerate the cert using SANs built from the Config host.
//! - `r`: same as `g` (refresh).
//! - `t`: cycle the advertised trust mode (Auto → CA → Pin → Auto).
//!
//! The fingerprint is displayed prominently when the resolved trust is `pin`
//! (self-signed path). When the trust is `ca` (external cert, h2c, or forced),
//! the panel shows "Trust: public CA — no fingerprint pinned" instead.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::app::state::AdvertiseTrust;
use crate::tui::theme::{palette, styles};

/// Render the Cert screen.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let cert = &state.cert;

    let block = styles::panel(true).title(Span::styled(" Cert ", styles::heading()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Rows: trust / fingerprint panel / SANs list / status / hints.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // trust / fingerprint panel
            Constraint::Min(3),    // SANs panel
            Constraint::Length(1), // status line
            Constraint::Length(2), // key hints
        ])
        .split(inner);

    render_trust_panel(
        frame,
        cert.loading,
        &cert.fingerprint,
        cert.advertise_trust,
        rows[0],
    );
    render_sans(frame, &cert.sans, rows[1]);
    render_status(frame, cert.loading, &cert.status, rows[2]);
    render_hints(frame, cert.advertise_trust, rows[3]);
}

/// Render the trust / fingerprint panel.
///
/// The panel title shows the current `advertise_trust` setting.  The body
/// shows either:
/// - The SHA-256 fingerprint (Pin or Auto with a cert present), or
/// - "Trust: public CA — no fingerprint pinned" (Ca, or Auto without a cert).
///
/// This reflects both the operator-declared override and the CA-vs-pin display
/// described in PLAN.md § "Cert screen".
fn render_trust_panel(
    frame: &mut Frame,
    loading: bool,
    fingerprint: &str,
    advertise_trust: AdvertiseTrust,
    area: Rect,
) {
    // Panel title reflects the active advertise_trust label.
    let title = format!(" Trust: {} ", advertise_trust.label());
    let block = styles::panel(false).title(Span::styled(title, styles::muted()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let body = if loading {
        vec![
            Line::from(""),
            Line::from(Span::styled("Generating…", styles::status_warn())),
        ]
    } else {
        // Determine whether to show fingerprint or CA notice based on the
        // advertise_trust setting.
        //   - Ca (forced): always show "public CA" notice.
        //   - Pin (forced): show fingerprint if present, warn if not.
        //   - Auto: show fingerprint when cert is available (pin path); show
        //     "public CA" notice when no local cert exists (likely h2c / external
        //     cert scenario where no self-signed cert was generated).
        match advertise_trust {
            AdvertiseTrust::Ca => vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Trust: public CA — no fingerprint pinned",
                    styles::accent_bold(),
                )),
            ],
            AdvertiseTrust::Pin => {
                if fingerprint.is_empty() {
                    // Pin forced but no cert on disk yet.
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "No cert yet — press g to generate one first.",
                            styles::status_warn(),
                        )),
                    ]
                } else {
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(fingerprint, styles::accent_bold())),
                    ]
                }
            }
            AdvertiseTrust::Auto => {
                if fingerprint.is_empty() {
                    // Auto + no cert: likely CA-fronted or h2c; no pin needed.
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "Trust: public CA — no fingerprint pinned",
                            styles::accent_bold(),
                        )),
                    ]
                } else {
                    // Auto with a cert: show the fingerprint (pin path).
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(fingerprint, styles::accent_bold())),
                    ]
                }
            }
        }
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
///
/// Shows the standard cert keys plus the `t` toggle for `advertise_trust`
/// with the current value label so the user knows what pressing `t` will cycle to.
fn render_hints(frame: &mut Frame, advertise_trust: AdvertiseTrust, area: Rect) {
    let next_label = advertise_trust.cycle().label();
    let hint = Line::from(vec![
        Span::styled("g", styles::accent()),
        Span::styled(" generate/ensure cert  ", styles::muted()),
        Span::styled("r", styles::accent()),
        Span::styled(" refresh  ", styles::muted()),
        Span::styled("t", styles::accent()),
        Span::styled(format!(" trust ({next_label})"), styles::muted()),
    ]);
    frame.render_widget(Paragraph::new(hint), area);
}
