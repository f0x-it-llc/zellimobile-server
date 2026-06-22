//! Dashboard placeholder.
//!
//! A bordered full-screen layout: a title bar, a left tab list of the six
//! screens (active = teal), a body panel showing "<Screen> — coming soon", and
//! a footer hint. Later waves replace the body per-screen; the chrome
//! (title / tabs / footer) stays.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::AppState;
use crate::app::state::Screen;
use crate::tui::theme::{palette, styles};

/// Render the dashboard chrome and the placeholder body for the active screen.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    // Paint the base background across the whole area first.
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::BG_BASE)),
        area,
    );

    // Vertical split: title bar / body / footer hint.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    render_title(frame, rows[0]);
    render_body(frame, state, rows[1]);
    render_footer(frame, rows[2]);
}

/// Top title bar.
fn render_title(frame: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled("zellimctl", styles::accent_bold()),
        Span::styled("  ·  zellimserver control panel", styles::muted()),
    ]);
    frame.render_widget(
        Paragraph::new(title).style(Style::default().bg(palette::BG_BASE)),
        area,
    );
}

/// Body: a left tab column + the active-screen body panel.
///
/// When the Pair screen is active the tab column is suppressed so the QR +
/// info layout has the full terminal width.  A one-line breadcrumb is shown
/// at the top of the body area instead.
fn render_body(frame: &mut Frame, state: &AppState, area: Rect) {
    if state.screen == Screen::Pair {
        // Full-width body for the Pair screen: show a minimal breadcrumb then
        // delegate to the pairing renderer for the rest.
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let breadcrumb = ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Screens", crate::tui::theme::styles::muted()),
            ratatui::text::Span::styled(" › ", crate::tui::theme::styles::muted()),
            ratatui::text::Span::styled("Pair", crate::tui::theme::styles::accent_bold()),
        ]);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(breadcrumb)
                .style(Style::default().bg(palette::BG_BASE)),
            rows[0],
        );

        crate::tui::screens::render_screen_body(frame, state, rows[1]);
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(18), Constraint::Min(20)])
            .split(area);

        render_tabs(frame, state, cols[0]);
        // Delegate to the screens module which dispatches per-screen renders.
        crate::tui::screens::render_screen_body(frame, state, cols[1]);
    }
}

/// Left tab list. The active screen is teal+bold; others are muted.
fn render_tabs(frame: &mut Frame, state: &AppState, area: Rect) {
    let lines: Vec<Line> = Screen::ALL
        .iter()
        .map(|screen| {
            let active = *screen == state.screen;
            let marker = if active { "▶ " } else { "  " };
            let style = if active {
                styles::accent_bold()
            } else {
                styles::muted()
            };
            Line::from(Span::styled(format!("{marker}{}", screen.label()), style))
        })
        .collect();

    let block = styles::panel(false).title(Span::styled(" Screens ", styles::heading()));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Body panel for the active screen — placeholder for screens without a dedicated renderer.
pub fn render_placeholder(frame: &mut Frame, state: &AppState, area: Rect) {
    let title = format!(" {} ", state.screen.label());
    let block = styles::panel(true).title(Span::styled(title, styles::heading()));

    let body = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{} — coming soon", state.screen.label()),
            styles::body(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This screen will be implemented in a later wave.",
            styles::muted(),
        )),
    ];

    frame.render_widget(Paragraph::new(body).block(block), area);
}

/// Bottom footer with navigation hints.
fn render_footer(frame: &mut Frame, area: Rect) {
    let hint = Line::from(vec![
        Span::styled("Tab", styles::accent()),
        Span::styled("/", styles::muted()),
        Span::styled("←→", styles::accent()),
        Span::styled(" switch  ·  ", styles::muted()),
        Span::styled("q", styles::accent()),
        Span::styled("/", styles::muted()),
        Span::styled("Ctrl-C", styles::accent()),
        Span::styled(" quit", styles::muted()),
    ]);
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().bg(palette::BG_BASE)),
        area,
    );
}
