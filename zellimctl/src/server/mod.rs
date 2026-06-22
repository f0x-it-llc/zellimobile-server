//! server — typed facade over zellimserver's library surface.
//!
//! Wraps `zellimserver::{config, control, tls}` into a small, ergonomic API
//! that the TUI screens consume.  The running server binary is launched via
//! [`start_daemon`] (spawn, not library call), while all pure read/write ops
//! (config, cert, status) go through the library directly.

pub mod tokens;

use anyhow::{Context, Result};
use zellimserver::config::EffectiveConfig;
use zellimserver::control::{ControlRequest, ControlResponse, StatusInfo};
use zellimserver::tls::SanEntry;

use crate::app::state::{San, ServerInfo};

// ── Infra ↔ app-layer conversions ──────────────────────────────────────────────
//
// This facade is the ONLY place that imports `zellimserver` types. It converts
// them into the app-layer mirrors (`ServerInfo`, `San`) so the TEA `app/` layer
// stays free of infra dependencies.

/// Convert the infra `StatusInfo` into the app-layer [`ServerInfo`] mirror.
fn server_info_from(info: StatusInfo) -> ServerInfo {
    ServerInfo {
        version: info.version,
        bind_addr: info.bind_addr,
        pid: info.pid,
        uptime_secs: info.uptime_secs,
        client_count: info.client_count,
    }
}

/// Convert an app-layer [`San`] into the infra `SanEntry`.
///
/// `San::Ip` values are re-parsed; an unparseable IP string falls back to a DNS
/// SAN so a malformed entry never silently disappears.
fn san_entry_from(san: &San) -> SanEntry {
    match san {
        San::Ip(s) => match s.parse::<std::net::IpAddr>() {
            Ok(ip) => SanEntry::Ip(ip),
            Err(_) => SanEntry::Dns(s.clone()),
        },
        San::Dns(d) => SanEntry::Dns(d.clone()),
    }
}

// ── Status / control ──────────────────────────────────────────────────────────

/// Query the running server for its status.
///
/// Returns `Some(ServerInfo)` when the server answers; a connection error
/// (socket absent, server unresponsive) maps to `None` ("stopped").
#[allow(dead_code)]
pub fn status() -> Option<ServerInfo> {
    match zellimserver::control::query(&ControlRequest::Status) {
        Ok(ControlResponse::Status(info)) => Some(server_info_from(info)),
        _ => None,
    }
}

/// Ask the running server to shut down gracefully.
#[allow(dead_code)]
pub fn stop() -> Result<()> {
    zellimserver::control::query(&ControlRequest::Shutdown)
        .context("stop: control socket query failed")?;
    Ok(())
}

/// Return the number of mobile clients currently attached, or `None` if the
/// server is not running.
#[allow(dead_code)]
pub fn client_count() -> Option<usize> {
    status().map(|info| info.client_count)
}

/// Whether the local zellimserver daemon is currently running.
#[allow(dead_code)]
pub fn is_running() -> bool {
    status().is_some()
}

// ── Daemon launch ─────────────────────────────────────────────────────────────

/// Read the persisted SAN sidecar (`<cert_dir>/server.san.json`) and return the
/// raw string values of each entry (IP address strings or DNS names).
///
/// Tolerates a missing file or a parse error — both map to an empty vec, which
/// causes no `--san` flags to be appended (the daemon re-derives its own defaults).
///
/// NOTE: This function lives in the `server/` facade (the only layer allowed to
/// import `zellimserver::` types) so the `app/` layer stays clean.
#[allow(dead_code)]
pub fn persisted_extra_sans() -> Vec<String> {
    let cfg = match effective_config() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("server: persisted_extra_sans: config error: {e}");
            return vec![];
        }
    };
    let path = cfg.cert_dir.join("server.san.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(r) => r,
        Err(_) => return vec![], // file absent → no extra SANs
    };
    let entries: Vec<zellimserver::tls::SanEntry> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            log::warn!(
                "server: persisted_extra_sans: failed to parse {}: {e}",
                path.display()
            );
            return vec![];
        }
    };
    entries
        .into_iter()
        .map(|e| match e {
            zellimserver::tls::SanEntry::Ip(ip) => ip.to_string(),
            zellimserver::tls::SanEntry::Dns(d) => d,
        })
        .collect()
}

/// Assemble the argument list for the `zellimserver start` command.
///
/// Extracted into a pure function so unit tests can assert the produced args
/// without spawning a real process.
///
/// Arguments:
/// - `bind_addr` — current effective bind address (forwarded as `--bind` only
///   when non-default).
/// - `extra_sans` — raw SAN strings read from the cert sidecar; each becomes a
///   `--san <value>` pair so the daemon never regenerates the cert away from the
///   one `zellimctl` pinned.
/// - `default_bind` — the default bind address constant from `zellimserver`
///   (passed in so this function is pure / not cfg-dependent).
///
/// Always starts with `["start", "--daemonize"]`.
#[allow(dead_code)]
pub fn build_daemon_args(bind_addr: &str, extra_sans: &[String], default_bind: &str) -> Vec<String> {
    let mut args = vec!["start".to_string(), "--daemonize".to_string()];

    if bind_addr != default_bind {
        args.push("--bind".to_string());
        args.push(bind_addr.to_string());
    }

    for san in extra_sans {
        args.push("--san".to_string());
        args.push(san.clone());
    }

    args
}

/// Spawn the `zellimserver` daemon process and return immediately.
///
/// The binary is located by checking the directory that contains the current
/// executable first (the workspace target dir places both binaries together),
/// then falling back to `zellimserver` on `$PATH`.
///
/// Forwards `--bind` (when non-default) and one `--san <value>` per entry from
/// the persisted SAN sidecar, so the daemon always serves exactly the cert that
/// `zellimctl` generated rather than re-deriving SANs from scratch.
///
/// Readiness must be confirmed by the caller via polling [`status()`].
#[allow(dead_code)]
pub fn start_daemon() -> Result<()> {
    let bin = find_server_binary();

    let (bind_addr, extra_sans) = match effective_config() {
        Ok(cfg) => {
            let sans = persisted_extra_sans();
            (cfg.bind_addr, sans)
        }
        Err(e) => {
            // If we cannot resolve config, proceed with defaults and no SANs
            // so the daemon can still start (it will use its own defaults).
            log::warn!("start_daemon: could not resolve config: {e}");
            (zellimserver::config::DEFAULT_BIND.to_string(), vec![])
        }
    };

    let args = build_daemon_args(&bind_addr, &extra_sans, zellimserver::config::DEFAULT_BIND);

    let mut cmd = std::process::Command::new(&bin);
    cmd.args(&args);
    cmd.spawn()
        .with_context(|| format!("start_daemon: failed to spawn {}", bin.display()))?;

    log::info!("server: spawned zellimserver daemon ({})", bin.display());
    Ok(())
}

/// Locate the `zellimserver` binary.
///
/// 1. Try `<current_exe_parent>/zellimserver` (workspace target dir co-location).
/// 2. Fall back to `zellimserver` by name (resolved by the OS via `$PATH`).
fn find_server_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let candidate = parent.join("zellimserver");
        if candidate.exists() {
            return candidate;
        }
    }
    // Fall back: let the shell/OS resolve via PATH.
    std::path::PathBuf::from("zellimserver")
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Resolve and return the effective configuration (no CLI override).
#[allow(dead_code)]
pub fn effective_config() -> Result<EffectiveConfig> {
    zellimserver::config::resolve(None).context("effective_config: failed to resolve config")
}

/// Persist a new bind address to the config file.
#[allow(dead_code)]
pub fn set_bind_addr(addr: &str) -> Result<()> {
    zellimserver::config::set_bind_addr(addr)
        .with_context(|| format!("set_bind_addr: failed to write {addr:?}"))
}

// ── TLS / cert ────────────────────────────────────────────────────────────────

/// Load or generate the TLS identity for the given SANs, returning the cert PEM.
///
/// The underlying [`zellimserver::tls::load_or_generate_identity`] persists the
/// cert+key under the zellij data dir so successive calls reuse the same cert.
///
/// Takes app-layer [`San`] mirrors (converted to infra `SanEntry` here, in the
/// single facade boundary).
#[allow(dead_code)]
pub fn ensure_cert(sans: &[San]) -> Result<String> {
    let entries: Vec<SanEntry> = sans.iter().map(san_entry_from).collect();
    let (_identity, cert_pem) = zellimserver::tls::load_or_generate_identity(&entries)
        .context("ensure_cert: failed to load or generate TLS identity")?;
    Ok(cert_pem)
}

/// Compute the lowercase hex SHA-256 fingerprint of a certificate PEM string.
///
/// Thin wrapper over [`zellimserver::tls::cert_sha256_fingerprint`].
#[allow(dead_code)]
pub fn cert_fingerprint(cert_pem: &str) -> Result<String> {
    zellimserver::tls::cert_sha256_fingerprint(cert_pem)
        .context("cert_fingerprint: failed to compute SHA-256 fingerprint")
}

/// Read the **persisted** server cert and return its SHA-256 fingerprint,
/// **without** generating or regenerating anything.
///
/// This is the fingerprint of the cert the running server actually serves (it
/// loads `server.crt` at startup). Pairing must pin THIS value — calling
/// [`ensure_cert`] during pairing could regenerate the on-disk cert and pin a
/// fingerprint the running server will never present (see review Critical #1).
///
/// Returns:
/// - `Ok(Some(fp))` when `cert_dir/server.crt` exists and is parseable,
/// - `Ok(None)` when no cert has been generated yet,
/// - `Err(_)` on an I/O or fingerprint-computation failure.
#[allow(dead_code)]
pub fn current_cert_fingerprint() -> Result<Option<String>> {
    let cfg = effective_config()?;
    let cert_path = cfg.cert_dir.join("server.crt");
    if !cert_path.exists() {
        return Ok(None);
    }
    let pem = std::fs::read_to_string(&cert_path)
        .with_context(|| format!("current_cert_fingerprint: read {}", cert_path.display()))?;
    let fp = zellimserver::tls::cert_sha256_fingerprint(&pem)
        .context("current_cert_fingerprint: failed to compute SHA-256 fingerprint")?;
    Ok(Some(fp))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: &str = zellimserver::config::DEFAULT_BIND;

    #[test]
    fn build_daemon_args_default_bind_omits_bind_flag() {
        // When the bind address matches the default, --bind must NOT be emitted
        // (the server already defaults to that address).
        let args = build_daemon_args(DEFAULT, &[], DEFAULT);
        assert_eq!(args, vec!["start", "--daemonize"]);
    }

    #[test]
    fn build_daemon_args_non_default_bind_included() {
        let args = build_daemon_args("0.0.0.0:50051", &[], DEFAULT);
        assert!(
            args.windows(2)
                .any(|w| w[0] == "--bind" && w[1] == "0.0.0.0:50051"),
            "expected --bind 0.0.0.0:50051 in {args:?}"
        );
    }

    #[test]
    fn build_daemon_args_each_san_becomes_flag_pair() {
        let sans = vec!["100.64.1.2".to_string(), "192.168.1.10".to_string()];
        let args = build_daemon_args(DEFAULT, &sans, DEFAULT);
        // Each SAN must appear as an adjacent ("--san", "<value>") pair.
        let pairs: Vec<(&str, &str)> = args
            .windows(2)
            .filter_map(|w| {
                if w[0] == "--san" {
                    Some((w[0].as_str(), w[1].as_str()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(pairs.len(), 2, "expected 2 --san pairs; got: {args:?}");
        assert!(
            pairs.iter().any(|&(_, v)| v == "100.64.1.2"),
            "100.64.1.2 missing from --san flags: {args:?}"
        );
        assert!(
            pairs.iter().any(|&(_, v)| v == "192.168.1.10"),
            "192.168.1.10 missing from --san flags: {args:?}"
        );
    }

    #[test]
    fn build_daemon_args_bind_and_sans_combined() {
        let sans = vec!["100.64.0.5".to_string()];
        let args = build_daemon_args("0.0.0.0:50051", &sans, DEFAULT);
        // Must have both --bind and --san.
        assert!(
            args.windows(2)
                .any(|w| w[0] == "--bind" && w[1] == "0.0.0.0:50051"),
            "--bind missing: {args:?}"
        );
        assert!(
            args.windows(2)
                .any(|w| w[0] == "--san" && w[1] == "100.64.0.5"),
            "--san missing: {args:?}"
        );
    }

    #[test]
    fn build_daemon_args_empty_sans_no_san_flags() {
        let args = build_daemon_args("0.0.0.0:50051", &[], DEFAULT);
        assert!(
            !args.iter().any(|a| a == "--san"),
            "no --san flags expected with empty sans: {args:?}"
        );
    }
}
