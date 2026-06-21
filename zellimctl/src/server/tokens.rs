//! tokens — thin wrappers over `zellij_utils::web_authentication_tokens`.
//!
//! These functions call the same shared token DB that the `zellimserver` CLI
//! uses (`create-token`, `list-tokens`, `revoke-token`).  They are intentionally
//! thin — all heavy logic lives in `zellij_utils`.
//!
//! [`TokenRecord`] is a local, `Clone`-capable mirror of the upstream
//! `zellij_utils::web_authentication_tokens::TokenInfo` (which only derives
//! `Debug`).

use anyhow::{Context, Result};

// ── Local clone-able token record ────────────────────────────────────────────

/// A local, `Clone`-capable record for a stored API token.
///
/// Mirrors `zellij_utils::web_authentication_tokens::TokenInfo` but adds
/// `Clone` so it can be stored in the `AppState` and carried by `Message`.
#[derive(Debug, Clone)]
pub struct TokenRecord {
    /// The display name of the token.
    pub name: String,
    /// Human-readable creation timestamp (as returned by the DB).
    pub created_at: String,
    /// Whether this token grants read-only access.
    pub read_only: bool,
}

impl From<zellij_utils::web_authentication_tokens::TokenInfo> for TokenRecord {
    fn from(t: zellij_utils::web_authentication_tokens::TokenInfo) -> Self {
        Self {
            name: t.name,
            created_at: t.created_at,
            read_only: t.read_only,
        }
    }
}

// ── Token DB operations ───────────────────────────────────────────────────────

/// Create a new API token.
///
/// Returns `(plaintext_token, token_name)`.  The plaintext token is shown once
/// and stored only as a hash — the caller must surface it to the user immediately.
///
/// `name` — optional display name; zellij generates one if `None`.
/// `read_only` — if `true` the token may only call read-only RPCs.
#[allow(dead_code)]
pub fn create(name: Option<String>, read_only: bool) -> Result<(String, String)> {
    zellij_utils::web_authentication_tokens::create_token(name, read_only)
        .context("tokens::create: failed to create token")
}

/// List all tokens stored in the zellij token DB.
///
/// Returns a `Vec<TokenRecord>` — a cloneable local mirror of the upstream type.
#[allow(dead_code)]
pub fn list() -> Result<Vec<TokenRecord>> {
    let raw = zellij_utils::web_authentication_tokens::list_tokens()
        .context("tokens::list: failed to list tokens")?;
    Ok(raw.into_iter().map(TokenRecord::from).collect())
}

/// Revoke a token by name.
///
/// Returns `true` if the token was found and removed, `false` if it didn't exist.
#[allow(dead_code)]
pub fn revoke(name: &str) -> Result<bool> {
    zellij_utils::web_authentication_tokens::revoke_token(name)
        .context("tokens::revoke: failed to revoke token")
}
