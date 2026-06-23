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

    // ── Token QR overlay ──────────────────────────────────────────────────────
    /// Build a pairing QR URI from an **existing** plaintext token (no mint, no
    /// revoke).
    ///
    /// Posts `Message::TokenQrReady { uri, .. }` on success or
    /// `Message::TokenQrFailed { err, .. }` on error. Carries the current overlay
    /// `seq` so stale responses (overlay since closed) are discarded. The token
    /// is the real user token the QR encodes — it is NEVER revoked here.
    ShowTokenQr {
        /// The plaintext token to encode into the pairing URI.
        token: String,
        /// Whether the token grants read-only access (embedded as `ro` in the URI).
        read_only: bool,
        /// The sequence number of this overlay (seq-guard against stale results).
        seq: u64,
    },

    // ── Dashboard ─────────────────────────────────────────────────────────────
    /// Read the persisted cert fingerprint + SAN sidecar **without** generating
    /// anything; posts `Message::CertInfoLoaded`.
    ///
    /// Never calls `ensure_cert` / `load_or_generate_identity`.  The Dashboard
    /// overview uses this for a read-only cert summary.
    LoadCertInfo,
}
