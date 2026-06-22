//! The TEA update function: `(state, message) -> actions`.
//!
//! Pure with respect to I/O: it mutates [`AppState`] in place and returns the
//! side effects ([`UpdateAction`]s) the runner should perform. No ratatui,
//! terminal, or async code here — that keeps the `app/` layer unit-testable
//! without a terminal.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::action::UpdateAction;
use super::message::Message;
use super::state::{AppState, ConfigField, PairingPhase, San, Screen, TokensFormPhase};

/// Apply a [`Message`] to the [`AppState`], returning any side effects.
pub fn update(state: &mut AppState, message: Message) -> Vec<UpdateAction> {
    match message {
        Message::Key(key) => handle_key(state, key),
        Message::Tick => handle_tick(state),
        Message::Quit => {
            state.should_quit = true;
            vec![UpdateAction::Quit]
        }
        Message::NavTo(screen) => transition_to(state, screen),
        // ── Async task results ────────────────────────────────────────────────
        Message::StatusLoaded(info) => {
            let client_count = info.as_ref().map(|i| i.client_count);
            match info {
                Some(info) => {
                    state.server.status = Some(info);
                    state.server.stopped = false;
                }
                None => {
                    state.server.status = None;
                    state.server.stopped = true;
                }
            }
            state.server.loading = false;
            // Drive pairing connection detection — only on the Pair screen.
            if state.screen == Screen::Pair
                && let Some(n) = client_count
            {
                check_pairing_connection(state, n);
            }
            Vec::new()
        }
        Message::ConfigLoaded(snapshot) => {
            state.config.apply_bind_addr(&snapshot.bind_addr);
            state.config.cert_dir = snapshot.cert_dir;
            state.config.reachable_ips = snapshot.reachable_ips;
            if state.config.ip_cursor >= state.config.reachable_ips.len() {
                state.config.ip_cursor = 0;
            }
            state.config.loading = false;
            state.config.status = "Config loaded.".to_string();
            Vec::new()
        }
        Message::CertEnsured { fingerprint, sans } => {
            state.cert.fingerprint = fingerprint;
            state.cert.sans = sans;
            state.cert.loading = false;
            state.cert.status = "Certificate ready.".to_string();
            Vec::new()
        }
        Message::ActionFailed(msg) => {
            // Surface the error on the active screen's status line.
            match state.screen {
                Screen::Config => state.config.status = format!("Error: {msg}"),
                Screen::Cert => state.cert.status = format!("Error: {msg}"),
                Screen::Server => state.server.action_msg = format!("Error: {msg}"),
                Screen::Tokens => state.tokens.status = format!("Error: {msg}"),
                _ => {}
            }
            // Clear loading flags.
            state.config.loading = false;
            state.cert.loading = false;
            state.server.loading = false;
            state.tokens.loading = false;
            Vec::new()
        }
        Message::ActionOk(msg) => {
            match state.screen {
                Screen::Config => state.config.status = msg,
                Screen::Cert => state.cert.status = msg,
                Screen::Server => state.server.action_msg = msg,
                Screen::Tokens => state.tokens.status = msg,
                _ => {}
            }
            state.config.loading = false;
            state.cert.loading = false;
            state.server.loading = false;
            state.tokens.loading = false;
            Vec::new()
        }

        // ── Token screen messages ─────────────────────────────────────────────
        Message::TokensLoaded(records) => {
            state.tokens.tokens = records;
            // Clamp cursor to valid range.
            if state.tokens.cursor >= state.tokens.tokens.len() {
                state.tokens.cursor = state.tokens.tokens.len().saturating_sub(1);
            }
            state.tokens.loading = false;
            state.tokens.status = "Tokens loaded.".to_string();
            Vec::new()
        }
        Message::TokenCreated { token, name } => {
            state.tokens.last_minted_secret = Some((name, token));
            state.tokens.loading = false;
            state.tokens.form_phase = TokensFormPhase::Browsing;
            state.tokens.form_name = String::new();
            // Trigger a refresh of the list.
            vec![UpdateAction::LoadTokens]
        }
        Message::TokensChanged => {
            state.tokens.loading = false;
            state.tokens.last_minted_secret = None;
            vec![UpdateAction::LoadTokens]
        }

        // ── Pairing screen messages ───────────────────────────────────────────
        Message::PairingReady {
            uri,
            baseline_clients,
            host,
            port,
            fingerprint_short,
            token_name,
            seq,
        } => {
            // Accept only a current-attempt result that arrives while the Pair
            // screen is still active. A seq mismatch (superseded attempt, or one
            // invalidated by leaving the screen) OR a result arriving off-screen
            // means the minted token is orphaned — revoke it rather than
            // resurrecting pairing state with an un-revoked bearer secret.
            if seq == state.pairing.seq && state.screen == Screen::Pair {
                state.pairing.pending_token_name = Some(token_name.clone());
                state.pairing.phase = PairingPhase::Showing {
                    uri,
                    baseline_clients,
                    host,
                    port,
                    fingerprint_short,
                    token_name,
                };
            } else {
                return vec![UpdateAction::RevokePairingToken(token_name)];
            }
            Vec::new()
        }
        Message::PairingFailed { err, seq } => {
            if seq == state.pairing.seq {
                state.pairing.phase = PairingPhase::Failed { err };
            }
            Vec::new()
        }
    }
}

/// Transition `state.screen` to `next`, running any leave + enter side effects.
///
/// Used by every navigation path (NavTo, Tab/BackTab, arrow nav) so the
/// leave-side cleanup (e.g. revoking an unused pairing token) is centralised and
/// can never be skipped by one path.
fn transition_to(state: &mut AppState, next: Screen) -> Vec<UpdateAction> {
    let mut actions = on_leave_screen(state, next);
    actions.extend(on_enter_screen(state, next));
    state.screen = next;
    actions
}

/// Side effects to run when leaving the current screen for `next`.
///
/// Leaving the Pair screen with an unused pairing token outstanding revokes it
/// (a bearer secret must not linger after the user navigates away) and resets
/// the pairing phase to `Idle`.
fn on_leave_screen(state: &mut AppState, next: Screen) -> Vec<UpdateAction> {
    let mut actions = Vec::new();
    if state.screen == Screen::Pair && next != Screen::Pair {
        // Revoke the pending pairing token if one is outstanding. The token name
        // lives both in `pending_token_name` (set as soon as the QR is ready)
        // and in the `Showing` phase; prefer the phase value when present so the
        // two can never drift, falling back to `pending_token_name`.
        let from_phase = match &state.pairing.phase {
            PairingPhase::Showing { token_name, .. } => Some(token_name.clone()),
            _ => None,
        };
        if let Some(name) = from_phase.or_else(|| state.pairing.pending_token_name.clone()) {
            actions.push(UpdateAction::RevokePairingToken(name));
        }
        state.pairing.pending_token_name = None;
        state.pairing.phase = PairingPhase::Idle;
        // Invalidate any in-flight pairing attempt (phase was `Generating`, no
        // token minted yet) so its `PairingReady` is treated as stale and the
        // token it mints is revoked — never resurrected off-screen. Closes the
        // leave-during-Generating race.
        state.pairing.seq = state.pairing.seq.wrapping_add(1);
    }
    actions
}

/// Actions to dispatch when entering a screen (load initial data).
fn on_enter_screen(state: &mut AppState, screen: Screen) -> Vec<UpdateAction> {
    match screen {
        Screen::Config => {
            state.config.loading = true;
            state.config.status = String::new();
            vec![UpdateAction::LoadConfig]
        }
        Screen::Server => {
            state.server.loading = true;
            state.server.action_msg = String::new();
            state.server.tick_counter = 0;
            vec![UpdateAction::RefreshStatus]
        }
        Screen::Tokens => {
            state.tokens.loading = true;
            state.tokens.status = String::new();
            state.tokens.last_minted_secret = None;
            state.tokens.form_phase = TokensFormPhase::Browsing;
            state.tokens.form_name = String::new();
            vec![UpdateAction::LoadTokens]
        }
        Screen::Pair => {
            // Reset pairing state when entering the screen.
            state.pairing.phase = PairingPhase::Idle;
            state.pairing.tick_counter = 0;
            state.pairing.pending_token_name = None;
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// Tick handler — drives the live poll for the Server screen and pairing detection.
fn handle_tick(state: &mut AppState) -> Vec<UpdateAction> {
    let mut actions = Vec::new();

    if state.screen == Screen::Server && !state.server.loading {
        // Emit RefreshStatus roughly every 20 ticks (~1 s at 50 ms/tick).
        state.server.tick_counter = state.server.tick_counter.wrapping_add(1);
        if state.server.tick_counter.is_multiple_of(20) {
            state.server.loading = true;
            actions.push(UpdateAction::RefreshStatus);
        }
    }

    if state.screen == Screen::Pair {
        // Poll status while showing a QR so we can detect when a client connects.
        if let PairingPhase::Showing { .. } = &state.pairing.phase {
            state.pairing.tick_counter = state.pairing.tick_counter.wrapping_add(1);
            if state.pairing.tick_counter.is_multiple_of(20) {
                actions.push(UpdateAction::RefreshStatus);
            }
        }
    }

    actions
}

/// Promote the pairing phase to `Connected` if a new client appeared.
///
/// Called from the `StatusLoaded` handler — only while we are on the Pair screen
/// and the phase is `Showing`. The rise is a heuristic (the attached-client
/// count went up), not verified per-token auth.
pub(crate) fn check_pairing_connection(state: &mut AppState, current_clients: usize) {
    if let PairingPhase::Showing {
        baseline_clients, ..
    } = &state.pairing.phase
        && current_clients > *baseline_clients
    {
        state.pairing.phase = PairingPhase::Connected;
    }
}

/// Translate a raw key event into state changes / actions.
fn handle_key(state: &mut AppState, key: KeyEvent) -> Vec<UpdateAction> {
    // Ctrl-C always quits.
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        state.should_quit = true;
        return vec![UpdateAction::Quit];
    }

    // `q` quits from any screen — UNLESS a text field is being edited (Config
    // host/port or the Tokens create-name form), where `q` is a literal char.
    // `Ctrl-C` (handled above) remains the always-available quit while editing.
    if key.code == KeyCode::Char('q') && !state.is_text_editing() {
        state.should_quit = true;
        return vec![UpdateAction::Quit];
    }

    // Screen-level Tab / BackTab navigation (always available).
    match key.code {
        KeyCode::Tab => {
            let next = state.screen.next();
            return transition_to(state, next);
        }
        KeyCode::BackTab => {
            let prev = state.screen.prev();
            return transition_to(state, prev);
        }
        _ => {}
    }

    // Per-screen key handling.
    match state.screen {
        Screen::Config => handle_config_key(state, key),
        Screen::Cert => handle_cert_key(state, key),
        Screen::Server => handle_server_key(state, key),
        Screen::Tokens => handle_tokens_key(state, key),
        Screen::Pair => handle_pair_key(state, key),
        Screen::Dashboard => handle_nav_keys(state, key),
    }
}

/// Navigation-only key handler for the Dashboard screen.
fn handle_nav_keys(state: &mut AppState, key: KeyEvent) -> Vec<UpdateAction> {
    match key.code {
        KeyCode::Right | KeyCode::Down => {
            let next = state.screen.next();
            transition_to(state, next)
        }
        KeyCode::Left | KeyCode::Up => {
            let prev = state.screen.prev();
            transition_to(state, prev)
        }
        _ => Vec::new(),
    }
}

/// Key handler for the Config screen.
///
/// Fields: host (text) → port (text) → IP picker (list).
/// - Tab / Down / Right: move to next field.
/// - BackTab / Up / Left: move to prev field (except in picker where Up scrolls).
/// - In IpPicker: Up/Down scroll, Enter selects IP into host.
///
/// While editing the **Host** or **Port** text fields, printable keys (including
/// `s`/`r`/`q`) are typed literally; save/reload move to `Ctrl-S` / `Ctrl-R` so
/// hostnames like `server.local` can be entered. In the **IpPicker** (a list,
/// not a text field), the bare `s` / `r` shortcuts still apply.
fn handle_config_key(state: &mut AppState, key: KeyEvent) -> Vec<UpdateAction> {
    // While editing a text field, Ctrl-S saves and Ctrl-R reloads.
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(state.config.focused, ConfigField::Host | ConfigField::Port)
    {
        match key.code {
            KeyCode::Char('s') => return config_save(state),
            KeyCode::Char('r') => return config_reload(state),
            _ => {}
        }
    }

    match state.config.focused {
        ConfigField::Host => match key.code {
            KeyCode::Tab | KeyCode::Down => {
                state.config.focused = ConfigField::Port;
                Vec::new()
            }
            KeyCode::Up | KeyCode::BackTab => Vec::new(),
            KeyCode::Enter => config_save(state),
            KeyCode::Backspace => {
                state.config.host.pop();
                Vec::new()
            }
            KeyCode::Char(c) => {
                state.config.host.push(c);
                Vec::new()
            }
            _ => Vec::new(),
        },
        ConfigField::Port => match key.code {
            KeyCode::Tab | KeyCode::Down => {
                if !state.config.reachable_ips.is_empty() {
                    state.config.focused = ConfigField::IpPicker;
                }
                Vec::new()
            }
            KeyCode::Up | KeyCode::BackTab => {
                state.config.focused = ConfigField::Host;
                Vec::new()
            }
            KeyCode::Enter => config_save(state),
            KeyCode::Backspace => {
                state.config.port.pop();
                Vec::new()
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                state.config.port.push(c);
                Vec::new()
            }
            _ => Vec::new(),
        },
        ConfigField::IpPicker => match key.code {
            KeyCode::Down => {
                if !state.config.reachable_ips.is_empty() {
                    state.config.ip_cursor =
                        (state.config.ip_cursor + 1) % state.config.reachable_ips.len();
                }
                Vec::new()
            }
            KeyCode::Up => {
                if !state.config.reachable_ips.is_empty() {
                    let len = state.config.reachable_ips.len();
                    state.config.ip_cursor = (state.config.ip_cursor + len - 1) % len;
                }
                Vec::new()
            }
            KeyCode::Tab => {
                state.config.focused = ConfigField::Host;
                Vec::new()
            }
            KeyCode::BackTab => {
                state.config.focused = ConfigField::Port;
                Vec::new()
            }
            KeyCode::Enter => {
                // Select the highlighted IP into the host field.
                if let Some(ip) = state.config.reachable_ips.get(state.config.ip_cursor) {
                    state.config.host = ip.to_string();
                    state.config.focused = ConfigField::Port;
                }
                Vec::new()
            }
            KeyCode::Char('s') => config_save(state),
            KeyCode::Char('r') => config_reload(state),
            _ => Vec::new(),
        },
    }
}

/// Validate the current host/port and dispatch a `SaveBind`, or set a validation
/// error on the status line instead of persisting garbage.
fn config_save(state: &mut AppState) -> Vec<UpdateAction> {
    if state.config.loading {
        return Vec::new();
    }
    let host = state.config.host.trim();
    if host.is_empty() {
        state.config.status = "Error: bind host must not be empty.".to_string();
        return Vec::new();
    }
    if state.config.port.trim().parse::<u16>().is_err() {
        state.config.status = "Error: port must be a number in 1..=65535.".to_string();
        return Vec::new();
    }
    let addr = state.config.bind_addr();
    state.config.loading = true;
    state.config.status = "Saving…".to_string();
    vec![UpdateAction::SaveBind(addr)]
}

/// Dispatch a config reload.
fn config_reload(state: &mut AppState) -> Vec<UpdateAction> {
    if state.config.loading {
        return Vec::new();
    }
    state.config.loading = true;
    state.config.status = "Loading…".to_string();
    vec![UpdateAction::LoadConfig]
}

/// Key handler for the Cert screen.
///
/// - `g`: ensure / regenerate cert using SANs from the config host.
/// - `r`: reload fingerprint only.
fn handle_cert_key(state: &mut AppState, key: KeyEvent) -> Vec<UpdateAction> {
    match key.code {
        KeyCode::Char('g') => {
            if !state.cert.loading {
                state.cert.loading = true;
                state.cert.status = "Generating cert…".to_string();
                // Derive SANs from whatever IP/host is in the config screen.
                let sans = build_sans_from_config(state);
                vec![UpdateAction::EnsureCert(sans)]
            } else {
                Vec::new()
            }
        }
        KeyCode::Char('r') => {
            if !state.cert.loading {
                state.cert.loading = true;
                state.cert.status = "Refreshing…".to_string();
                let sans = build_sans_from_config(state);
                vec![UpdateAction::EnsureCert(sans)]
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Key handler for the Server panel screen.
///
/// - `s`: start daemon.
/// - `x`: stop daemon.
/// - `r`: refresh status.
fn handle_server_key(state: &mut AppState, key: KeyEvent) -> Vec<UpdateAction> {
    match key.code {
        KeyCode::Char('s') => {
            if !state.server.loading {
                state.server.loading = true;
                state.server.stopped = false;
                state.server.action_msg = "Starting server…".to_string();
                vec![UpdateAction::StartServer]
            } else {
                Vec::new()
            }
        }
        KeyCode::Char('x') => {
            if !state.server.loading {
                state.server.loading = true;
                state.server.action_msg = "Stopping server…".to_string();
                vec![UpdateAction::StopServer]
            } else {
                Vec::new()
            }
        }
        KeyCode::Char('r') => {
            if !state.server.loading {
                state.server.loading = true;
                state.server.action_msg = String::new();
                vec![UpdateAction::RefreshStatus]
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Key handler for the Tokens screen.
///
/// Browsing mode:
/// - `j`/`Down`: move cursor down.
/// - `k`/`Up`: move cursor up.
/// - `c`: open the create form.
/// - `d`/`x`: revoke the selected token.
/// - `r`: reload list.
/// - `Esc`: clear the minted secret.
///
/// Creating mode (a text-editing field — see [`AppState::is_text_editing`]):
/// - Char input (including `c`, `q`, Space): type the token name literally.
/// - `Backspace`: delete last char.
/// - `Ctrl-Space`: toggle read-only.
/// - `Enter`: submit the create request.
/// - `Esc`: cancel and return to browsing.
fn handle_tokens_key(state: &mut AppState, key: KeyEvent) -> Vec<UpdateAction> {
    match state.tokens.form_phase {
        TokensFormPhase::Browsing => match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !state.tokens.tokens.is_empty() {
                    state.tokens.cursor = (state.tokens.cursor + 1) % state.tokens.tokens.len();
                }
                Vec::new()
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !state.tokens.tokens.is_empty() {
                    let len = state.tokens.tokens.len();
                    state.tokens.cursor = (state.tokens.cursor + len - 1) % len;
                }
                Vec::new()
            }
            KeyCode::Char('c') => {
                state.tokens.form_phase = TokensFormPhase::Creating;
                state.tokens.form_name = String::new();
                state.tokens.form_read_only = false;
                Vec::new()
            }
            KeyCode::Char('d') | KeyCode::Char('x') => {
                if !state.tokens.loading
                    && let Some(name) = state.tokens.selected_name().map(str::to_string)
                {
                    state.tokens.loading = true;
                    state.tokens.status = format!("Revoking '{name}'…");
                    state.tokens.last_minted_secret = None;
                    return vec![UpdateAction::RevokeToken(name)];
                }
                Vec::new()
            }
            KeyCode::Char('r') => {
                if !state.tokens.loading {
                    state.tokens.loading = true;
                    state.tokens.status = "Refreshing…".to_string();
                    vec![UpdateAction::LoadTokens]
                } else {
                    Vec::new()
                }
            }
            KeyCode::Esc => {
                state.tokens.last_minted_secret = None;
                Vec::new()
            }
            _ => Vec::new(),
        },
        TokensFormPhase::Creating => match key.code {
            KeyCode::Esc => {
                state.tokens.form_phase = TokensFormPhase::Browsing;
                state.tokens.form_name = String::new();
                Vec::new()
            }
            // Enter submits. (Note: `c` is NOT a submit shortcut here — it is a
            // literal character so token names may contain it.)
            KeyCode::Enter => {
                if !state.tokens.loading {
                    let name = state.tokens.form_name.trim().to_string();
                    let name_opt = if name.is_empty() { None } else { Some(name) };
                    let read_only = state.tokens.form_read_only;
                    state.tokens.loading = true;
                    state.tokens.status = "Creating token…".to_string();
                    vec![UpdateAction::CreateToken {
                        name: name_opt,
                        read_only,
                    }]
                } else {
                    Vec::new()
                }
            }
            // Ctrl-Space toggles read-only while editing (a bare Space is a
            // literal character in the name field).
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.tokens.form_read_only = !state.tokens.form_read_only;
                Vec::new()
            }
            KeyCode::Backspace => {
                state.tokens.form_name.pop();
                Vec::new()
            }
            KeyCode::Char(c) => {
                state.tokens.form_name.push(c);
                Vec::new()
            }
            _ => Vec::new(),
        },
    }
}

/// Key handler for the Pair screen.
///
/// - `p`/`Enter`/`g`: start / regenerate pairing (bumps seq, dispatches `StartPairing`).
/// - `Space`: toggle read-only for the next generated token.
/// - `r`: same as `p` (regenerate).
fn handle_pair_key(state: &mut AppState, key: KeyEvent) -> Vec<UpdateAction> {
    match key.code {
        KeyCode::Char('p') | KeyCode::Enter | KeyCode::Char('g') | KeyCode::Char('r') => {
            let mut actions = Vec::new();
            // Revoke the previously-minted pending pairing token before minting a
            // new one, so superseded bearer secrets don't accumulate.
            if let Some(prev) = state.pairing.pending_token_name.take() {
                actions.push(UpdateAction::RevokePairingToken(prev));
            }
            // Bump seq to invalidate any in-flight result.
            state.pairing.seq = state.pairing.seq.wrapping_add(1);
            state.pairing.phase = PairingPhase::Generating;
            state.pairing.tick_counter = 0;
            let seq = state.pairing.seq;
            let read_only = state.pairing.read_only;
            actions.push(UpdateAction::StartPairing { read_only, seq });
            actions
        }
        KeyCode::Char(' ') => {
            state.pairing.read_only = !state.pairing.read_only;
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// Build a SAN list from the host currently entered in the Config screen.
///
/// Returns app-layer [`San`] mirrors (the `server/` facade converts them into
/// infra `SanEntry`). Tries to parse the host as an IP first; falls back to DNS.
fn build_sans_from_config(state: &AppState) -> Vec<San> {
    let host = state.config.host.trim();
    if host.is_empty() {
        return vec![];
    }
    vec![San::from_host(host)]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Screen;
    use crossterm::event::KeyEventKind;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn q_sets_should_quit() {
        let mut state = AppState::new();
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('q'))));
        assert!(state.should_quit);
        assert!(matches!(actions.as_slice(), [UpdateAction::Quit]));
    }

    #[test]
    fn ctrl_c_quits() {
        let mut state = AppState::new();
        let ev = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let actions = update(&mut state, Message::Key(ev));
        assert!(state.should_quit);
        assert!(matches!(actions.as_slice(), [UpdateAction::Quit]));
    }

    #[test]
    fn quit_message_sets_flag() {
        let mut state = AppState::new();
        let actions = update(&mut state, Message::Quit);
        assert!(state.should_quit);
        assert!(matches!(actions.as_slice(), [UpdateAction::Quit]));
    }

    #[test]
    fn tab_advances_screen() {
        let mut state = AppState::new();
        assert_eq!(state.screen, Screen::Dashboard);
        // Tab from Dashboard → Config triggers LoadConfig.
        let actions = update(&mut state, Message::Key(key(KeyCode::Tab)));
        assert_eq!(state.screen, Screen::Config);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::LoadConfig))
        );
    }

    #[test]
    fn backtab_goes_back() {
        let mut state = AppState::new();
        update(&mut state, Message::Key(key(KeyCode::BackTab)));
        assert_eq!(state.screen, Screen::Pair);
    }

    #[test]
    fn nav_to_jumps_directly() {
        let mut state = AppState::new();
        update(&mut state, Message::NavTo(Screen::Tokens));
        assert_eq!(state.screen, Screen::Tokens);
    }

    #[test]
    fn tick_is_noop_on_non_server_screen() {
        let mut state = AppState::new();
        let actions = update(&mut state, Message::Tick);
        assert!(actions.is_empty());
        assert!(!state.should_quit);
    }

    #[test]
    fn tick_triggers_refresh_on_server_screen_after_20() {
        let mut state = AppState::new();
        state.screen = Screen::Server;
        // Drive 19 ticks — no refresh yet.
        for _ in 0..19 {
            let actions = update(&mut state, Message::Tick);
            assert!(actions.is_empty());
        }
        // 20th tick → RefreshStatus.
        let actions = update(&mut state, Message::Tick);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::RefreshStatus))
        );
    }

    #[test]
    fn unhandled_key_kind_still_navigates_on_press() {
        let mut ev = key(KeyCode::Tab);
        ev.kind = KeyEventKind::Press;
        let mut state = AppState::new();
        update(&mut state, Message::Key(ev));
        assert_eq!(state.screen, Screen::Config);
    }

    #[test]
    fn server_s_key_dispatches_start() {
        let mut state = AppState::new();
        state.screen = Screen::Server;
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('s'))));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::StartServer))
        );
    }

    #[test]
    fn server_s_key_clears_stopped_flag() {
        // Regression: pressing `s` (StartServer) while the daemon is known-stopped
        // must clear `stopped` so the renderer doesn't hold a stale "Stopped" panel
        // for the one cycle before the first StatusLoaded(Some(_)) arrives.
        let mut state = AppState::new();
        state.screen = Screen::Server;
        state.server.stopped = true;
        state.server.status = None;
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('s'))));
        assert!(!state.server.stopped, "stopped should be cleared on StartServer");
        assert!(state.server.loading);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::StartServer))
        );
    }

    #[test]
    fn server_x_key_dispatches_stop() {
        let mut state = AppState::new();
        state.screen = Screen::Server;
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('x'))));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::StopServer))
        );
    }

    #[test]
    fn server_r_key_dispatches_refresh() {
        let mut state = AppState::new();
        state.screen = Screen::Server;
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('r'))));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::RefreshStatus))
        );
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn config_ctrl_s_dispatches_save_while_editing() {
        let mut state = AppState::new();
        state.screen = Screen::Config;
        state.config.host = "127.0.0.1".to_string();
        state.config.port = "50051".to_string();
        // Focused on Host (a text field) → Ctrl-S saves.
        let actions = update(&mut state, Message::Key(ctrl(KeyCode::Char('s'))));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::SaveBind(_)))
        );
    }

    #[test]
    fn config_enter_dispatches_save_while_editing() {
        let mut state = AppState::new();
        state.screen = Screen::Config;
        state.config.host = "10.0.0.5".to_string();
        state.config.port = "50051".to_string();
        let actions = update(&mut state, Message::Key(key(KeyCode::Enter)));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::SaveBind(addr) if addr == "10.0.0.5:50051"))
        );
    }

    #[test]
    fn config_host_field_can_type_a_dns_name_and_save() {
        // Regression for review Major #3: 's'/'r'/'q' must be literal characters
        // while editing the host field, so a name like "server.local" can be typed.
        let mut state = AppState::new();
        state.screen = Screen::Config;
        state.config.host = String::new();
        state.config.port = "50051".to_string();
        for c in "server.local".chars() {
            let actions = update(&mut state, Message::Key(key(KeyCode::Char(c))));
            // None of these keystrokes should trigger a side effect.
            assert!(actions.is_empty(), "char {c:?} produced an action");
            assert!(!state.should_quit, "char {c:?} quit the app");
        }
        assert_eq!(state.config.host, "server.local");
        // Now save and confirm the full address is dispatched.
        let actions = update(&mut state, Message::Key(ctrl(KeyCode::Char('s'))));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::SaveBind(addr) if addr == "server.local:50051"))
        );
    }

    #[test]
    fn config_save_rejects_empty_host() {
        let mut state = AppState::new();
        state.screen = Screen::Config;
        state.config.host = String::new();
        state.config.port = "50051".to_string();
        let actions = update(&mut state, Message::Key(ctrl(KeyCode::Char('s'))));
        assert!(actions.is_empty());
        assert!(state.config.status.contains("Error"));
    }

    #[test]
    fn config_save_rejects_bad_port() {
        let mut state = AppState::new();
        state.screen = Screen::Config;
        state.config.host = "127.0.0.1".to_string();
        state.config.port = "not-a-port".to_string();
        let actions = update(&mut state, Message::Key(key(KeyCode::Enter)));
        assert!(actions.is_empty());
        assert!(state.config.status.contains("Error"));
    }

    #[test]
    fn q_is_literal_in_config_host_field() {
        let mut state = AppState::new();
        state.screen = Screen::Config;
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('q'))));
        assert!(!state.should_quit);
        assert!(actions.is_empty());
        assert_eq!(state.config.host, "q");
    }

    #[test]
    fn q_is_literal_in_tokens_create_form() {
        let mut state = AppState::new();
        state.screen = Screen::Tokens;
        state.tokens.form_phase = TokensFormPhase::Creating;
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('q'))));
        assert!(!state.should_quit);
        assert!(actions.is_empty());
        assert_eq!(state.tokens.form_name, "q");
    }

    #[test]
    fn cert_g_key_dispatches_ensure_cert() {
        let mut state = AppState::new();
        state.screen = Screen::Cert;
        state.config.host = "192.168.1.5".to_string();
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('g'))));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::EnsureCert(_)))
        );
    }

    /// Build a test `ServerInfo` with the given client count.
    fn server_info(client_count: usize) -> crate::app::state::ServerInfo {
        crate::app::state::ServerInfo {
            version: "0.1.0".to_string(),
            bind_addr: "127.0.0.1:50051".to_string(),
            pid: 1234,
            uptime_secs: 42,
            client_count,
        }
    }

    #[test]
    fn status_loaded_running_updates_server_state() {
        let mut state = AppState::new();
        update(&mut state, Message::StatusLoaded(Some(server_info(2))));
        assert!(state.server.status.is_some());
        assert!(!state.server.stopped);
        assert_eq!(state.server.status.unwrap().client_count, 2);
    }

    #[test]
    fn status_loaded_none_marks_stopped() {
        let mut state = AppState::new();
        state.server.status = Some(server_info(1));
        update(&mut state, Message::StatusLoaded(None));
        assert!(state.server.status.is_none());
        assert!(state.server.stopped);
    }

    #[test]
    fn config_loaded_populates_fields() {
        use crate::app::message::ConfigSnapshot;
        let mut state = AppState::new();
        let snap = ConfigSnapshot {
            bind_addr: "0.0.0.0:9090".to_string(),
            cert_dir: "/tmp/certs".to_string(),
            reachable_ips: vec![],
        };
        update(&mut state, Message::ConfigLoaded(snap));
        assert_eq!(state.config.host, "0.0.0.0");
        assert_eq!(state.config.port, "9090");
        assert_eq!(state.config.cert_dir, "/tmp/certs");
    }

    #[test]
    fn cert_ensured_updates_cert_state() {
        let mut state = AppState::new();
        update(
            &mut state,
            Message::CertEnsured {
                fingerprint: "ab:cd".to_string(),
                sans: vec!["127.0.0.1".to_string()],
            },
        );
        assert_eq!(state.cert.fingerprint, "ab:cd");
        assert_eq!(state.cert.sans, vec!["127.0.0.1".to_string()]);
    }

    #[test]
    fn action_failed_sets_status_on_config_screen() {
        let mut state = AppState::new();
        state.screen = Screen::Config;
        update(&mut state, Message::ActionFailed("disk full".to_string()));
        assert!(state.config.status.contains("disk full"));
    }

    // ── Tokens screen tests ───────────────────────────────────────────────────

    #[test]
    fn enter_tokens_screen_dispatches_load() {
        let mut state = AppState::new();
        let actions = update(&mut state, Message::NavTo(Screen::Tokens));
        assert_eq!(state.screen, Screen::Tokens);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::LoadTokens))
        );
    }

    #[test]
    fn tokens_c_key_opens_create_form() {
        use crate::app::state::TokensFormPhase;
        let mut state = AppState::new();
        state.screen = Screen::Tokens;
        update(&mut state, Message::Key(key(KeyCode::Char('c'))));
        assert_eq!(state.tokens.form_phase, TokensFormPhase::Creating);
    }

    #[test]
    fn tokens_esc_cancels_create_form() {
        use crate::app::state::TokensFormPhase;
        let mut state = AppState::new();
        state.screen = Screen::Tokens;
        state.tokens.form_phase = TokensFormPhase::Creating;
        state.tokens.form_name = "foo".to_string();
        update(&mut state, Message::Key(key(KeyCode::Esc)));
        assert_eq!(state.tokens.form_phase, TokensFormPhase::Browsing);
        assert!(state.tokens.form_name.is_empty());
    }

    #[test]
    fn tokens_ctrl_space_toggles_read_only_in_creating() {
        use crate::app::state::TokensFormPhase;
        let mut state = AppState::new();
        state.screen = Screen::Tokens;
        state.tokens.form_phase = TokensFormPhase::Creating;
        assert!(!state.tokens.form_read_only);
        // Ctrl-Space toggles; a bare Space is now a literal char in the name.
        update(
            &mut state,
            Message::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL)),
        );
        assert!(state.tokens.form_read_only);
    }

    #[test]
    fn tokens_bare_space_is_literal_in_creating() {
        use crate::app::state::TokensFormPhase;
        let mut state = AppState::new();
        state.screen = Screen::Tokens;
        state.tokens.form_phase = TokensFormPhase::Creating;
        update(&mut state, Message::Key(key(KeyCode::Char(' '))));
        assert_eq!(state.tokens.form_name, " ");
        assert!(!state.tokens.form_read_only);
    }

    #[test]
    fn tokens_loaded_updates_list() {
        use crate::server::tokens::TokenRecord;
        let mut state = AppState::new();
        let records = vec![TokenRecord {
            name: "my-token".to_string(),
            created_at: "2026-01-01".to_string(),
            read_only: false,
        }];
        update(&mut state, Message::TokensLoaded(records));
        assert_eq!(state.tokens.tokens.len(), 1);
        assert_eq!(state.tokens.tokens[0].name, "my-token");
        assert!(!state.tokens.loading);
    }

    #[test]
    fn token_created_stores_secret_and_triggers_reload() {
        let mut state = AppState::new();
        state.screen = Screen::Tokens;
        let actions = update(
            &mut state,
            Message::TokenCreated {
                token: "plaintext-secret".to_string(),
                name: "new-tok".to_string(),
            },
        );
        assert_eq!(
            state.tokens.last_minted_secret,
            Some(("new-tok".to_string(), "plaintext-secret".to_string()))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::LoadTokens))
        );
    }

    // ── Pairing screen tests ──────────────────────────────────────────────────

    #[test]
    fn pair_p_key_bumps_seq_and_dispatches_start() {
        let mut state = AppState::new();
        state.screen = Screen::Pair;
        let initial_seq = state.pairing.seq;
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('p'))));
        assert_eq!(state.pairing.seq, initial_seq + 1);
        assert!(actions.iter().any(
            |a| matches!(a, UpdateAction::StartPairing { seq, .. } if *seq == state.pairing.seq)
        ));
    }

    #[test]
    fn pairing_ready_with_matching_seq_transitions_to_showing() {
        let mut state = AppState::new();
        state.screen = Screen::Pair;
        state.pairing.seq = 7;
        update(
            &mut state,
            Message::PairingReady {
                uri: "zellimobile://pair?v=1".to_string(),
                baseline_clients: 0,
                host: "192.168.1.1".to_string(),
                port: 50051,
                fingerprint_short: "abcd1234…".to_string(),
                token_name: "pair-7-1".to_string(),
                seq: 7,
            },
        );
        assert!(matches!(state.pairing.phase, PairingPhase::Showing { .. }));
        assert_eq!(
            state.pairing.pending_token_name.as_deref(),
            Some("pair-7-1")
        );
    }

    #[test]
    fn pairing_ready_with_stale_seq_is_ignored() {
        let mut state = AppState::new();
        state.screen = Screen::Pair;
        state.pairing.seq = 7; // current seq is 7
        let actions = update(
            &mut state,
            Message::PairingReady {
                uri: "zellimobile://pair?v=1".to_string(),
                baseline_clients: 0,
                host: "192.168.1.1".to_string(),
                port: 50051,
                fingerprint_short: "abcd…".to_string(),
                token_name: "pair-5-1".to_string(),
                seq: 5, // stale
            },
        );
        // Phase should remain Idle (was not changed).
        assert!(matches!(state.pairing.phase, PairingPhase::Idle));
        // The orphaned token from the stale attempt is revoked.
        assert!(matches!(
            actions.as_slice(),
            [UpdateAction::RevokePairingToken(name)] if name == "pair-5-1"
        ));
    }

    #[test]
    fn leave_during_generating_revokes_inflight_token_and_does_not_resurrect() {
        // Regression (round-1 re-review Major): leaving the Pair screen while a
        // StartPairing task is still in flight (phase Generating, no token yet)
        // must invalidate that attempt so the late PairingReady revokes its
        // minted token instead of resurrecting `Showing` off-screen.
        let mut state = AppState::new();
        state.screen = Screen::Pair;
        state.pairing.phase = PairingPhase::Generating;
        state.pairing.pending_token_name = None;
        let seq_at_dispatch = state.pairing.seq;

        // User tabs away mid-generation.
        update(&mut state, Message::NavTo(Screen::Server));
        assert_eq!(state.screen, Screen::Server);
        assert!(matches!(state.pairing.phase, PairingPhase::Idle));
        // seq was bumped, so the in-flight result is now stale.
        assert_ne!(state.pairing.seq, seq_at_dispatch);

        // The in-flight task completes and posts its result with the old seq.
        let actions = update(
            &mut state,
            Message::PairingReady {
                uri: "zellimobile://pair?v=1".to_string(),
                baseline_clients: 0,
                host: "192.168.1.1".to_string(),
                port: 50051,
                fingerprint_short: "abcd…".to_string(),
                token_name: "pair-leak".to_string(),
                seq: seq_at_dispatch,
            },
        );
        // No off-screen resurrection; the orphan token is revoked.
        assert!(matches!(state.pairing.phase, PairingPhase::Idle));
        assert!(state.pairing.pending_token_name.is_none());
        assert!(matches!(
            actions.as_slice(),
            [UpdateAction::RevokePairingToken(name)] if name == "pair-leak"
        ));
    }

    /// Build a `Showing` pairing phase for tests.
    fn showing_phase(baseline: usize) -> PairingPhase {
        PairingPhase::Showing {
            uri: "zellimobile://pair?v=1".to_string(),
            baseline_clients: baseline,
            host: "192.168.1.1".to_string(),
            port: 50051,
            fingerprint_short: "abcd…".to_string(),
            token_name: "pair-1-1".to_string(),
        }
    }

    #[test]
    fn pairing_connection_detected_when_client_count_rises() {
        let mut state = AppState::new();
        state.screen = Screen::Pair;
        state.pairing.phase = showing_phase(2);
        // one more than baseline
        update(&mut state, Message::StatusLoaded(Some(server_info(3))));
        assert!(matches!(state.pairing.phase, PairingPhase::Connected));
    }

    #[test]
    fn pair_regenerate_revokes_prior_pending_token() {
        // Review Major #4: regenerating must revoke the previously-minted token.
        let mut state = AppState::new();
        state.screen = Screen::Pair;
        state.pairing.pending_token_name = Some("pair-1-1".to_string());
        let actions = update(&mut state, Message::Key(key(KeyCode::Char('r'))));
        // Both a revoke (of the old token) and a fresh StartPairing are emitted.
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::RevokePairingToken(name) if name == "pair-1-1"))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::StartPairing { .. }))
        );
        // The pending name is cleared (it will be re-set by PairingReady).
        assert!(state.pairing.pending_token_name.is_none());
    }

    #[test]
    fn leaving_pair_screen_revokes_pending_token() {
        // Review Major #8: an unused pairing token must be revoked on leave.
        let mut state = AppState::new();
        state.screen = Screen::Pair;
        state.pairing.pending_token_name = Some("pair-3-9".to_string());
        let actions = update(&mut state, Message::NavTo(Screen::Server));
        assert_eq!(state.screen, Screen::Server);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, UpdateAction::RevokePairingToken(name) if name == "pair-3-9"))
        );
        assert!(state.pairing.pending_token_name.is_none());
        assert!(matches!(state.pairing.phase, PairingPhase::Idle));
    }

    #[test]
    fn pairing_connection_not_detected_off_pair_screen() {
        // Same client-count rise, but we are NOT on the Pair screen — the
        // heuristic must not fire (review Major #9: gate to Pair screen).
        let mut state = AppState::new();
        state.screen = Screen::Server;
        state.pairing.phase = showing_phase(2);
        update(&mut state, Message::StatusLoaded(Some(server_info(3))));
        assert!(matches!(state.pairing.phase, PairingPhase::Showing { .. }));
    }

    #[test]
    fn pairing_payload_assembly_produces_valid_uri() {
        // Unit test for the pairing payload assembly path.
        // Exercises PairingParams::to_uri() with realistic values.
        use crate::pairing::payload::PairingParams;
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

        let params = PairingParams {
            host: "10.0.1.5".to_string(),
            port: 50051,
            cert_fp_hex: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
                .to_string(),
            token: "secret-token-value".to_string(),
            read_only: false,
            label: "My Server".to_string(),
        };
        let uri = params.to_uri();

        assert!(uri.starts_with("zellimobile://pair?v=1"));
        assert!(uri.contains("h=10.0.1.5"));
        assert!(uri.contains("p=50051"));
        assert!(uri.contains("ro=0"));
        assert!(uri.contains("n=My%20Server"));

        // Verify token round-trips correctly.
        let t_val = uri
            .split('&')
            .find(|s| s.starts_with("t="))
            .and_then(|s| s.strip_prefix("t="))
            .expect("t= param missing");
        let decoded = URL_SAFE_NO_PAD.decode(t_val).unwrap();
        assert_eq!(std::str::from_utf8(&decoded).unwrap(), "secret-token-value");
    }
}
