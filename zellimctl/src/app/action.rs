//! Side-effect actions emitted by [`super::update::update`].
//!
//! `update` is pure: it mutates [`super::state::AppState`] and returns a list
//! of `UpdateAction`s describing side effects to perform. The runner dispatches
//! each action — most spawn a `tokio::task::spawn_blocking` task that posts
//! results back as a [`super::message::Message`] over a cloned `mpsc::Sender`.
//!
//! All spawning lives in `tui/runner.rs`; `app/` stays free of async code.

use super::state::San;

/// A side effect for the runner to perform after an update cycle.
#[derive(Debug, Clone)]
pub enum UpdateAction {
    /// Break the event loop and restore the terminal. Handled directly by the
    /// runner via `AppState.should_quit`; carried here so the action surface is
    /// in place for all async variants.
    Quit,

    /// Poll the server for its current status (non-blocking path: spawns a
    /// blocking task and posts `Message::StatusLoaded` or `ActionFailed`).
    RefreshStatus,

    /// Launch the `zellimserver` daemon (spawns via [`crate::server::start_daemon`]).
    StartServer,

    /// Send a shutdown request to the running daemon.
    StopServer,

    /// Load the effective server config + reachable IPs and post `Message::ConfigLoaded`.
    LoadConfig,

    /// Persist `bind_addr` to the config file via [`crate::server::set_bind_addr`].
    SaveBind(String),

    /// Ensure / regenerate the TLS cert for the given SANs and post `Message::CertEnsured`.
    EnsureCert(Vec<San>),

    // ── Token management actions ──────────────────────────────────────────────
    /// List all tokens and post `Message::TokensLoaded`.
    LoadTokens,

    /// Create a new token (with optional name and read-only flag).
    ///
    /// Posts `Message::TokenCreated { token, name }` on success, then
    /// `Message::TokensChanged` to trigger a refresh.
    CreateToken {
        name: Option<String>,
        read_only: bool,
    },

    /// Revoke the token with the given name.
    ///
    /// Posts `Message::TokensChanged` on completion.
    RevokeToken(String),

    // ── Pairing actions ───────────────────────────────────────────────────────
    /// Begin the pairing QR flow: gather config, ensure cert, mint token, build URI.
    ///
    /// Posts `Message::PairingReady { uri, baseline_clients }` on success or
    /// `Message::PairingFailed(err)` on error. Carries the current `seq` so
    /// stale responses are discarded.
    StartPairing {
        /// Read-only toggle for the freshly minted pairing token.
        read_only: bool,
        /// The sequence number of this attempt (seq-guard against stale results).
        seq: u64,
    },

    /// Silently revoke a previously-minted pairing token (best-effort).
    ///
    /// Emitted when a pairing QR is superseded (regenerate) or the Pair screen
    /// is left with an unused token outstanding. Unlike [`Self::RevokeToken`],
    /// it does not refresh the Tokens list or surface status — it just tidies up
    /// the bearer secret. The result is ignored.
    RevokePairingToken(String),
}
