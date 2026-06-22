//! tls — self-signed TLS certificate management.
//!
//! Generates a self-signed certificate + key pair for `127.0.0.1` and
//! `localhost` using `rcgen`, persists them as PEM files under the zellij
//! data dir (`ZELLIJ_PROJ_DIR.data_dir()/zellimserver/`), and reloads them
//! on restart so the client can pin the same cert across server restarts.
//!
//! ## Cert dir layout
//!
//! ```text
//! $XDG_DATA_HOME/zellij/zellimserver/
//!     server.crt      — X.509 certificate (PEM)
//!     server.key      — private key (PEM)
//!     server.san.json — JSON list of extra SANs included when cert was generated
//! ```
//!
//! The directory is created with `0700` permissions; the files themselves
//! inherit `0600` from the umask (or are `chmod`-ed explicitly on Unix).
//!
//! ## SAN support
//!
//! The cert always covers `127.0.0.1` and `localhost` (built-in).  Additional
//! SANs can be requested via [`SanEntry`] (parsed from `--san` CLI flags or the
//! `ZELLIMSERVER_SAN` environment variable, comma-separated).  If the on-disk
//! cert does not cover all requested extra SANs it is regenerated.
//!
//! ## Usage
//!
//! ```no_run
//! use zellimserver::tls::{load_or_generate_identity, SanEntry};
//! use tonic::transport::{ServerTlsConfig, Identity};
//!
//! let extra: Vec<SanEntry> = vec![SanEntry::parse("100.64.0.5")];
//! let (identity, cert_pem) = load_or_generate_identity(&extra).unwrap();
//! let tls = ServerTlsConfig::new().identity(identity);
//! ```

use anyhow::{Context, Result};
use rcgen::{CertificateParams, KeyPair, SanType};
use sha2::Digest;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use tonic::transport::Identity;

// ─── SAN entry ────────────────────────────────────────────────────────────────

/// A Subject Alternative Name entry for the self-signed certificate.
///
/// Parsed from CLI `--san` values or the `ZELLIMSERVER_SAN` environment
/// variable (comma-separated).  Each value is tried as an IP address first;
/// on parse failure it is treated as a DNS name.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SanEntry {
    /// An IP-address SAN (IPv4 or IPv6).
    Ip(std::net::IpAddr),
    /// A DNS-name SAN.
    Dns(String),
}

impl SanEntry {
    /// Parse a single SAN value: IP if it parses as one, otherwise DNS.
    pub fn parse(s: &str) -> Self {
        if let Ok(ip) = s.trim().parse::<IpAddr>() {
            SanEntry::Ip(ip)
        } else {
            SanEntry::Dns(s.trim().to_owned())
        }
    }

    /// Parse the `ZELLIMSERVER_SAN` environment variable (comma-separated list).
    ///
    /// Returns an empty vec when the env var is absent or empty.
    pub fn from_env() -> Vec<Self> {
        std::env::var("ZELLIMSERVER_SAN")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(Self::parse)
            .collect()
    }

    /// Human-readable display string (used in log output).
    fn display(&self) -> String {
        match self {
            SanEntry::Ip(ip) => ip.to_string(),
            SanEntry::Dns(d) => d.clone(),
        }
    }
}

// ─── Cert dir ─────────────────────────────────────────────────────────────────

/// Returns the directory where the server cert/key are persisted.
fn cert_dir() -> Result<PathBuf> {
    let base = zellij_utils::consts::ZELLIJ_PROJ_DIR.data_dir();
    let dir = base.join("zellimserver");
    std::fs::create_dir_all(&dir).with_context(|| format!("create cert dir {}", dir.display()))?;

    // Restrict directory permissions on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        let _ = std::fs::set_permissions(&dir, perms);
    }

    Ok(dir)
}

// ─── SAN sidecar ──────────────────────────────────────────────────────────────

/// Read the persisted extra-SAN list from `server.san.json`.
///
/// Returns an empty vec if the file is absent or unparseable (that means the
/// cert was generated without extras → no extra SANs covered).
///
/// Exposed as `pub` so the `zellimctl` server facade can reuse this parser
/// instead of duplicating it with its own `serde_json` call.
pub fn read_san_sidecar(dir: &Path) -> Vec<SanEntry> {
    let path = dir.join("server.san.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        log::warn!("tls: failed to parse SAN sidecar {}: {e}", path.display());
        vec![]
    })
}

/// Persist the extra-SAN list to `server.san.json`.
fn write_san_sidecar(dir: &Path, sans: &[SanEntry]) -> Result<()> {
    let path = dir.join("server.san.json");
    let json = serde_json::to_string(sans).context("tls: serialize SAN list")?;
    write_restricted(&path, &json)
        .with_context(|| format!("tls: write SAN sidecar {}", path.display()))
}

/// Returns `true` if `stored_sans` covers every entry in `requested`.
///
/// The built-in `127.0.0.1` / `localhost` are always present in the cert, so
/// only the *extra* SANs beyond those need to be tracked in the sidecar.
fn sidecar_covers(stored_sans: &[SanEntry], requested: &[SanEntry]) -> bool {
    requested.iter().all(|r| stored_sans.contains(r))
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Load the persisted cert+key, or generate a fresh self-signed pair and
/// persist it for future restarts.
///
/// `extra_sans` — additional Subject Alternative Names beyond the built-in
/// `127.0.0.1` / `localhost`.  If the on-disk cert does NOT cover all requested
/// SANs the cert+key are regenerated (and the new SANs persisted to a sidecar).
///
/// Returns `(Identity, cert_pem_string)`:
/// - `Identity` — passed directly to `ServerTlsConfig::identity()`.
/// - `cert_pem_string` — the public certificate in PEM format; hand this to
///   clients so they can trust the self-signed cert without a CA.
pub fn load_or_generate_identity(extra_sans: &[SanEntry]) -> Result<(Identity, String)> {
    let dir = cert_dir()?;
    let cert_path = dir.join("server.crt");
    let key_path = dir.join("server.key");

    if cert_path.exists() && key_path.exists() {
        let stored_sans = read_san_sidecar(&dir);
        if sidecar_covers(&stored_sans, extra_sans) {
            let cert_pem = std::fs::read_to_string(&cert_path)
                .with_context(|| format!("read {}", cert_path.display()))?;
            let key_pem = std::fs::read_to_string(&key_path)
                .with_context(|| format!("read {}", key_path.display()))?;
            log::info!(
                "tls: loading existing cert/key from {} (covers all requested SANs)",
                dir.display()
            );
            let identity = Identity::from_pem(cert_pem.as_bytes(), key_pem.as_bytes());
            return Ok((identity, cert_pem));
        }
        log::info!(
            "tls: existing cert at {} does not cover all requested SANs — regenerating",
            cert_path.display()
        );
    }

    let san_desc: Vec<String> = extra_sans.iter().map(SanEntry::display).collect();
    log::info!(
        "tls: generating self-signed cert for 127.0.0.1 + localhost{}{} → {}",
        if san_desc.is_empty() { "" } else { " + " },
        san_desc.join(", "),
        dir.display()
    );

    let (cert_pem, key_pem) = generate_self_signed_pem(extra_sans)?;

    // Persist cert, key, and the SAN sidecar.
    write_restricted(&cert_path, &cert_pem)
        .with_context(|| format!("write {}", cert_path.display()))?;
    write_restricted(&key_path, &key_pem)
        .with_context(|| format!("write {}", key_path.display()))?;
    write_san_sidecar(&dir, extra_sans)?;

    log::info!("tls: persisted cert to {}", cert_path.display());

    let identity = Identity::from_pem(cert_pem.as_bytes(), key_pem.as_bytes());
    Ok((identity, cert_pem))
}

/// Generate a fresh self-signed cert+key for `127.0.0.1`, `localhost`, and any
/// `extra_sans`.
///
/// Returns `(cert_pem, key_pem)`.
///
/// This function is `pub` primarily to enable unit testing.
pub fn generate_self_signed_pem(extra_sans: &[SanEntry]) -> Result<(String, String)> {
    let signing_key = KeyPair::generate().context("rcgen: generate key pair")?;

    let mut params = CertificateParams::new(vec!["localhost".to_owned()])
        .context("rcgen: CertificateParams::new")?;

    // Always include 127.0.0.1.
    params
        .subject_alt_names
        .push(SanType::IpAddress(IpAddr::V4(std::net::Ipv4Addr::new(
            127, 0, 0, 1,
        ))));

    // Add any extra SANs.
    for san in extra_sans {
        match san {
            SanEntry::Ip(ip) => {
                params.subject_alt_names.push(SanType::IpAddress(*ip));
            }
            SanEntry::Dns(name) => {
                params.subject_alt_names.push(SanType::DnsName(
                    name.clone()
                        .try_into()
                        .with_context(|| format!("rcgen: invalid DNS SAN {:?}", name))?,
                ));
            }
        }
    }

    let cert = params
        .self_signed(&signing_key)
        .context("rcgen: self_signed")?;

    Ok((cert.pem(), signing_key.serialize_pem()))
}

/// Write `content` to `path` and restrict its permissions to `0600` on Unix.
fn write_restricted(path: &std::path::Path, content: &str) -> Result<()> {
    std::fs::write(path, content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }

    Ok(())
}

/// Lowercase hex SHA-256 of the leaf certificate's DER bytes (no colons).
///
/// Used by `zellimctl` to embed a pin in the pairing QR code; the mobile client
/// recomputes it over the presented certificate and compares the fingerprint before
/// trusting the connection.
///
/// `cert_pem` must contain at least one `CERTIFICATE` PEM block.  Only the
/// **first** block (the leaf certificate) is hashed.
///
/// Returns a 64-character lowercase hex string.
pub fn cert_sha256_fingerprint(cert_pem: &str) -> Result<String> {
    use std::io::BufReader;

    let mut reader = BufReader::new(cert_pem.as_bytes());
    let der = rustls_pemfile::certs(&mut reader)
        .next()
        .ok_or_else(|| anyhow::anyhow!("tls: no CERTIFICATE block found in PEM input"))?
        .context("tls: failed to decode certificate DER from PEM")?;

    let digest = sha2::Sha256::digest(der.as_ref());
    Ok(format!("{:064x}", digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_64_hex_chars_and_deterministic() {
        let (cert_pem, _key_pem) = generate_self_signed_pem(&[]).expect("generate cert");

        let fp1 = cert_sha256_fingerprint(&cert_pem).expect("fingerprint 1");
        let fp2 = cert_sha256_fingerprint(&cert_pem).expect("fingerprint 2");

        // Must be exactly 64 lowercase hex characters.
        assert_eq!(fp1.len(), 64, "fingerprint length should be 64 chars");
        assert!(
            fp1.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "fingerprint should be lowercase hex: {fp1}"
        );

        // Must be deterministic for the same input.
        assert_eq!(fp1, fp2, "fingerprint should be deterministic");
    }
}
