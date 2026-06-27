//! Resolution of herdr's two Unix-domain socket paths.
//!
//! herdr exposes two sockets next to each other in its data directory:
//!
//! - the **JSON-API control socket** (`herdr.sock`) — used by [`super::control`]
//!   (P2.02) for workspace/tab/pane/layout operations;
//! - the **binary wire relay socket** (`herdr-client.sock`) — used by the wire
//!   relay (P2.03) for terminal attach.
//!
//! Both are derived here so the two consumers agree on a single resolution rule.
//! The rule mirrors herdr's own public contract (verified against herdr
//! v0.7.1 `session::active_api_socket_path` + `server::socket_paths`):
//!
//! 1. If `HERDR_SOCKET_PATH` is set, it **is** the JSON-API socket path; the wire
//!    socket is derived from it by inserting `-client` before the `.sock`
//!    extension (`…/herdr.sock` → `…/herdr-client.sock`).
//! 2. Otherwise the default JSON-API socket is `<config-dir>/herdr.sock`, where
//!    `<config-dir>` is `$XDG_CONFIG_HOME/herdr` or `$HOME/.config/herdr`.
//!
//! We only model the **default (unnamed) session** here — named-session
//! (`--session`) selection is a herdr-CLI concern that muxrd does not drive; the
//! operator points muxrd at a specific instance via `HERDR_SOCKET_PATH`.

use std::path::{Path, PathBuf};

/// Environment variable that overrides the herdr JSON-API socket path.
/// Matches herdr's `api::SOCKET_PATH_ENV_VAR`.
pub const HERDR_SOCKET_PATH_ENV: &str = "HERDR_SOCKET_PATH";

/// The resolved pair of herdr socket paths for one herdr instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HerdrSocketPaths {
    /// Line-delimited JSON-API control socket (P2.02).
    pub api: PathBuf,
    /// Binary wire terminal-relay socket (P2.03).
    pub wire: PathBuf,
}

impl HerdrSocketPaths {
    /// Resolve both socket paths from the process environment.
    pub fn resolve() -> Self {
        let api = resolve_api_socket_path();
        let wire = derive_wire_socket_path(&api);
        Self { api, wire }
    }

    /// Build the pair from an explicit JSON-API socket path (testing / explicit
    /// configuration). The wire path is derived the same way as [`Self::resolve`].
    pub fn from_api_socket(api: impl Into<PathBuf>) -> Self {
        let api = api.into();
        let wire = derive_wire_socket_path(&api);
        Self { api, wire }
    }
}

/// Resolve the herdr JSON-API socket path from `HERDR_SOCKET_PATH`, falling back
/// to `<config-dir>/herdr.sock`.
pub fn resolve_api_socket_path() -> PathBuf {
    let env_override = std::env::var(HERDR_SOCKET_PATH_ENV).ok();
    resolve_api_socket_path_inner(env_override.as_deref(), &default_config_dir())
}

/// Pure core of [`resolve_api_socket_path`], split out for unit testing without
/// mutating process-global environment.
fn resolve_api_socket_path_inner(env_override: Option<&str>, config_dir: &Path) -> PathBuf {
    match env_override {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => config_dir.join("herdr.sock"),
    }
}

/// Derive the wire (client) socket path from a JSON-API socket path by inserting
/// `-client` before the extension: `…/herdr.sock` → `…/herdr-client.sock`.
///
/// Mirrors herdr's `derive_client_socket_from_api_socket`: the wire socket always
/// lives beside the API socket and shares its stem with a `-client.sock` suffix,
/// regardless of the API socket's original extension.
pub fn derive_wire_socket_path(api: &Path) -> PathBuf {
    let stem = api.file_stem().and_then(|s| s.to_str()).unwrap_or("herdr");
    let parent = api.parent().unwrap_or_else(|| Path::new(""));
    parent.join(format!("{stem}-client.sock"))
}

/// herdr's default config directory for the **release** binary (`herdr`, not the
/// `herdr-dev` debug variant): `$XDG_CONFIG_HOME/herdr` or `$HOME/.config/herdr`.
fn default_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME")
        && !dir.is_empty()
    {
        return PathBuf::from(dir).join("herdr");
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        return PathBuf::from(home).join(".config").join("herdr");
    }
    std::env::temp_dir().join("herdr")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_socket_uses_env_override_when_set() {
        let path = resolve_api_socket_path_inner(
            Some("/run/user/1000/herdr-custom.sock"),
            Path::new("/home/u/.config/herdr"),
        );
        assert_eq!(path, PathBuf::from("/run/user/1000/herdr-custom.sock"));
    }

    #[test]
    fn api_socket_falls_back_to_config_dir_default() {
        let path = resolve_api_socket_path_inner(None, Path::new("/home/u/.config/herdr"));
        assert_eq!(path, PathBuf::from("/home/u/.config/herdr/herdr.sock"));
    }

    #[test]
    fn empty_env_override_is_ignored() {
        let path = resolve_api_socket_path_inner(Some(""), Path::new("/home/u/.config/herdr"));
        assert_eq!(path, PathBuf::from("/home/u/.config/herdr/herdr.sock"));
    }

    #[test]
    fn wire_socket_derived_from_dot_sock_api_path() {
        let wire = derive_wire_socket_path(Path::new("/home/u/.config/herdr/herdr.sock"));
        assert_eq!(
            wire,
            PathBuf::from("/home/u/.config/herdr/herdr-client.sock")
        );
    }

    #[test]
    fn wire_socket_derived_from_custom_override_path() {
        let wire = derive_wire_socket_path(Path::new("/run/user/1000/herdr-custom.sock"));
        assert_eq!(
            wire,
            PathBuf::from("/run/user/1000/herdr-custom-client.sock")
        );
    }

    #[test]
    fn wire_socket_derived_when_api_has_no_sock_extension() {
        // herdr appends `-client.sock` to the stem regardless of original ext.
        let wire = derive_wire_socket_path(Path::new("/tmp/custom-api"));
        assert_eq!(wire, PathBuf::from("/tmp/custom-api-client.sock"));
    }

    #[test]
    fn socket_pair_from_api_socket_derives_wire_sibling() {
        let pair = HerdrSocketPaths::from_api_socket("/srv/herdr/herdr.sock");
        assert_eq!(pair.api, PathBuf::from("/srv/herdr/herdr.sock"));
        assert_eq!(pair.wire, PathBuf::from("/srv/herdr/herdr-client.sock"));
    }
}
