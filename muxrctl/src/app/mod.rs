//! Application layer (TEA: state / message / update / action).
//!
//! This layer is deliberately free of ratatui and terminal I/O — it is the
//! pure core that [`crate::tui`] drives and renders. The only external UI type
//! it touches is `crossterm::event::KeyEvent` (carried by [`message::Message`]).

pub mod action;
pub mod message;
pub mod state;
pub mod update;

pub use action::UpdateAction;
pub use message::Message;
pub use state::AppState;
#[allow(unused_imports)] // re-exported for later waves / external callers.
pub use state::Screen;
pub use update::update;
