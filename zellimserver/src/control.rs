//! control — the zellimserver control socket (Phase E2).
//!
//! A tiny local-IPC contract used by `status`/`stop` to talk to a running
//! server.  It mirrors zellij's `web_server_commands` pattern (a
//! length-prefixed message over an `interprocess` `LocalSocketStream`) but uses
//! our **own** JSON contract instead of protobuf — the surface is two requests
//! (`Shutdown`, `Status`) and two responses (`Ok`, `Status{...}`).
//!
//! ## Wire format
//!
//! Each message is a `u32` little-endian length prefix followed by that many
//! bytes of `serde_json`.  Request and response use the same framing.
//!
//! ## Lifecycle
//!
//! The running server (foreground or daemon) calls [`spawn_listener`] which
//! binds the socket at [`socket_path`] and spawns a blocking accept loop on a
//! dedicated OS thread (NOT a tokio task — the accept loop is sync IPC and we
//! keep it off the runtime).  On a `Shutdown` request it fires the provided
//! shutdown trigger (a `tokio::sync::oneshot` sender wired into
//! `serve_with_shutdown`) and returns `Ok`.
//!
//! `status`/`stop` use [`query`] to send a single request and read the reply.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{self, CertMode};

/// Filename of the control socket inside the data dir.
const SOCKET_NAME: &str = "control.sock";

/// Serde default for [`StatusInfo::cert_mode`]: assume `SelfSigned` when the
/// field is absent (older server).
fn default_cert_mode() -> CertMode {
    CertMode::SelfSigned
}

/// Upper bound on a single control message body (64 KiB).
///
/// The wire format is a `u32` length prefix followed by that many bytes.  A
/// crafted prefix could otherwise request a multi-gigabyte `vec![0u8; len]`
/// allocation (review Major D — control-socket hardening); we reject anything
/// larger than this before allocating.  The real messages are tiny JSON blobs.
const MAX_CONTROL_MSG: usize = 64 * 1024;

/// A request sent to the running server over the control socket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlRequest {
    /// Ask the server to shut down gracefully.
    Shutdown,
    /// Ask the server for its status (version / bind / pid / uptime).
    Status,
}

/// A response from the running server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlResponse {
    /// Generic acknowledgement (e.g. after `Shutdown`).
    Ok,
    /// Status payload (reply to [`ControlRequest::Status`]).
    Status(StatusInfo),
}

/// Server status reported over the control socket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInfo {
    /// The server crate version (`CARGO_PKG_VERSION`).
    pub version: String,
    /// The address the server is bound to.
    pub bind_addr: String,
    /// The server process id.
    pub pid: u32,
    /// Seconds the server has been running.
    pub uptime_secs: u64,
    /// Total number of mobile clients currently attached across all sessions.
    ///
    /// Defaults to 0 so that older clients / messages that pre-date this field
    /// still deserialize correctly (`#[serde(default)]`).
    #[serde(default)]
    pub client_count: usize,
    /// The active TLS / transport mode (self_signed, external, h2c).
    ///
    /// Defaults to `SelfSigned` when deserialising a response from an older
    /// server that pre-dates this field, which is the most conservative
    /// assumption for backward compatibility.
    #[serde(default = "default_cert_mode")]
    pub cert_mode: CertMode,
}

/// Path to the control socket: `data_dir()/control.sock`.
pub fn socket_path() -> Result<PathBuf> {
    Ok(config::data_dir()?.join(SOCKET_NAME))
}

// ── Framing helpers ───────────────────────────────────────────────────────────

fn write_msg<W: Write, T: Serialize>(w: &mut W, msg: &T) -> Result<()> {
    let bytes = serde_json::to_vec(msg).context("control: serialize message")?;
    let len = u32::try_from(bytes.len()).context("control: message too large")?;
    w.write_all(&len.to_le_bytes())
        .context("control: write length prefix")?;
    w.write_all(&bytes).context("control: write body")?;
    w.flush().context("control: flush")?;
    Ok(())
}

fn read_msg<R: Read, T: for<'de> Deserialize<'de>>(r: &mut R) -> Result<T> {
    let mut len_bytes = [0u8; 4];
    r.read_exact(&mut len_bytes)
        .context("control: read length prefix")?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    // Reject an oversized length prefix BEFORE allocating, so a crafted prefix
    // can't drive a giant `vec![0u8; len]` allocation (review Major D).
    if len > MAX_CONTROL_MSG {
        anyhow::bail!("control: message length {len} exceeds maximum {MAX_CONTROL_MSG}");
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).context("control: read body")?;
    serde_json::from_slice(&buf).context("control: deserialize message")
}

// ── Client side (status / stop) ───────────────────────────────────────────────

/// Send a single request to the running server and read its response.
///
/// Returns an error if the socket is absent or the server is unresponsive —
/// callers treat that as "not running".
pub fn query(req: &ControlRequest) -> Result<ControlResponse> {
    let path = socket_path()?;
    if !path.exists() {
        anyhow::bail!("control socket {} does not exist", path.display());
    }
    let mut stream = zellij_utils::consts::ipc_connect(&path)
        .with_context(|| format!("control: connect to {}", path.display()))?;
    write_msg(&mut stream, req)?;
    read_msg(&mut stream)
}

// ── Server side (listener) ────────────────────────────────────────────────────

/// Shared shutdown trigger: a oneshot sender consumed on the first `Shutdown`.
type ShutdownTrigger = Mutex<Option<tokio::sync::oneshot::Sender<()>>>;

/// Spawn the control-socket accept loop on a dedicated OS thread.
///
/// `bind_addr` is the effective server bind address (reported in `Status`).
/// `started_at` is the instant the server began (for `uptime_secs`).
/// `shutdown_tx` is fired the first time a `Shutdown` request arrives; it should
/// be the trigger side of a `serve_with_shutdown` future.
/// `clients` is a cloneable handle to the per-session attached-client registry
/// used to report the total client count in `Status` responses.
/// `cert_mode` is the active TLS / transport mode reported in `Status` responses.
///
/// The socket is bound up-front (so a `status`/`stop` race right after start
/// still finds it) and removed by the caller on exit (see [`cleanup`]).
pub fn spawn_listener(
    bind_addr: String,
    started_at: Instant,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    clients: crate::client_count::SessionClients,
    cert_mode: CertMode,
) -> Result<()> {
    let path = socket_path()?;

    // Remove any stale socket file from a previous (crashed) run; ipc_bind
    // fails with EADDRINUSE on a leftover socket path.
    let _ = std::fs::remove_file(&path);

    // `incoming()` comes from the ListenerExt trait — bring it into scope.
    use interprocess::local_socket::traits::ListenerExt;

    let listener = zellij_utils::consts::ipc_bind(&path)
        .with_context(|| format!("control: bind {}", path.display()))?;

    let trigger: std::sync::Arc<ShutdownTrigger> =
        std::sync::Arc::new(Mutex::new(Some(shutdown_tx)));

    std::thread::Builder::new()
        .name("zellimserver-control".to_string())
        .spawn(move || {
            for conn in listener.incoming() {
                let mut stream = match conn {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("control: accept error: {e}");
                        continue;
                    }
                };
                let req: ControlRequest = match read_msg(&mut stream) {
                    Ok(r) => r,
                    Err(e) => {
                        log::warn!("control: bad request: {e:#}");
                        continue;
                    }
                };
                let resp = match req {
                    ControlRequest::Status => ControlResponse::Status(StatusInfo {
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        bind_addr: bind_addr.clone(),
                        pid: std::process::id(),
                        uptime_secs: started_at.elapsed().as_secs(),
                        client_count: clients.total_count(),
                        cert_mode,
                    }),
                    ControlRequest::Shutdown => {
                        log::info!("control: shutdown requested");
                        if let Some(tx) = trigger.lock().unwrap().take() {
                            let _ = tx.send(());
                        }
                        ControlResponse::Ok
                    }
                };
                if let Err(e) = write_msg(&mut stream, &resp) {
                    log::warn!("control: failed to write response: {e:#}");
                }
                if matches!(req, ControlRequest::Shutdown) {
                    // The server is winding down; stop accepting.
                    break;
                }
            }
            log::debug!("control: accept loop exited");
        })
        .context("control: spawn accept thread")?;

    Ok(())
}

/// Remove the control socket file (best-effort).
pub fn cleanup() {
    if let Ok(path) = socket_path() {
        let _ = std::fs::remove_file(&path);
    }
}
