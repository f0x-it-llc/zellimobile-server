//! Terminal setup, restoration, and the panic hook.
//!
//! Mirrors fdemon's `terminal.rs` minus mouse capture (the skeleton does not
//! capture the mouse). The key invariant: install the panic hook AFTER
//! `ratatui::init()` so our hook wraps ratatui's and `ratatui::restore()` runs
//! on panic — the terminal is never left in raw mode.

use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::DefaultTerminal;

/// Guards against double-installation of the panic hook. Installing twice would
/// chain duplicate `restore()` closures.
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Enter the alternate screen / raw mode and return the terminal handle.
///
/// Returns ratatui's [`DefaultTerminal`]. Callers must pair this with
/// [`restore`] (or rely on the panic hook installed by [`install_panic_hook`]).
pub fn init() -> DefaultTerminal {
    ratatui::init()
}

/// Leave the alternate screen and restore the cooked terminal mode on a clean
/// exit. Best-effort and idempotent (`ratatui::restore` tolerates repeat calls).
pub fn restore() {
    ratatui::restore();
}

/// Install a panic hook that restores the terminal before the panic propagates.
///
/// Wraps the existing hook so any pre-existing hook still runs after cleanup.
/// Idempotent: only the first call per process installs the hook.
///
/// MUST be called after [`init`] (`ratatui::init()`): both use the "take + wrap"
/// `set_hook` pattern, and whichever installs last wraps the other. Installing
/// ours last guarantees `restore()` runs first on panic, before any outer hook
/// writes to the (now-restored) primary screen.
pub fn install_panic_hook() {
    if HOOK_INSTALLED.swap(true, Ordering::AcqRel) {
        return;
    }

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Best-effort cleanup — we are already panicking.
        ratatui::restore();
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_panic_hook_is_idempotent() {
        HOOK_INSTALLED.store(false, Ordering::Relaxed);

        install_panic_hook();
        assert!(HOOK_INSTALLED.load(Ordering::Acquire));

        // Second call must be a no-op (flag already set).
        install_panic_hook();
        assert!(HOOK_INSTALLED.load(Ordering::Acquire));

        // Restore a benign hook so we don't leave a wrapped hook for other tests.
        let _ = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
    }
}
