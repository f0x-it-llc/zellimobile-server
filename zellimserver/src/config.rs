//! config — server configuration with precedence resolution.
//!
//! ## Precedence (highest → lowest)
//!
//! 1. CLI flags (`--bind`)
//! 2. Environment variables (`ZELLIMSERVER_BIND`)
//! 3. Config file (`$DATA_DIR/zellimserver/config.toml`)
//! 4. Hard-coded defaults (`127.0.0.1:50051`)
//!
//! ## Config file format (TOML)
//!
//! ```toml
//! # zellimserver config — edit to override defaults.
//! bind_addr = "127.0.0.1:50051"
//! cert_dir  = "/home/user/.local/share/zellij/zellimserver"
//! log_path  = "/home/user/.local/share/zellij/zellimserver/zellimserver.log"
//! ```
//!
//! All fields are optional; missing fields fall back to defaults.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default bind address used when nothing else overrides it.
pub const DEFAULT_BIND: &str = "127.0.0.1:50051";

/// The raw on-disk config (all fields optional; missing → use defaults).
#[derive(Debug, Default, Deserialize, Serialize)]
struct FileConfig {
    bind_addr: Option<String>,
    cert_dir: Option<String>,
    log_path: Option<String>,
}

/// The fully-resolved effective configuration.
///
/// Every field is guaranteed to be populated (via the precedence chain).
#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    /// The address + port the server will bind to.
    pub bind_addr: String,
    /// Directory containing `server.crt` and `server.key`.
    pub cert_dir: PathBuf,
    /// Path where the server writes its log output (used in daemon mode).
    pub log_path: PathBuf,
    /// Path to the config file that was read (or would be written).
    pub config_file: PathBuf,
}

impl EffectiveConfig {
    /// Human-readable summary, one field per line.
    pub fn display(&self) -> String {
        format!(
            "bind_addr  = {}\n\
             cert_dir   = {}\n\
             log_path   = {}\n\
             config_file= {}",
            self.bind_addr,
            self.cert_dir.display(),
            self.log_path.display(),
            self.config_file.display(),
        )
    }
}

/// Returns the path to the config file inside the zellimserver data dir.
pub fn config_file_path() -> Result<PathBuf> {
    let data_dir = data_dir()?;
    Ok(data_dir.join("config.toml"))
}

/// Returns (and creates) the data directory: `$XDG_DATA_HOME/zellij/zellimserver/`.
///
/// On Unix the directory is restricted to mode `0700` (owner-only).  This is the
/// primary access control for the control socket (`control.sock`) and the
/// session pidfile that live inside it: there is no per-message auth on the
/// control socket, so a `0700` data dir is what prevents other local users from
/// connecting to it and issuing `Shutdown` (review Major D).
pub fn data_dir() -> Result<PathBuf> {
    let base = zellij_utils::consts::ZELLIJ_PROJ_DIR.data_dir();
    let dir = base.join("zellimserver");
    std::fs::create_dir_all(&dir).with_context(|| format!("create data dir {}", dir.display()))?;

    // Restrict directory permissions on Unix (matches `tls.rs::cert_dir`).  The
    // 0700 dir is the PRIMARY access control for the un-authenticated control
    // socket (see the doc comment), so a failure here is a real security
    // regression and must NOT be silently discarded (review round-2 minor):
    // surface it loudly (warn) and propagate the error so `init`/`start` fail
    // rather than running with a world-accessible control socket.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(&dir, perms).map_err(|e| {
            log::warn!(
                "config: FAILED to chmod 0700 the data dir {} ({e}); the control \
                 socket access guarantee is NOT in place — refusing to continue",
                dir.display()
            );
            anyhow::anyhow!(
                "failed to set 0700 permissions on data dir {}: {e}",
                dir.display()
            )
        })?;
    }

    Ok(dir)
}

/// The bundled bar-less default layout used for app-created sessions, baked into
/// the binary so the server is self-contained (no external file to deploy).
const DEFAULT_SESSION_LAYOUT_KDL: &str = include_str!("../assets/zellimobile-default.kdl");

/// File name for the materialised default layout under `<data_dir>/layouts/`.
const DEFAULT_SESSION_LAYOUT_FILE: &str = "zellimobile-default.kdl";

/// Materialise the bundled bar-less default layout to disk and return its
/// absolute path, for `CreateSession` to pass to `zellij --layout`.
///
/// The mobile client renders tab/pane controls itself, so sessions it creates
/// should not show zellij's tab-bar/status-bar. The built-in zellij default
/// layout declares those bar plugins; this layout (an empty
/// `default_tab_template`) does not, so every tab — including ones created at
/// runtime — opens bar-less.
///
/// Idempotent: the file is (over)written every call so a server upgrade always
/// ships the current layout. The returned path is a fixed, server-controlled
/// location derived from [`data_dir`] — never client input — so passing it to
/// `--layout` as an absolute path is safe (it is exempt from the client-layout
/// name allowlist enforced in `grpc::session_ops`).
pub fn ensure_default_session_layout() -> Result<PathBuf> {
    let dir = data_dir()?.join("layouts");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create layouts dir {}", dir.display()))?;
    let path = dir.join(DEFAULT_SESSION_LAYOUT_FILE);
    std::fs::write(&path, DEFAULT_SESSION_LAYOUT_KDL)
        .with_context(|| format!("write default layout {}", path.display()))?;
    Ok(path)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialise all config-file tests so they don't race on the shared file.
    static CONFIG_FILE_LOCK: Mutex<()> = Mutex::new(());

    /// RAII guard that restores the original config file content on drop.
    /// Ensures restoration runs even if the test panics.
    struct ConfigRestoreGuard {
        path: PathBuf,
        original: Option<String>,
    }

    impl Drop for ConfigRestoreGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(content) => {
                    let _ = std::fs::write(&self.path, content);
                }
                None => {
                    let _ = std::fs::remove_file(&self.path);
                }
            }
        }
    }

    /// Round-trip `set_bind_addr` → `resolve(None)`.
    ///
    /// This test writes to the real data-dir config file.  A mutex serialises it
    /// with other config tests, and a RAII guard restores the original content
    /// even if an assertion panics.
    #[test]
    fn set_bind_addr_round_trips() {
        let _guard = CONFIG_FILE_LOCK.lock().unwrap();

        // Save whatever is currently on disk (may not exist yet) and construct
        // the restore guard to ensure cleanup runs even on panic.
        let path = config_file_path().expect("config_file_path");
        let original = std::fs::read_to_string(&path).ok();
        let _restore = ConfigRestoreGuard {
            path: path.clone(),
            original,
        };

        // Act: write a distinctive bind address.
        let test_addr = "0.0.0.0:9999";
        set_bind_addr(test_addr).expect("set_bind_addr");

        // Assert: resolve(None) should return the value we wrote.
        // (No env override, no CLI override — file wins over default.)
        let cfg = resolve(None).expect("resolve");
        assert_eq!(
            cfg.bind_addr, test_addr,
            "resolved bind_addr should match what was written"
        );
        // Restore guard runs automatically when this scope exits.
    }

    /// The bundled default layout must be materialised on disk, live under the
    /// data dir's `layouts/`, and be BAR-LESS (no tab-bar/status-bar plugins).
    #[test]
    fn default_session_layout_is_written_and_barless() {
        let path = ensure_default_session_layout().expect("ensure default layout");
        assert!(path.exists(), "layout file should exist at {path:?}");
        assert!(
            path.ends_with("layouts/zellimobile-default.kdl"),
            "unexpected layout path {path:?}"
        );
        let body = std::fs::read_to_string(&path).expect("read layout");
        assert!(
            body.contains("default_tab_template"),
            "layout should define a default_tab_template"
        );
        // The bars are `plugin location=...` panes; a bar-less layout declares no
        // plugin panes at all. Check non-comment lines so the explanatory header
        // (which mentions tab-bar/status-bar) doesn't trip the assertion.
        let has_plugin = body
            .lines()
            .map(|l| l.trim_start())
            .filter(|l| !l.starts_with("//"))
            .any(|l| l.contains("plugin"));
        assert!(
            !has_plugin,
            "bar-less layout must not declare any plugin (tab-bar/status-bar) panes"
        );
    }
}

/// Load or create the config file, then apply env + CLI overrides.
///
/// `bind_override` — set when the user passes `--bind` on the CLI.
pub fn resolve(bind_override: Option<&str>) -> Result<EffectiveConfig> {
    let config_file = config_file_path()?;
    let data_dir = data_dir()?;

    // ── Read file (tolerant — missing file → defaults; parse error → warn) ──
    let file_cfg: FileConfig = if config_file.exists() {
        let raw = std::fs::read_to_string(&config_file)
            .with_context(|| format!("read {}", config_file.display()))?;
        toml::from_str(&raw).unwrap_or_else(|e| {
            log::warn!(
                "config: failed to parse {}: {e} — using defaults",
                config_file.display()
            );
            FileConfig::default()
        })
    } else {
        FileConfig::default()
    };

    // ── Precedence chain ─────────────────────────────────────────────────────

    // bind_addr: CLI flag > ZELLIMSERVER_BIND env > config file > default
    let bind_addr = bind_override
        .map(|s| s.to_owned())
        .or_else(|| std::env::var("ZELLIMSERVER_BIND").ok())
        .or(file_cfg.bind_addr)
        .unwrap_or_else(|| DEFAULT_BIND.to_owned());

    // cert_dir: config file > default (data_dir)
    let cert_dir = file_cfg
        .cert_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.clone());

    // log_path: config file > default (data_dir/zellimserver.log)
    let log_path = file_cfg
        .log_path
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("zellimserver.log"));

    Ok(EffectiveConfig {
        bind_addr,
        cert_dir,
        log_path,
        config_file,
    })
}

/// Persist `bind_addr` into the config file (creating it from the template if
/// absent), preserving all other fields.
///
/// Used by `zellimctl`'s Config screen so users can update the bind address
/// without manually editing TOML.  The write is done atomically: read the
/// current config → update `bind_addr` → serialize → write to a temp file in
/// the same directory → `chmod 0600` the temp file (Unix) → `rename` the temp
/// file over the real config (POSIX-atomic on the same filesystem). This ensures
/// no crash mid-write leaves a partial or world-readable file, and no other
/// fields are silently lost.
pub fn set_bind_addr(bind_addr: &str) -> Result<()> {
    // Ensure the file exists (idempotent; writes a template when absent).
    let path = ensure_config_file()?;

    // Read the current content, falling back to defaults on parse errors so a
    // corrupted file doesn't block the update.
    let file_cfg: FileConfig = if path.exists() {
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("set_bind_addr: read {}", path.display()))?;
        toml::from_str(&raw).unwrap_or_else(|e| {
            log::warn!(
                "config: set_bind_addr: failed to parse {}: {e} — overwriting with defaults + new bind_addr",
                path.display()
            );
            FileConfig::default()
        })
    } else {
        FileConfig::default()
    };

    // Update the bind_addr field and serialise.
    let updated = FileConfig {
        bind_addr: Some(bind_addr.to_owned()),
        ..file_cfg
    };
    let toml_str = toml::to_string_pretty(&updated).context("set_bind_addr: serialize config")?;

    // Write to a temporary file in the same directory as the target.
    let parent = path
        .parent()
        .context("config file has no parent directory")?;
    let temp_path = parent.join(".config.toml.tmp");
    std::fs::write(&temp_path, &toml_str)
        .with_context(|| format!("set_bind_addr: write temp file {}", temp_path.display()))?;

    // Restrict permissions on the temp file before renaming (Unix).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&temp_path, perms).with_context(|| {
            format!(
                "set_bind_addr: chmod 0600 temp file {}",
                temp_path.display()
            )
        })?;
    }

    // Atomically rename temp file over the real config (POSIX-atomic on same filesystem).
    std::fs::rename(&temp_path, &path).with_context(|| {
        format!(
            "set_bind_addr: rename {} to {}",
            temp_path.display(),
            path.display()
        )
    })?;

    log::info!(
        "config: set bind_addr = {bind_addr:?} in {}",
        path.display()
    );
    Ok(())
}

/// Ensure the config file exists.  If it doesn't, write a commented template
/// with the current defaults so the user can inspect and edit it.
pub fn ensure_config_file() -> Result<PathBuf> {
    let path = config_file_path()?;
    if !path.exists() {
        let data_dir = data_dir()?;
        let template = format!(
            "# zellimserver configuration file\n\
             # All fields are optional; missing fields use built-in defaults.\n\
             #\n\
             # bind_addr = \"{DEFAULT_BIND}\"\n\
             # cert_dir  = \"{}\"\n\
             # log_path  = \"{}\"\n",
            data_dir.display(),
            data_dir.join("zellimserver.log").display(),
        );
        std::fs::write(&path, template)
            .with_context(|| format!("write config file {}", path.display()))?;
        log::info!("config: created template at {}", path.display());
    }
    Ok(path)
}
