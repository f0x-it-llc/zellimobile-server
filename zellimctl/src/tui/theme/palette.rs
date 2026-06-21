//! Design tokens for the zellimctl theme.
//!
//! Dark, teal-accent, JetBrains-Mono aesthetic mapped from `mobile-design.html`.
//! All values are true-color `Color::Rgb`. On terminals without true-color,
//! ratatui/crossterm auto-fall back to the nearest 256-color match.
//!
//! These are the single source of truth for color; render code imports from
//! [`super`] (theme) and never hard-codes RGB values.
//!
//! The full PLAN token table is defined here up front; not every token is
//! consumed by the skeleton's screens yet, so unused-const warnings are
//! suppressed module-wide rather than churning this file each wave.
#![allow(dead_code)]

use ratatui::style::Color;

// --- Background layers ---
/// App background (`#0D1117`).
pub const BG_BASE: Color = Color::Rgb(0x0D, 0x11, 0x17);
/// Panels / cards (`#131B24`).
pub const BG_SURFACE: Color = Color::Rgb(0x13, 0x1B, 0x24);
/// Inputs, selected row (`#1A2332`).
pub const BG_RAISED: Color = Color::Rgb(0x1A, 0x23, 0x32);
/// Hover (`#243040`).
pub const BG_HOVER: Color = Color::Rgb(0x24, 0x30, 0x40);
/// Modal scrim / overlay (`#1F2D3D`).
pub const BG_OVERLAY: Color = Color::Rgb(0x1F, 0x2D, 0x3D);
/// Text on teal buttons (`#080D13`).
pub const BG_DEEP: Color = Color::Rgb(0x08, 0x0D, 0x13);

// --- Accent ---
/// Primary accent / active (`#0ABDA0`).
pub const TEAL: Color = Color::Rgb(0x0A, 0xBD, 0xA0);
/// Dimmed accent (`#0A8F78`).
pub const TEAL_DIM: Color = Color::Rgb(0x0A, 0x8F, 0x78);
/// Secondary accent (`#6272A4`).
pub const PURPLE: Color = Color::Rgb(0x62, 0x72, 0xA4);

// --- Status ---
/// Success (`#50FA7B`).
pub const GREEN: Color = Color::Rgb(0x50, 0xFA, 0x7B);
/// Warning (`#F1FA8C`).
pub const YELLOW: Color = Color::Rgb(0xF1, 0xFA, 0x8C);
/// Notice (`#FFB86C`).
pub const ORANGE: Color = Color::Rgb(0xFF, 0xB8, 0x6C);
/// Error (`#FF5555`).
pub const RED: Color = Color::Rgb(0xFF, 0x55, 0x55);
/// Highlight / pink (`#FF79C6`).
pub const PINK: Color = Color::Rgb(0xFF, 0x79, 0xC6);

// --- Text tiers ---
/// Primary foreground (`#E8EDF2`).
pub const FG: Color = Color::Rgb(0xE8, 0xED, 0xF2);
/// Muted text (`#8899AA`).
pub const MUTED: Color = Color::Rgb(0x88, 0x99, 0xAA);
/// Dim text / inactive border (`#4A5568`).
pub const DIM: Color = Color::Rgb(0x4A, 0x55, 0x68);
/// Faintest decoration (`#2D3748`).
pub const FAINTEST: Color = Color::Rgb(0x2D, 0x37, 0x48);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_rgb() {
        assert_eq!(BG_BASE, Color::Rgb(13, 17, 23));
        assert_eq!(TEAL, Color::Rgb(10, 189, 160));
        assert_eq!(FG, Color::Rgb(232, 237, 242));
    }

    #[test]
    fn status_tokens_defined() {
        let _: Color = GREEN;
        let _: Color = YELLOW;
        let _: Color = ORANGE;
        let _: Color = RED;
        let _: Color = PINK;
    }
}
