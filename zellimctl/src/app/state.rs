//! Application state (TEA model).
//!
//! Pure data: no ratatui imports. Mutated only by [`super::update::update`].
//! Each screen gets its own sub-struct so the update function can delegate
//! clearly. The `AppState` owns all of them.

use std::net::Ipv4Addr;

use crate::server::tokens::TokenRecord;

// ── App-layer infra mirrors ────────────────────────────────────────────────────
//
// The `app/` layer must stay free of `zellimserver::` types (TEA purity +
// layer boundary). These plain mirrors stand in for the infra types; the
// `server/` facade is the only place that converts to/from the real
// `zellimserver::tls::SanEntry` / `zellimserver::control::StatusInfo`.

/// App-layer mirror of `zellimserver::tls::SanEntry`.
///
/// A Subject Alternative Name carried through the TEA layer as plain strings.
/// The `server/` facade converts this into `zellimserver::tls::SanEntry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum San {
    /// An IP-address SAN (stringified; the facade re-parses it).
    Ip(String),
    /// A DNS-name SAN.
    Dns(String),
}

impl San {
    /// Build a [`San`] from a host string: IP if it parses as one, else DNS.
    pub fn from_host(host: &str) -> Self {
        let h = host.trim();
        if h.parse::<std::net::IpAddr>().is_ok() {
            San::Ip(h.to_string())
        } else {
            San::Dns(h.to_string())
        }
    }

    /// Human-readable value (used for display and as the cert SAN list entry).
    pub fn value(&self) -> &str {
        match self {
            San::Ip(s) | San::Dns(s) => s,
        }
    }
}

/// App-layer mirror of `zellimserver::control::StatusInfo`.
///
/// A plain snapshot of a running server's status with no infra types. The
/// `server/` facade converts `StatusInfo` into this on the way into the TEA
/// layer.
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// The server crate version.
    pub version: String,
    /// The address the server is bound to.
    pub bind_addr: String,
    /// The server process id.
    pub pid: u32,
    /// Seconds the server has been running.
    pub uptime_secs: u64,
    /// Total number of mobile clients currently attached across all sessions.
    pub client_count: usize,
}

// ── Screen navigation ─────────────────────────────────────────────────────────

/// The navigable screens of the control panel.
///
/// Ordering matters: `Tab` / arrow navigation cycles through them in this
/// declaration order (see [`Screen::next`] / [`Screen::prev`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Config,
    Cert,
    Tokens,
    Server,
    Pair,
}

impl Screen {
    /// All screens in navigation order. Drives the dashboard tab list.
    pub const ALL: [Screen; 6] = [
        Screen::Dashboard,
        Screen::Config,
        Screen::Cert,
        Screen::Tokens,
        Screen::Server,
        Screen::Pair,
    ];

    /// Human-readable label for tab lists and panel headings.
    pub fn label(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Config => "Config",
            Screen::Cert => "Cert",
            Screen::Tokens => "Tokens",
            Screen::Server => "Server",
            Screen::Pair => "Pair",
        }
    }

    /// Index of this screen within [`Screen::ALL`].
    pub fn index(self) -> usize {
        Screen::ALL
            .iter()
            .position(|s| *s == self)
            .expect("Screen must be present in Screen::ALL")
    }

    /// The next screen in navigation order, wrapping at the end.
    pub fn next(self) -> Screen {
        let i = self.index();
        Screen::ALL[(i + 1) % Screen::ALL.len()]
    }

    /// The previous screen in navigation order, wrapping at the start.
    pub fn prev(self) -> Screen {
        let i = self.index();
        Screen::ALL[(i + Screen::ALL.len() - 1) % Screen::ALL.len()]
    }
}

// ── Per-screen substates ─────────────────────────────────────────────────────

/// Which field in the Config form is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigField {
    #[default]
    Host,
    Port,
    IpPicker,
}

/// State for the Config screen (bind address form + IP picker).
#[derive(Debug, Clone, Default)]
pub struct ConfigState {
    /// Editable host part of the bind address.
    pub host: String,
    /// Editable port part of the bind address.
    pub port: String,
    /// Non-loopback reachable IPv4 addresses discovered from interfaces.
    pub reachable_ips: Vec<Ipv4Addr>,
    /// Index of the currently highlighted IP in the picker list.
    pub ip_cursor: usize,
    /// The currently focused form field.
    pub focused: ConfigField,
    /// Directory where TLS certs are stored (display only).
    pub cert_dir: String,
    /// Extra advertise SANs from the `ZELLIMSERVER_SAN` env var, merged into the
    /// cert SANs alongside the reachable IPs (e.g. a tailnet IP not visible as a
    /// local interface inside a container). Loaded via `ConfigLoaded`.
    pub advertise_sans: Vec<String>,
    /// Transient status line shown after save / load.
    pub status: String,
    /// True while a LoadConfig or SaveBind task is in flight.
    pub loading: bool,
}

impl ConfigState {
    /// Return the current bind address as `"host:port"`.
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Populate form fields from a resolved bind address string.
    pub fn apply_bind_addr(&mut self, addr: &str) {
        if let Some(colon) = addr.rfind(':') {
            self.host = addr[..colon].to_string();
            self.port = addr[colon + 1..].to_string();
        } else {
            self.host = addr.to_string();
            self.port = "50051".to_string();
        }
    }
}

/// State for the Cert screen.
#[derive(Debug, Clone, Default)]
pub struct CertState {
    /// SHA-256 fingerprint of the current cert (hex, lowercase), or empty if unknown.
    pub fingerprint: String,
    /// SANs currently in the certificate.
    pub sans: Vec<String>,
    /// Transient status line.
    pub status: String,
    /// True while EnsureCert is in flight.
    pub loading: bool,
}

// ── Tokens screen state ───────────────────────────────────────────────────────

/// The phase of the token create mini-form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TokensFormPhase {
    /// The list is shown; user browses or initiates create/revoke.
    #[default]
    Browsing,
    /// The "create" form is open; user types a name.
    Creating,
}

/// State for the Tokens screen.
#[derive(Debug, Clone, Default)]
pub struct TokensState {
    /// All tokens currently in the DB.
    pub tokens: Vec<TokenRecord>,
    /// Index of the highlighted row.
    pub cursor: usize,
    /// True while a load / create / revoke task is in flight.
    pub loading: bool,
    /// Transient status / error line.
    pub status: String,
    /// The one-time minted secret (shown after creation until next action).
    pub last_minted_secret: Option<(String, String)>, // (name, plaintext)
    /// Current phase of the mini create-form.
    pub form_phase: TokensFormPhase,
    /// Text typed into the "name" field while creating.
    pub form_name: String,
    /// Read-only toggle in the create form.
    pub form_read_only: bool,
}

impl TokensState {
    /// The selected token name, if any.
    pub fn selected_name(&self) -> Option<&str> {
        self.tokens.get(self.cursor).map(|t| t.name.as_str())
    }
}

// ── Pairing screen state ──────────────────────────────────────────────────────

/// Phase of the pairing QR flow.
#[derive(Debug, Clone, Default)]
pub enum PairingPhase {
    /// Not started yet (or was reset).
    #[default]
    Idle,
    /// Generation is in progress (task is in flight).
    Generating,
    /// QR is ready; displaying the code and waiting for a client to connect.
    Showing {
        uri: String,
        baseline_clients: usize,
        host: String,
        port: u16,
        fingerprint_short: String,
        /// Name of the pairing token minted for this QR (so it can be revoked
        /// on supersede / leave).
        token_name: String,
    },
    /// A new client connected (client_count > baseline).
    Connected,
    /// Generation failed with an error.
    Failed { err: String },
}

/// State for the Pairing screen.
#[derive(Debug, Clone, Default)]
pub struct PairingState {
    /// Current phase of the pairing state machine.
    pub phase: PairingPhase,
    /// Process-monotonic sequence counter for seq-guarded state transitions.
    ///
    /// Incremented each time a new `StartPairing` action is dispatched.
    /// Async results carry the seq they were started with; stale results
    /// (seq < current) are discarded.
    pub seq: u64,
    /// Whether the freshly-minted pairing token should be read-only.
    pub read_only: bool,
    /// Tick counter used for the ~1 s status poll cadence on this screen.
    pub tick_counter: u32,
    /// Name of the currently-pending pairing token (minted but not yet
    /// superseded/revoked). Carried so a regenerate or screen-leave can revoke
    /// the prior token before/while minting a new one. `None` when no token is
    /// outstanding.
    pub pending_token_name: Option<String>,
}

/// State for the Server panel screen.
#[derive(Debug, Clone, Default)]
pub struct ServerPanelState {
    /// Last-known server status; `None` means not yet fetched.
    pub status: Option<ServerInfo>,
    /// True if we know the server is not running (Stopped).
    pub stopped: bool,
    /// True while a start/stop/refresh task is in flight.
    pub loading: bool,
    /// Transient message shown after start/stop actions.
    pub action_msg: String,
    /// Tick counter used to drive the ~1 s live poll cadence.
    pub tick_counter: u32,
}

// ── Root app state ────────────────────────────────────────────────────────────

/// The TEA model: the entire UI is a pure function of this state.
#[derive(Debug)]
pub struct AppState {
    /// The currently visible screen.
    pub screen: Screen,
    /// Set to break the runner's event loop and restore the terminal.
    pub should_quit: bool,
    /// Config screen state.
    pub config: ConfigState,
    /// Cert screen state.
    pub cert: CertState,
    /// Server panel screen state.
    pub server: ServerPanelState,
    /// Tokens screen state.
    pub tokens: TokensState,
    /// Pairing screen state.
    pub pairing: PairingState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Dashboard,
            should_quit: false,
            config: ConfigState::default(),
            cert: CertState::default(),
            server: ServerPanelState::default(),
            tokens: TokensState::default(),
            pairing: PairingState::default(),
        }
    }
}

impl AppState {
    /// Construct the initial state (lands on the Dashboard).
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a printable key should be routed to a focused text field rather
    /// than triggering global / screen-level single-key shortcuts.
    ///
    /// True when:
    /// - the Config screen has the Host or Port field focused, or
    /// - the Tokens screen is in the `Creating` (name-entry) form phase.
    ///
    /// While editing, only `Ctrl-C` quits; save/reload move to `Ctrl-S` /
    /// `Ctrl-R` so common hostnames containing `q`/`s`/`r` can be typed.
    pub fn is_text_editing(&self) -> bool {
        match self.screen {
            Screen::Config => matches!(self.config.focused, ConfigField::Host | ConfigField::Port),
            Screen::Tokens => self.tokens.form_phase == TokensFormPhase::Creating,
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_screens_have_distinct_labels() {
        for s in Screen::ALL {
            assert!(!s.label().is_empty());
        }
    }

    #[test]
    fn next_wraps_around() {
        assert_eq!(Screen::Pair.next(), Screen::Dashboard);
        assert_eq!(Screen::Dashboard.next(), Screen::Config);
    }

    #[test]
    fn prev_wraps_around() {
        assert_eq!(Screen::Dashboard.prev(), Screen::Pair);
        assert_eq!(Screen::Config.prev(), Screen::Dashboard);
    }

    #[test]
    fn index_matches_all_order() {
        for (i, s) in Screen::ALL.iter().enumerate() {
            assert_eq!(s.index(), i);
        }
    }

    #[test]
    fn default_state_lands_on_dashboard() {
        let state = AppState::new();
        assert_eq!(state.screen, Screen::Dashboard);
        assert!(!state.should_quit);
    }

    #[test]
    fn config_state_apply_bind_addr() {
        let mut cs = ConfigState::default();
        cs.apply_bind_addr("0.0.0.0:50051");
        assert_eq!(cs.host, "0.0.0.0");
        assert_eq!(cs.port, "50051");
    }

    #[test]
    fn config_state_apply_bind_addr_ipv6() {
        let mut cs = ConfigState::default();
        cs.apply_bind_addr("127.0.0.1:9090");
        assert_eq!(cs.host, "127.0.0.1");
        assert_eq!(cs.port, "9090");
    }

    #[test]
    fn config_state_bind_addr_roundtrip() {
        let mut cs = ConfigState::default();
        cs.apply_bind_addr("192.168.1.5:50051");
        assert_eq!(cs.bind_addr(), "192.168.1.5:50051");
    }
}
