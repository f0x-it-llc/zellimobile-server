//! Semantic style builders for the zellimctl theme.
//!
//! Render code calls these instead of constructing `Style` from raw palette
//! constants, so emphasis conventions (BOLD headings, DIM muted text) stay
//! consistent across screens.
//!
//! The full semantic style set is defined up front; `status_*` builders are
//! consumed by later-wave screens, so unused-fn warnings are suppressed
//! module-wide rather than churning this file each wave.
#![allow(dead_code)]

use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

use super::palette;

/// Bold heading in the primary foreground color.
pub fn heading() -> Style {
    Style::default()
        .fg(palette::FG)
        .add_modifier(Modifier::BOLD)
}

/// Muted, dimmed secondary text.
pub fn muted() -> Style {
    Style::default()
        .fg(palette::MUTED)
        .add_modifier(Modifier::DIM)
}

/// Teal accent text (active tab, links, emphasis).
pub fn accent() -> Style {
    Style::default().fg(palette::TEAL)
}

/// Bold teal accent for the active tab / selected emphasis.
pub fn accent_bold() -> Style {
    Style::default()
        .fg(palette::TEAL)
        .add_modifier(Modifier::BOLD)
}

/// Plain body text in the primary foreground.
pub fn body() -> Style {
    Style::default().fg(palette::FG)
}

/// Success status text.
pub fn status_ok() -> Style {
    Style::default().fg(palette::GREEN)
}

/// Warning status text.
pub fn status_warn() -> Style {
    Style::default().fg(palette::YELLOW)
}

/// Error status text.
pub fn status_err() -> Style {
    Style::default().fg(palette::RED)
}

/// Border style for inactive / passive panels.
pub fn panel_border() -> Style {
    Style::default().fg(palette::DIM)
}

/// Border style for the focused / active panel.
pub fn panel_border_active() -> Style {
    Style::default().fg(palette::TEAL)
}

/// A bordered panel block; `active` highlights the border in teal.
pub fn panel(active: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if active {
            panel_border_active()
        } else {
            panel_border()
        })
        .style(Style::default().bg(palette::BG_SURFACE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_is_bold() {
        assert_eq!(heading().fg, Some(palette::FG));
        assert!(heading().add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn muted_is_dim() {
        assert_eq!(muted().fg, Some(palette::MUTED));
        assert!(muted().add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn accent_uses_teal() {
        assert_eq!(accent().fg, Some(palette::TEAL));
        assert_eq!(accent_bold().fg, Some(palette::TEAL));
    }

    #[test]
    fn status_styles_have_colors() {
        assert_eq!(status_ok().fg, Some(palette::GREEN));
        assert_eq!(status_warn().fg, Some(palette::YELLOW));
        assert_eq!(status_err().fg, Some(palette::RED));
    }

    #[test]
    fn panel_border_switches_on_active() {
        assert_eq!(panel_border().fg, Some(palette::DIM));
        assert_eq!(panel_border_active().fg, Some(palette::TEAL));
        // Construction succeeds for both states.
        let _ = panel(true);
        let _ = panel(false);
    }
}
