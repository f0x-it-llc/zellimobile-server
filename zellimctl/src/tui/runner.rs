//! The TUI event loop.
//!
//! Mirrors fdemon's proven **poll + drain** runner (NOT `tokio::select!`):
//!
//! 1. Drain all pending [`Message`]s from the mpsc channel via `try_recv` and
//!    feed each to [`update`], collecting [`UpdateAction`]s.
//! 2. Apply runner-side effects: `Quit` exits the loop; async actions are
//!    dispatched onto `tokio::task::spawn_blocking` tasks that post their
//!    results back over a cloned `tx`.
//! 3. `terminal.draw(...)` **unconditionally** every ~50 ms tick — ratatui's
//!    double-buffer diff suppresses redundant terminal writes, and the steady
//!    cadence is what live status polling rides on.
//! 4. `crossterm::event::poll(50 ms)` → on a key press, send `Message::Key`;
//!    on timeout, send `Message::Tick`.
//!
//! The loop exits as soon as `AppState.should_quit` is set.

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use crate::app::{AppState, Message, UpdateAction, update};

/// Poll cadence / tick interval.
const TICK: Duration = Duration::from_millis(50);

/// Channel capacity for the message bus. Cloned senders feed async task results.
const CHANNEL_CAPACITY: usize = 256;

/// Run the TUI to completion. Returns when the user quits.
///
/// Owns the message channel: the receiver is drained here; the sender is cloned
/// into each async [`UpdateAction`] task so results post back into the loop.
pub fn run(terminal: &mut DefaultTerminal, state: &mut AppState) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<Message>(CHANNEL_CAPACITY);

    while !state.should_quit {
        // (1) Drain all pending messages and run the update cycle.
        while let Ok(message) = rx.try_recv() {
            let actions = update(state, message);
            apply_actions(state, actions, tx.clone());
        }

        if state.should_quit {
            break;
        }

        // (3) Render unconditionally.
        terminal.draw(|frame| crate::tui::screens::render(frame, state))?;

        // (4) Poll terminal input; translate to a Message (or Tick on timeout).
        let message = poll_input()?;
        // Best-effort send; the channel is only saturated under pathological
        // backpressure, which this loop cannot produce (one message/tick).
        let _ = tx.try_send(message);
    }

    Ok(())
}

/// Apply runner-side effects from an update cycle.
///
/// `Quit` is handled synchronously via `state.should_quit` (already set by
/// `update`). All async actions are dispatched onto `tokio::task::spawn_blocking`
/// tasks; results are sent back over a clone of `tx`.
fn apply_actions(state: &mut AppState, actions: Vec<UpdateAction>, tx: mpsc::Sender<Message>) {
    for action in actions {
        match action {
            UpdateAction::Quit => {
                state.should_quit = true;
            }
            UpdateAction::RefreshStatus => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let state = crate::server::status();
                    let msg = Message::StatusLoaded(state);
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::StartServer => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = match crate::server::start_daemon() {
                        Ok(()) => {
                            // Brief wait so the daemon has a chance to start.
                            std::thread::sleep(Duration::from_millis(500));
                            // Re-query status so the UI reflects the new state.
                            let st = crate::server::status();
                            Message::StatusLoaded(st)
                        }
                        Err(e) => Message::ActionFailed(e.to_string()),
                    };
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::StopServer => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = match crate::server::stop() {
                        Ok(()) => Message::StatusLoaded(None),
                        Err(e) => Message::ActionFailed(e.to_string()),
                    };
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::LoadConfig => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = load_config_snapshot();
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::SaveBind(addr) => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = match crate::server::set_bind_addr(&addr) {
                        Ok(()) => Message::ActionOk(format!("Saved bind address: {addr}")),
                        Err(e) => Message::ActionFailed(e.to_string()),
                    };
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::EnsureCert(sans) => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = ensure_cert_task(&sans);
                    let _ = tx.blocking_send(msg);
                });
            }

            // ── Token management ──────────────────────────────────────────────
            UpdateAction::LoadTokens => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = match crate::server::tokens::list() {
                        Ok(tokens) => Message::TokensLoaded(tokens),
                        Err(e) => Message::ActionFailed(format!("list tokens failed: {e}")),
                    };
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::CreateToken { name, read_only } => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = match crate::server::tokens::create(name, read_only) {
                        Ok((token, name)) => Message::TokenCreated { token, name },
                        Err(e) => Message::ActionFailed(format!("create token failed: {e}")),
                    };
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::RevokeToken(name) => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = match crate::server::tokens::revoke(&name) {
                        Ok(_) => Message::TokensChanged,
                        Err(e) => Message::ActionFailed(format!("revoke token failed: {e}")),
                    };
                    let _ = tx.blocking_send(msg);
                });
            }

            // ── Dashboard ─────────────────────────────────────────────────────
            UpdateAction::LoadCertInfo => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = load_cert_info_task();
                    let _ = tx.blocking_send(msg);
                });
            }

            // ── Pairing ───────────────────────────────────────────────────────
            UpdateAction::StartPairing { read_only, seq } => {
                let tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    let msg = start_pairing_task(read_only, seq);
                    let _ = tx.blocking_send(msg);
                });
            }
            UpdateAction::RevokePairingToken(name) => {
                // Best-effort tidy-up of a superseded / abandoned pairing token.
                // No result is posted back — this is fire-and-forget.
                tokio::task::spawn_blocking(move || {
                    if let Err(e) = crate::server::tokens::revoke(&name) {
                        log::warn!("pairing: failed to revoke pending token {name:?}: {e}");
                    }
                });
            }
        }
    }
}

/// Load config + reachable IPs and return the appropriate `Message`.
fn load_config_snapshot() -> Message {
    use crate::app::message::ConfigSnapshot;

    let cfg = match crate::server::effective_config() {
        Ok(c) => c,
        Err(e) => return Message::ActionFailed(format!("Config load failed: {e}")),
    };
    let reachable_ips = crate::pairing::net::reachable_ipv4();

    Message::ConfigLoaded(ConfigSnapshot {
        bind_addr: cfg.bind_addr,
        cert_dir: cfg.cert_dir.display().to_string(),
        reachable_ips,
    })
}

/// Read the persisted cert fingerprint + SAN sidecar **without** generating anything.
///
/// Calls only the read-only facade functions `current_cert_fingerprint()` and
/// `persisted_extra_sans()`.  Never calls `ensure_cert` / `load_or_generate_identity`.
fn load_cert_info_task() -> Message {
    let fingerprint = match crate::server::current_cert_fingerprint() {
        Ok(fp) => fp,
        Err(e) => return Message::ActionFailed(format!("cert info: {e}")),
    };
    let sans = crate::server::persisted_extra_sans();
    Message::CertInfoLoaded { fingerprint, sans }
}

/// Ensure (or regenerate) the TLS cert and return the appropriate `Message`.
fn ensure_cert_task(sans: &[crate::app::state::San]) -> Message {
    let cert_pem = match crate::server::ensure_cert(sans) {
        Ok(p) => p,
        Err(e) => return Message::ActionFailed(format!("Cert generation failed: {e}")),
    };
    let fingerprint = match crate::server::cert_fingerprint(&cert_pem) {
        Ok(fp) => fp,
        Err(e) => return Message::ActionFailed(format!("Fingerprint failed: {e}")),
    };
    // Collect active SANs as display strings.
    let active_sans: Vec<String> = sans.iter().map(|s| s.value().to_string()).collect();

    Message::CertEnsured {
        fingerprint,
        sans: active_sans,
    }
}

/// Build the pairing QR URI and return the appropriate `Message`.
///
/// Steps (all in `spawn_blocking`):
/// 1. Guard: the server must be **running** (we pin the cert it serves) — else
///    `PairingFailed("Start the server first")`.
/// 2. Read host + port from `effective_config().bind_addr`.
/// 3. Pick the advertise host: prefer the configured bind host when it is a
///    concrete (non-loopback, non-unspecified) address; otherwise fall back to
///    the first reachable non-loopback IPv4.
/// 4. **Read the persisted cert fingerprint without regenerating** (the running
///    server serves the cert it loaded at startup — regenerating here would pin
///    a fingerprint it never presents). No cert yet → fail with guidance.
/// 5. Mint a fresh token `pair-<seq>-<unix_millis>` (collision-free across quick
///    regenerates).
/// 6. Build `PairingParams{...}.to_uri()`; capture `client_count` as baseline.
fn start_pairing_task(read_only: bool, seq: u64) -> Message {
    use crate::pairing::payload::PairingParams;

    // 1. Guard: the server must be running so the QR pins the cert it serves.
    if !crate::server::is_running() {
        return Message::PairingFailed {
            err: "Start the server first.".to_string(),
            seq,
        };
    }

    // 2. Effective config (for bind addr).
    let cfg = match crate::server::effective_config() {
        Ok(c) => c,
        Err(e) => {
            return Message::PairingFailed {
                err: format!("config: {e}"),
                seq,
            };
        }
    };

    // 3. Parse host + port from bind_addr.
    let (bind_host, port) = {
        let addr = &cfg.bind_addr;
        if let Some(colon) = addr.rfind(':') {
            let h = addr[..colon].to_string();
            let p: u16 = addr[colon + 1..].parse().unwrap_or(50051);
            (h, p)
        } else {
            (addr.clone(), 50051)
        }
    };

    // 4. Pick the advertise host (prefer the configured bind IP).
    let advertise_host = choose_advertise_host(&bind_host);

    // 5. Pin the fingerprint of the cert the running server actually serves.
    //    NEVER regenerate here — that would pin a fingerprint the server won't
    //    present (review Critical #1).
    let fingerprint = match crate::server::current_cert_fingerprint() {
        Ok(Some(fp)) => fp,
        Ok(None) => {
            return Message::PairingFailed {
                err: "No certificate yet — open the Cert screen and generate one first."
                    .to_string(),
                seq,
            };
        }
        Err(e) => {
            return Message::PairingFailed {
                err: format!("cert fingerprint: {e}"),
                seq,
            };
        }
    };

    // 6. Mint a fresh pairing token with a collision-free name.
    let unix_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let token_name = format!("pair-{seq}-{unix_millis}");
    let (token_plaintext, token_name) =
        match crate::server::tokens::create(Some(token_name), read_only) {
            Ok(t) => t,
            Err(e) => {
                return Message::PairingFailed {
                    err: format!("mint token: {e}"),
                    seq,
                };
            }
        };

    // 7. Build the URI.
    let uri = PairingParams {
        host: advertise_host.clone(),
        port,
        cert_fp_hex: fingerprint.clone(),
        token: token_plaintext,
        read_only,
        label: "zellimserver".to_string(),
    }
    .to_uri();

    // 8. Capture current client count as baseline.
    let baseline_clients = crate::server::client_count().unwrap_or(0);

    // Build a short fingerprint for display (first 16 hex chars + "…").
    let fingerprint_short = if fingerprint.len() > 16 {
        format!("{}…", &fingerprint[..16])
    } else {
        fingerprint
    };

    Message::PairingReady {
        uri,
        baseline_clients,
        host: advertise_host,
        port,
        fingerprint_short,
        token_name,
        seq,
    }
}

/// Choose the host to advertise in the pairing QR.
///
/// Prefer the user's configured bind host when it is a concrete reachable
/// address (not loopback, not the unspecified `0.0.0.0` / `::` wildcard). Only
/// when the bind host is loopback / unspecified do we fall back to the first
/// reachable non-loopback IPv4 discovered from local interfaces.
fn choose_advertise_host(bind_host: &str) -> String {
    if is_concrete_advertise_host(bind_host) {
        return bind_host.to_string();
    }
    crate::pairing::net::reachable_ipv4()
        .into_iter()
        .next()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| bind_host.to_string())
}

/// Whether `host` is a concrete address worth advertising directly — i.e. it
/// parses as an IP that is neither loopback nor unspecified. Non-IP hosts (DNS
/// names) are treated as concrete (the user typed something specific).
fn is_concrete_advertise_host(host: &str) -> bool {
    let h = host.trim();
    if h.is_empty() {
        return false;
    }
    match h.parse::<std::net::IpAddr>() {
        Ok(ip) => !ip.is_loopback() && !ip.is_unspecified(),
        // A DNS name (e.g. "server.local") is a deliberate choice — keep it.
        Err(_) => true,
    }
}

/// Poll crossterm for input for one [`TICK`]; return the resulting message.
///
/// A key **press** becomes [`Message::Key`]; the poll timeout (or any
/// non-key/non-press event) becomes [`Message::Tick`].
fn poll_input() -> Result<Message> {
    if event::poll(TICK)?
        && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        // Only act on Press to avoid double-firing on terminals that report
        // key release/repeat (crossterm "kitty" enhanced reporting).
        return Ok(Message::Key(key));
    }
    Ok(Message::Tick)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concrete_host_keeps_configured_bind_ip() {
        // A user-configured LAN IP must be advertised as-is (review Major #5).
        assert!(is_concrete_advertise_host("192.168.1.50"));
        assert_eq!(choose_advertise_host("192.168.1.50"), "192.168.1.50");
    }

    #[test]
    fn dns_bind_host_is_treated_as_concrete() {
        assert!(is_concrete_advertise_host("server.local"));
        assert_eq!(choose_advertise_host("server.local"), "server.local");
    }

    #[test]
    fn loopback_and_unspecified_are_not_concrete() {
        // These force the fall-back to a reachable interface IP.
        assert!(!is_concrete_advertise_host("127.0.0.1"));
        assert!(!is_concrete_advertise_host("0.0.0.0"));
        assert!(!is_concrete_advertise_host("::"));
        assert!(!is_concrete_advertise_host("::1"));
        assert!(!is_concrete_advertise_host(""));
    }

    #[test]
    fn unspecified_bind_falls_back_to_reachable_or_self() {
        // With an unspecified bind host, choose_advertise_host returns either a
        // discovered reachable IP or (if none) the original host — never panics,
        // and never returns the wildcard when a reachable IP exists.
        let chosen = choose_advertise_host("0.0.0.0");
        if let Some(ip) = crate::pairing::net::reachable_ipv4().into_iter().next() {
            assert_eq!(chosen, ip.to_string());
        } else {
            assert_eq!(chosen, "0.0.0.0");
        }
    }
}
