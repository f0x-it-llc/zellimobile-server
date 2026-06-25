//! cli — clap CLI definitions for `zellimserver`.
//!
//! All subcommands are defined here; the `src/bin/zellimserver.rs` entrypoint
//! parses these and dispatches to the appropriate handler.

use clap::{Args, Parser, Subcommand};

/// zellimserver — gRPC server for Zellij remote control.
///
/// Run `zellimserver init` first to generate the TLS certificate, then
/// `zellimserver start` to serve.  Use `zellimserver help <subcommand>` for
/// details on any subcommand.
#[derive(Debug, Parser)]
#[command(name = "zellimserver", version, about, long_about = None)]
pub struct Cli {
    /// Override the bind address (e.g. `0.0.0.0:50051`).
    /// Takes precedence over the config file and ZELLIMSERVER_BIND env var.
    #[arg(long, global = true, value_name = "ADDR:PORT")]
    pub bind: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Ensure the data dir exists and generate the self-signed TLS cert+key.
    ///
    /// Safe to re-run (idempotent): if the cert already exists AND covers the
    /// requested SANs, it is left unchanged.  Pass `--san` to add extra Subject
    /// Alternative Names (e.g. a LAN IP) so the cert is valid for
    /// connections from a phone over the network.  Also reads ZELLIMSERVER_SAN
    /// (comma-separated list of IPs or DNS names).
    Init(InitArgs),

    /// Create a new API token and print it once (store it — it won't be shown again).
    #[command(name = "create-token")]
    CreateToken(CreateTokenArgs),

    /// List all tokens (name, created_at, read_only).
    #[command(name = "list-tokens")]
    ListTokens,

    /// Revoke a token by name.
    #[command(name = "revoke-token")]
    RevokeToken(RevokeTokenArgs),

    /// Manage the server configuration file.
    ///
    /// Without flags: ensure the config file exists at its default location.
    /// With `--show`: print the EFFECTIVE config (after applying env + flags).
    Config(ConfigArgs),

    /// Start the TLS gRPC server.
    ///
    /// The server reads the effective config (flags > env > file > defaults).
    /// Without `--daemonize` it runs in the foreground; with it, the process
    /// detaches (pidfile + logfile under the data dir) and the command returns
    /// immediately.  Either way a control socket is opened for `status`/`stop`.
    Start(StartArgs),

    /// Report whether a server is running (via the control socket).
    ///
    /// Prints "running" + {version, bind, pid, uptime} if a daemon answers,
    /// otherwise "stopped" (cleaning up a stale pidfile/socket if found).
    Status,

    /// Stop a running server.
    ///
    /// Sends `Shutdown` over the control socket; falls back to the pidfile +
    /// SIGTERM if the socket is unresponsive.  Cleans up the socket + pidfile.
    Stop,
}

// ── init ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Extra Subject Alternative Name(s) to include in the TLS cert.
    ///
    /// Repeatable: `--san 100.64.0.5 --san myhost.example.com`.  Each value is
    /// treated as an IP address if it parses as one, otherwise as a DNS name.
    /// The cert always includes `127.0.0.1` and `localhost`; extras are added
    /// on top.  Also reads ZELLIMSERVER_SAN (comma-separated env var).
    ///
    /// If the on-disk cert does not cover all requested SANs it is regenerated.
    #[arg(long, value_name = "HOST_OR_IP", action = clap::ArgAction::Append)]
    pub san: Vec<String>,

    /// Path to an external TLS certificate PEM file to use instead of the
    /// auto-generated self-signed cert.
    ///
    /// Must be paired with `--tls-key`.  The file is validated to exist and be
    /// readable.  Also reads ZELLIMSERVER_TLS_CERT env var when absent.
    ///
    /// When provided, the self-signed cert is neither generated nor overwritten.
    #[arg(long, value_name = "PATH")]
    pub tls_cert: Option<std::path::PathBuf>,

    /// Path to the private key PEM file that corresponds to `--tls-cert`.
    ///
    /// Must be paired with `--tls-cert`.  Also reads ZELLIMSERVER_TLS_KEY env
    /// var when absent.
    #[arg(long, value_name = "PATH")]
    pub tls_key: Option<std::path::PathBuf>,

    /// Validate the setup for h2c (plaintext HTTP/2) mode — no cert is needed.
    ///
    /// **WARNING:** This mode sends all data — including API tokens and terminal
    /// output — over a PLAINTEXT connection.  It MUST only be used when the
    /// server is sitting behind a trusted TLS-terminating reverse proxy (e.g.
    /// Traefik with a Let's Encrypt cert, Cloudflare origin proxy).  NEVER
    /// expose a server in h2c mode directly to the internet or an untrusted
    /// network.
    ///
    /// Also enabled when ZELLIMSERVER_H2C env var is set to a truthy value
    /// (non-empty and not "0").  Mutually exclusive with `--tls-cert` /
    /// `--tls-key`.
    #[arg(long)]
    pub insecure_h2c: bool,
}

// ── start ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct StartArgs {
    /// Detach into the background (write a pidfile, redirect logs to the
    /// configured log file) and return immediately.
    #[arg(long)]
    pub daemonize: bool,

    /// Extra Subject Alternative Name(s) to include in the TLS cert.
    ///
    /// Same semantics as `init --san`.  If the on-disk cert does not cover the
    /// requested SANs it is regenerated before the server begins serving.
    /// Also reads ZELLIMSERVER_SAN (comma-separated env var).
    #[arg(long, value_name = "HOST_OR_IP", action = clap::ArgAction::Append)]
    pub san: Vec<String>,

    /// Path to an external TLS certificate PEM file to serve instead of the
    /// auto-generated self-signed cert.
    ///
    /// Must be paired with `--tls-key`.  Mutually exclusive with
    /// `--insecure-h2c`.  Also reads ZELLIMSERVER_TLS_CERT env var when absent.
    ///
    /// Use this when terminating TLS at the server itself with a cert from
    /// Let's Encrypt, a corporate CA, or a Cloudflare Origin CA.
    #[arg(long, value_name = "PATH")]
    pub tls_cert: Option<std::path::PathBuf>,

    /// Path to the private key PEM file that corresponds to `--tls-cert`.
    ///
    /// Must be paired with `--tls-cert`.  Mutually exclusive with
    /// `--insecure-h2c`.  Also reads ZELLIMSERVER_TLS_KEY env var when absent.
    #[arg(long, value_name = "PATH")]
    pub tls_key: Option<std::path::PathBuf>,

    /// Serve UNENCRYPTED HTTP/2 (h2c) instead of TLS.
    ///
    /// **WARNING:** This mode sends all data — including API tokens and terminal
    /// output — over a PLAINTEXT connection.  It MUST only be used when the
    /// server is sitting behind a trusted TLS-terminating reverse proxy (e.g.
    /// Traefik with a Let's Encrypt cert, Cloudflare origin proxy).  NEVER
    /// expose a server in h2c mode directly to the internet or an untrusted
    /// network.
    ///
    /// Also enabled when ZELLIMSERVER_H2C env var is set to a truthy value
    /// (non-empty and not "0").  Mutually exclusive with `--tls-cert` /
    /// `--tls-key`.
    #[arg(long)]
    pub insecure_h2c: bool,

    /// Explicitly acknowledge that `--insecure-h2c` is being used on a
    /// non-loopback address behind a trusted TLS-terminating reverse proxy.
    ///
    /// By default, binding h2c on a non-loopback address (anything other than
    /// `127.0.0.1` / `[::1]`) is refused as a safety measure — plaintext gRPC
    /// must NOT be exposed directly to the network.  Set this flag to confirm
    /// that a TLS-terminating proxy (e.g. Traefik + Let's Encrypt, Cloudflare)
    /// is in front of this server.
    ///
    /// Has no effect unless `--insecure-h2c` is also set.  Also enabled when
    /// `ZELLIMSERVER_H2C_ALLOW_PUBLIC` env var is set to a truthy value
    /// (non-empty and not "0").
    #[arg(long = "i-know-this-is-behind-a-proxy")]
    pub h2c_allow_public: bool,
}

// ── create-token ────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct CreateTokenArgs {
    /// Human-readable name for the token (default: auto-generated).
    #[arg(long, short = 'n', value_name = "NAME")]
    pub name: Option<String>,

    /// If set, the token may only be used for read-only operations.
    #[arg(long)]
    pub read_only: bool,
}

// ── revoke-token ─────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct RevokeTokenArgs {
    /// Name of the token to revoke.
    pub name: String,
}

// ── config ───────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct ConfigArgs {
    /// Print the effective configuration (after env + CLI flag overrides).
    #[arg(long)]
    pub show: bool,
}
