//! QR code widget — centered, black-on-white, with a manual quiet-zone border.
//!
//! Wraps `tui_qrcode::{QrCodeWidget, QuietZone}` with an explicit white
//! background and manual quiet-zone padding so phone scanners accept the code.
//!
//! If the available area is too small to hold the matrix, a fallback is rendered
//! that shows the required minimum dimensions and the raw `zellimobile://pair?…`
//! URI so the user can copy/paste it even on a tiny terminal.
//!
//! # Usage
//!
//! ```ignore
//! frame.render_widget(QrWidget::new("https://example.com"), area);
//! ```

use qrcode::{EcLevel, QrCode};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use tui_qrcode::{QrCodeWidget, QuietZone};

/// Horizontal quiet-zone padding in terminal columns (≈ 4 modules).
const QUIET_H: u16 = 4;
/// Vertical quiet-zone padding in terminal rows (≥ 1 row each side).
const QUIET_V: u16 = 1;

/// Minimum area dimensions required to render the QR matrix at all.
///
/// The `zellimobile://pair?…` URI is ≈ 170 bytes.  We use ECC-L (error
/// correction level Low) because the code is displayed on-screen and scanned
/// immediately — there is no physical damage risk that warrants higher
/// redundancy.  ECC-L keeps a ~170-byte payload at **version 8 = 33×33
/// modules** instead of version 9 (37×37) at the default ECC-M.
///
/// Dense1x2 renders 1 col per module and 2 modules per character row, so the
/// 33-module matrix occupies 33 cols × 17 rows (ceil(33/2)).
/// With horizontal quiet-zone padding and 1-row top/bottom padding the
/// minimum useful area is (33 + 2*4) × (17 + 2*1) = 41 × 19.
const MIN_WIDTH: u16 = 41;
const MIN_HEIGHT: u16 = 19;

/// A stateless QR-code widget.
///
/// Renders a `zellimobile://pair?...` (or any) payload as a scannable
/// half-block QR matrix, centered with a white quiet-zone border.
///
/// The widget stores the payload so it can also render it as plain text in the
/// fallback path when the terminal is too small.
pub struct QrWidget {
    payload: String,
}

impl QrWidget {
    /// Create a new QR widget for the given payload.
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }

    /// Return the computed block width (cols) this widget will occupy.
    ///
    /// Returns `None` when the payload cannot be encoded (should never happen
    /// for well-formed `zellimobile://` URIs).
    pub fn block_width(&self) -> Option<u16> {
        let qr = QrCode::with_error_correction_level(self.payload.as_bytes(), EcLevel::L).ok()?;
        let qr_widget = QrCodeWidget::new(qr)
            .quiet_zone(QuietZone::Disabled)
            .style(Style::default().fg(Color::Black).bg(Color::White));
        // Use a dummy large rect just to obtain the matrix size.
        let dummy = Rect::new(0, 0, 200, 100);
        let sz = qr_widget.size(dummy);
        Some(sz.width.saturating_add(QUIET_H * 2))
    }

    /// Draw the widget into `area` of `frame`.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            self.render_too_small(frame, area);
            return;
        }

        // Build the QR code with ECC-L to keep the 33-module (version 8) matrix.
        // ECC-L is appropriate here: the code is on-screen and immediately scanned.
        let qr = match QrCode::with_error_correction_level(self.payload.as_bytes(), EcLevel::L) {
            Ok(q) => q,
            Err(_) => {
                render_error(frame, area, "QR encode failed");
                return;
            }
        };

        // The tui-qrcode widget with quiet zone disabled; we add our own padding.
        let qr_widget = QrCodeWidget::new(qr)
            .quiet_zone(QuietZone::Disabled)
            .style(Style::default().fg(Color::Black).bg(Color::White));

        // Compute the pixel size of the matrix so we can center it.
        let matrix_size = qr_widget.size(area);
        let matrix_w = matrix_size.width;
        let matrix_h = matrix_size.height;

        // Total block including our manual quiet zone.
        let block_w = matrix_w.saturating_add(QUIET_H * 2);
        let block_h = matrix_h.saturating_add(QUIET_V * 2);

        if block_w > area.width || block_h > area.height {
            self.render_too_small(frame, area);
            return;
        }

        // Center the block within the available area.
        let pad_left = (area.width.saturating_sub(block_w)) / 2;
        let pad_top = (area.height.saturating_sub(block_h)) / 2;

        // White background for the entire block (quiet zone + matrix).
        let block_rect = Rect::new(area.x + pad_left, area.y + pad_top, block_w, block_h);
        frame.render_widget(
            ratatui::widgets::Block::default().style(Style::default().bg(Color::White)),
            block_rect,
        );

        // Matrix rect is inset by the quiet zone padding.
        let matrix_rect = ratatui::layout::Rect::new(
            block_rect.x + QUIET_H,
            block_rect.y + QUIET_V,
            matrix_w.min(block_rect.width.saturating_sub(QUIET_H * 2)),
            matrix_h.min(block_rect.height.saturating_sub(QUIET_V * 2)),
        );

        frame.render_widget(qr_widget, matrix_rect);
    }

    /// Render the "terminal too small" fallback: show required dimensions and
    /// the raw URI so users on small terminals can pair by copy/paste.
    fn render_too_small(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!(
                    "[ terminal too small — need {}×{} cols×rows ]",
                    MIN_WIDTH, MIN_HEIGHT
                ),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Pair via copy/paste — URI:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ];
        // Append the raw URI so the user can select and copy it.
        lines.push(Line::from(Span::styled(
            self.payload.clone(),
            Style::default().fg(Color::Cyan),
        )));
        frame.render_widget(
            Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::Yellow)),
            area,
        );
    }
}

/// Render a generic error fallback.
fn render_error(frame: &mut Frame, area: Rect, msg: &str) {
    frame.render_widget(
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::Red)))
            .alignment(Alignment::Center),
        area,
    );
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_widget_new_stores_payload() {
        let w = QrWidget::new("hello");
        assert_eq!(w.payload, "hello");
    }

    #[test]
    fn min_dimensions_are_sane() {
        // Version-8 QR at ECC-L = 33 modules; Dense1x2 = 33 cols × 17 rows.
        // MIN_WIDTH  >= 33 (matrix cols) + 2*4 (quiet) = 41.
        assert!(MIN_WIDTH >= 33 + QUIET_H * 2);
        // MIN_HEIGHT >= 17 (matrix rows) + 2*1 (quiet) = 19.
        assert!(MIN_HEIGHT >= 17 + QUIET_V * 2);
    }

    #[test]
    fn quiet_zone_constants_match_spec() {
        // QUIET_H ≈ 4 horizontal cells of white space on each side.
        assert_eq!(QUIET_H, 4);
        // QUIET_V ≥ 1 row of white space on each side.
        assert!(QUIET_V >= 1);
    }
}
