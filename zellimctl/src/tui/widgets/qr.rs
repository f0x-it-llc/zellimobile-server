//! QR code widget — centered, black-on-white, with a manual quiet-zone border.
//!
//! Wraps `tui_qrcode::{QrCodeWidget, QuietZone}` with an explicit white
//! background and manual quiet-zone padding so phone scanners accept the code.
//!
//! If the available area is too small to hold the matrix, a "terminal too small"
//! fallback message is rendered instead.
//!
//! # Usage
//!
//! ```ignore
//! frame.render_widget(QrWidget::new("https://example.com"), area);
//! ```

use qrcode::QrCode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use tui_qrcode::{QrCodeWidget, QuietZone};

/// Horizontal quiet-zone padding in terminal columns (≈ 4 modules).
const QUIET_H: u16 = 4;
/// Vertical quiet-zone padding in terminal rows (≥ 1 row each side).
const QUIET_V: u16 = 1;

/// Minimum area dimensions required to render the QR matrix at all.
///
/// A version-1 QR code has 21 modules; at 1 char per module the matrix
/// occupies 21×11 cells (half-block Dense1x2 halves the row count).
/// With horizontal quiet-zone padding and 1-row top/bottom padding the
/// minimum useful area is (21 + 2*4) × (11 + 2*1) = 29 × 13.
const MIN_WIDTH: u16 = 29;
const MIN_HEIGHT: u16 = 13;

/// A stateless QR-code widget.
///
/// Renders a `zellimobile://pair?...` (or any) payload as a scannable
/// half-block QR matrix, centered with a white quiet-zone border.
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

    /// Draw the widget into `area` of `frame`.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            render_too_small(frame, area);
            return;
        }

        // Build the QR code.
        let qr = match QrCode::new(self.payload.as_bytes()) {
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
            render_too_small(frame, area);
            return;
        }

        // Center the block within the available area.
        let pad_left = (area.width.saturating_sub(block_w)) / 2;
        let pad_top = (area.height.saturating_sub(block_h)) / 2;

        // White background for the entire block (quiet zone + matrix).
        let block_rect = Rect::new(area.x + pad_left, area.y + pad_top, block_w, block_h);
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::White)),
            block_rect,
        );

        // Matrix rect is inset by the quiet zone padding.
        let matrix_rect = Rect::new(
            block_rect.x + QUIET_H,
            block_rect.y + QUIET_V,
            matrix_w.min(block_rect.width.saturating_sub(QUIET_H * 2)),
            matrix_h.min(block_rect.height.saturating_sub(QUIET_V * 2)),
        );

        frame.render_widget(qr_widget, matrix_rect);
    }
}

/// Render a "terminal too small" fallback message.
fn render_too_small(frame: &mut Frame, area: Rect) {
    let msg = vec![
        Line::from(""),
        Line::from(Span::styled(
            "[ terminal too small — enlarge to scan ]",
            Style::default().fg(Color::Yellow),
        )),
    ];
    frame.render_widget(
        Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow)),
        area,
    );
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
        // MIN_WIDTH >= 21 (QR) + 2*4 (quiet) = 29.
        assert!(MIN_WIDTH >= 21 + QUIET_H * 2);
        // MIN_HEIGHT >= 11 (QR) + 2*1 (quiet) = 13.
        assert!(MIN_HEIGHT >= 11 + QUIET_V * 2);
    }

    #[test]
    fn quiet_zone_constants_match_spec() {
        // QUIET_H ≈ 4 horizontal cells of white space on each side.
        assert_eq!(QUIET_H, 4);
        // QUIET_V ≥ 1 row of white space on each side.
        assert!(QUIET_V >= 1);
    }
}
