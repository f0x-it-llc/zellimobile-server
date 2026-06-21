//! Screen rendering. Dispatches on [`crate::app::state::Screen`].
//!
//! The dashboard module owns the shared chrome (title bar, tab list, footer).
//! Config, Cert, Server, Tokens, and Pair screens render their own body panels.

use ratatui::Frame;

use crate::app::AppState;
use crate::app::state::Screen;

pub mod cert;
pub mod config;
pub mod dashboard;
pub mod pairing;
pub mod server;
pub mod tokens;

/// Top-level render entry point: draws the full frame for the current state.
///
/// The shared chrome (title bar + tab list + footer) always comes from the
/// dashboard module. Each screen then renders its own body into the body area.
pub fn render(frame: &mut Frame, state: &AppState) {
    dashboard::render(frame, state, frame.area());
}

/// Render the body panel for the active screen.
///
/// Called by [`dashboard::render_body`] with the body area after the chrome
/// (tab list column) has been laid out.
pub fn render_screen_body(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    match state.screen {
        Screen::Config => config::render(frame, state, area),
        Screen::Cert => cert::render(frame, state, area),
        Screen::Server => server::render(frame, state, area),
        Screen::Tokens => tokens::render(frame, state, area),
        Screen::Pair => pairing::render(frame, state, area),
        // Dashboard uses a simple placeholder.
        Screen::Dashboard => dashboard::render_placeholder(frame, state, area),
    }
}
