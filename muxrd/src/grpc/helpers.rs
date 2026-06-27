//! Cross-cutting free helpers used across grpc submodules.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::actions::ActionAck;
use crate::cli::BackendKind;
use crate::multiplexer::{BackendSet, MuxBackend, PaneRef};
use crate::proto::{ActionAck as ProtoAck, PaneTarget};

// ─── Control routing ──────────────────────────────────────────────────────────

/// Try to route a control command through a live relay's AttachClient.
///
/// Routing priority:
/// 1. If `connection_id` is non-empty AND an entry with that key exists AND its
///    stored session matches `session` → route to that specific relay.
/// 2. Otherwise → scan all entries for any **writable** relay attached to
///    `session` (session-scoped fallback; preserves solo-client and legacy-client
///    behavior where the client doesn't send a connection_id).
///
/// All commands routed through this function are mutating *at the relay level*
/// (`SwitchTab`, `FocusPane`, `ToggleFullscreen`). `FocusPane` is accepted for
/// read-only token holders at the RPC gate, but the inbound task still drops it
/// for a read-only *relay*. The session-scoped fallback therefore skips read-only
/// relay entries: sending to one would succeed at the channel level but the
/// inbound task would silently drop the command at its own guard → false
/// `ok:true` response and client UI desync (Issue B).
///
/// Returns `Some(ok-ack)` if a relay was found and the command was queued;
/// `None` → caller falls back to the ephemeral CLI path.
///
/// Never errors on a stale / unknown / mismatched `connection_id`.
pub(super) fn try_route_control(
    control: &crate::relay::ControlRegistry,
    session: &str,
    connection_id: &str,
    cmd: crate::relay::RelayControl,
) -> Option<Response<ProtoAck>> {
    // ── Resolve sender + info string (no .await — this is a sync helper) ──────
    // Clone the sender out so the DashMap Ref (shard read-lock) is released
    // before we send — never hold a shard guard across the channel send.
    //
    // Routing priority:
    //   1. Per-connection: connection_id non-empty + session matches.
    //      (The exact-connection_id path is not filtered for read_only — the
    //      caller's own upstream `reject_if_read_only` gate already denied the
    //      RPC if the CALLER'S token is read-only. Routing to the exact relay
    //      is always intentional for a non-read-only caller.)
    //   2. Session fallback: any WRITABLE relay for the session. Read-only
    //      entries are skipped: their inbound tasks would drop the command
    //      silently → false ok:true (Issue B fix).
    //   3. Neither found → return None (caller uses ephemeral CLI path).
    let (tx, info): (
        tokio::sync::mpsc::UnboundedSender<crate::relay::RelayControl>,
        &str,
    ) = if !connection_id.is_empty() {
        let maybe = control
            .get(connection_id)
            .filter(|entry| entry.session == session)
            .map(|entry| entry.sender.clone());
        if let Some(sender) = maybe {
            (sender, "routed via relay client (per-connection)")
        } else {
            // connection_id absent/stale/mismatched — session fallback.
            // Only writable relays: a read-only relay's inbound task would
            // silently drop mutating commands (Issue B).
            let maybe_fallback = control
                .iter()
                .find(|entry| entry.session == session && !entry.read_only)
                .map(|entry| entry.sender.clone());
            (
                maybe_fallback?,
                "routed via relay client (session fallback, writable)",
            )
        }
    } else {
        // No connection_id — session fallback (writable relays only).
        let maybe_fallback = control
            .iter()
            .find(|entry| entry.session == session && !entry.read_only)
            .map(|entry| entry.sender.clone());
        (
            maybe_fallback?,
            "routed via relay client (session fallback, writable)",
        )
    };

    if tx.send(cmd).is_ok() {
        Some(Response::new(ProtoAck {
            ok: true,
            error: String::new(),
            info: info.to_owned(),
        }))
    } else {
        // Sender dead (relay tearing down); caller falls back to ephemeral.
        None
    }
}

// ─── Read-only gate + ack helpers ──────────────────────────────────────────────

/// Reject a request if its session token is read-only.
///
/// The `SessionReadOnly` extension is stashed by [`crate::auth::BearerAuthLayer`]
/// on every authenticated request.  Call this at the top of every **mutating**
/// RPC; focus/scroll/reads skip it.  Returns `Status::permission_denied` for
/// read-only tokens.
///
/// **Fail-closed**: if the `SessionReadOnly` extension is MISSING the request is
/// denied.  The auth layer always sets this extension on non-public paths, so a
/// missing extension on a mutating RPC is a bug — we must never silently allow it.
pub(super) fn reject_if_read_only<T>(request: &Request<T>, rpc: &str) -> Result<(), Status> {
    match request.extensions().get::<crate::auth::SessionReadOnly>() {
        Some(ro) if ro.0 => {
            log::info!("{rpc}: rejected — session token is read-only");
            Err(Status::permission_denied(
                "session token is read-only — mutating operations are not allowed",
            ))
        }
        Some(_) => Ok(()), // extension present and not read-only
        None => {
            // The auth layer always sets SessionReadOnly on non-public paths.
            // A missing extension on a mutating RPC is a bug — deny it (fail-closed).
            log::warn!(
                "{rpc}: rejected — SessionReadOnly extension absent (auth layer bug); \
                 denying to avoid fail-open"
            );
            Err(Status::permission_denied(
                "internal auth error: read-only flag not set — request denied (fail-closed)",
            ))
        }
    }
}

/// Validate a session name at an RPC boundary, mapping a rejection to
/// `Status::invalid_argument` (review Major G — path traversal).
///
/// Call this at the top of every handler/resolver that accepts a session name
/// before that name is used to build a socket path or reaches the `zellij`
/// binary.  Delegates to [`crate::ipc::validate_session_name`].
pub(super) fn validate_session(session: &str) -> Result<(), Status> {
    crate::ipc::validate_session_name(session).map_err(Status::invalid_argument)
}

// ─── Option C: backend-qualified session id routing (T04) ────────────────────────

/// Parse a `<backend>:` prefix into a [`BackendKind`].
///
/// The string scheme is **lowercase** and matches both the [`BackendKind`]
/// `Display` impl and [`make_id`] (the minting side, T05). Keep all three in sync:
/// `zellij` / `herdr`.
fn parse_backend_kind(s: &str) -> Option<BackendKind> {
    match s {
        "zellij" => Some(BackendKind::Zellij),
        "herdr" => Some(BackendKind::Herdr),
        _ => None,
    }
}

/// Mint an opaque, backend-qualified session id: `"<backend>:<bare>"`.
///
/// The inverse of [`resolve_session`]. The `<backend>` token is the lowercase
/// [`BackendKind`] `Display` form, so a round-trip
/// (`resolve_session(make_id(k, n))`) yields `(backend_for(k), n)`. Used by the
/// session-enumerating RPCs (T05) so every `id` they emit resolves cleanly here.
///
/// Kept beside [`resolve_session`] as the single source of truth for the wire
/// format. Consumed by the session-enumerating RPCs (`ListSessions` /
/// `CreateSession`, T05) and the relay's backend-qualified client-count key.
pub(crate) fn make_id(kind: BackendKind, bare: &str) -> String {
    format!("{kind}:{bare}")
}

/// Map a [`BackendKind`] to its proto [`crate::proto::Backend`] tag.
///
/// Used by the session-enumerating RPCs (T05) to tag each [`crate::proto::SessionInfo`]
/// and to populate `VersionInfo.available_backends`.
pub(crate) fn proto_backend(kind: BackendKind) -> crate::proto::Backend {
    match kind {
        BackendKind::Zellij => crate::proto::Backend::Zellij,
        BackendKind::Herdr => crate::proto::Backend::Herdr,
    }
}

/// Map a proto [`crate::proto::Backend`] tag to a [`BackendKind`].
///
/// Returns `None` for `BACKEND_UNSPECIFIED` (the caller decides whether an
/// unspecified backend is an error or defaults to the sole available one).
pub(crate) fn kind_from_proto(backend: crate::proto::Backend) -> Option<BackendKind> {
    match backend {
        crate::proto::Backend::Zellij => Some(BackendKind::Zellij),
        crate::proto::Backend::Herdr => Some(BackendKind::Herdr),
        crate::proto::Backend::Unspecified => None,
    }
}

/// Resolve an opaque session `id` to the backend that owns it and the bare
/// session name to hand that backend (Option C routing).
///
/// Wire format: `id == "<backend>:<bare>"` (e.g. `"zellij:dev"`). The split is on
/// the **first** `':'`; the prefix maps to a [`BackendKind`] via
/// [`parse_backend_kind`], which selects the concrete backend from `backends`.
///
/// Errors:
/// - unknown `<backend>` token → `InvalidArgument`,
/// - a recognised backend that this server is not running → `NotFound`
///   (the client should re-list sessions),
/// - a bare name failing the session-name guard → `InvalidArgument`.
///
/// **Back-compat:** an `id` with no `':'` is treated as a legacy *bare* name
/// (an older single-backend client, or a hand-typed name). It resolves to the
/// sole backend when exactly one is running; on a multi-backend server it is
/// rejected (`InvalidArgument`) asking the client to re-list, since there is no
/// way to know which backend owns an unqualified name.
///
/// **Security:** only the **bare** name is run through the [`validate_session`]
/// guard (path-traversal / allowlist) — never the full id — preserving the
/// invariant that every name reaching a backend / socket path is validated.
pub(crate) fn resolve_session(
    backends: &BackendSet,
    id: &str,
) -> Result<(Arc<dyn MuxBackend>, String), Status> {
    let (_, backend, bare) = resolve_session_kind(backends, id)?;
    Ok((backend, bare))
}

/// Like [`resolve_session`] but also returns the owning [`BackendKind`].
///
/// The relay needs the kind to build the backend-qualified client-count key
/// ([`make_id`]) so its `attach`/`detach` count bucket matches the one
/// `ListSessions` reads — even when the client supplied a legacy bare name on a
/// single-backend server (the kind is then the sole backend's). Keeping this as
/// the kind-aware core lets [`resolve_session`] stay a thin shim for the many
/// id-only call sites.
pub(crate) fn resolve_session_kind(
    backends: &BackendSet,
    id: &str,
) -> Result<(BackendKind, Arc<dyn MuxBackend>, String), Status> {
    let (kind, backend, bare) = match id.split_once(':') {
        Some((kind_str, bare)) => {
            // Any ':' means the id is backend-qualified (bare names are
            // [A-Za-z0-9_-] and never contain ':'). An unrecognised prefix is a
            // client error, not a bare name.
            let kind = parse_backend_kind(kind_str).ok_or_else(|| {
                Status::invalid_argument(format!(
                    "session id {id:?} names an unknown backend {kind_str:?} \
                     (expected one of: zellij, herdr)"
                ))
            })?;
            let backend = backends.get(kind).ok_or_else(|| {
                Status::not_found(format!(
                    "session id {id:?} targets the '{kind}' backend, which is not \
                     running on this server — re-list sessions"
                ))
            })?;
            (kind, backend.clone(), bare.to_owned())
        }
        None => {
            // No prefix → legacy bare name. Only resolvable on a single-backend
            // server; otherwise the owner is ambiguous.
            if backends.len() == 1 {
                let kind = backends
                    .kinds()
                    .next()
                    .expect("BackendSet invariant: at least one backend");
                (kind, backends.primary().clone(), id.to_owned())
            } else {
                return Err(Status::invalid_argument(format!(
                    "session {id:?} is missing a '<backend>:' prefix and this server \
                     runs multiple backends — re-list sessions to obtain a \
                     backend-qualified id"
                )));
            }
        }
    };
    // Validate the BARE name only (never the full id) — preserves the
    // path-traversal / allowlist invariant at the backend boundary.
    validate_session(&bare)?;
    Ok((kind, backend, bare))
}

/// Validate a `CreateSession --layout` value (review Major I — arbitrary layout
/// file load → host code execution).
///
/// We only allow a bare layout *name* drawn from `[A-Za-z0-9_-]` (same allowlist
/// as session names): no absolute path, no `/`, no `..`, no metacharacters.
/// zellij resolves a bare name against its own layout directory.  Anything else
/// → `Status::invalid_argument`.
pub fn validate_layout_name(layout: &str) -> Result<(), Status> {
    if layout.is_empty()
        || !layout
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(Status::invalid_argument(format!(
            "invalid layout {layout:?}: only a bare layout name [A-Za-z0-9_-] is allowed \
             (absolute paths, '/', and '..' are rejected)"
        )));
    }
    Ok(())
}

/// Run a blocking action helper on the blocking pool and map its result into a
/// proto [`ProtoAck`].  A `LogError` ack (`ok == false`) is surfaced as an `Ok`
/// response with `ok=false` + `error` populated (it's a logical failure, not a
/// transport error); a hard IPC failure becomes `Status::internal`.
pub(super) async fn run_action<F>(rpc: &'static str, f: F) -> Result<Response<ProtoAck>, Status>
where
    F: FnOnce() -> anyhow::Result<ActionAck> + Send + 'static,
{
    let ack = tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| Status::internal(format!("{rpc}: action task panicked: {e}")))?
        .map_err(|e| {
            log::warn!("{rpc}: action failed: {e:#}");
            Status::internal(format!("{rpc}: {e:#}"))
        })?;

    log::info!(
        "{rpc}: ok={} error={:?} info={:?}",
        ack.ok,
        ack.error,
        ack.info
    );
    Ok(Response::new(ProtoAck {
        ok: ack.ok,
        error: ack.error.unwrap_or_default(),
        info: ack.info.unwrap_or_default(),
    }))
}

/// Build the neutral [`PaneRef`] from a proto [`PaneTarget`].
///
/// The [`PaneRef`] carries the same `(id, is_plugin)` pair and is passed straight
/// through to the backend handlers and the [`crate::relay::RelayControl`] variants
/// (both neutral as of P1.03 — no per-call-site pane-id conversion).
///
/// Option C: the `session` field is now an opaque routing id and is **not**
/// inspected here — callers strip + validate it via
/// [`MuxrService::resolve_session`](super::MuxrService::resolve_session).
pub(super) fn pane_ref(target: &PaneTarget) -> PaneRef {
    PaneRef {
        id: target.pane_id,
        is_plugin: target.is_plugin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::SessionReadOnly;
    use crate::cli::BackendKind;
    use crate::multiplexer::{BackendSet, MuxBackend, ZellijBackend};
    use crate::relay::{ControlEntry, ControlRegistry, RelayControl};
    use std::sync::Arc;
    use tokio::sync::mpsc;

    // ─── try_route_control tests ─────────────────────────────────────────────

    /// Build a ControlRegistry with one writable entry and return the receiver
    /// so tests can inspect what was sent.
    fn make_registry(
        conn_id: &str,
        session: &str,
    ) -> (ControlRegistry, mpsc::UnboundedReceiver<RelayControl>) {
        make_registry_with_flags(conn_id, session, false)
    }

    /// Build a ControlRegistry with one entry of the given `read_only` flag.
    fn make_registry_with_flags(
        conn_id: &str,
        session: &str,
        read_only: bool,
    ) -> (ControlRegistry, mpsc::UnboundedReceiver<RelayControl>) {
        let registry: ControlRegistry = Arc::new(dashmap::DashMap::new());
        let (tx, rx) = mpsc::unbounded_channel::<RelayControl>();
        registry.insert(
            conn_id.to_owned(),
            ControlEntry {
                session: session.to_owned(),
                sender: tx,
                read_only,
            },
        );
        (registry, rx)
    }

    #[test]
    fn routes_by_connection_id_when_session_matches() {
        // A request carrying the exact connection_id that matches an entry for
        // the same session must be routed to that relay's sender.
        let (reg, mut rx) = make_registry("conn-1", "my-session");
        let result = try_route_control(&reg, "my-session", "conn-1", RelayControl::SwitchTab(42));
        assert!(result.is_some(), "should route via connection_id");
        // Verify the command arrived on the relay's receiver.
        match rx.try_recv() {
            Ok(RelayControl::SwitchTab(42)) => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn falls_back_to_session_when_connection_id_is_empty() {
        // Empty connection_id → session-scoped fallback: route to any relay for
        // the session.
        let (reg, mut rx) = make_registry("conn-2", "session-A");
        let result = try_route_control(
            &reg,
            "session-A",
            "", // empty — no connection_id from client
            RelayControl::SwitchTab(99),
        );
        assert!(result.is_some(), "session fallback should route");
        match rx.try_recv() {
            Ok(RelayControl::SwitchTab(99)) => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn falls_back_to_session_when_connection_id_is_stale() {
        // A stale/unknown connection_id must NOT error; it falls back to the
        // session-scoped relay.
        let (reg, mut rx) = make_registry("conn-3", "session-B");
        let result = try_route_control(
            &reg,
            "session-B",
            "stale-id-xyz", // not in the registry
            RelayControl::SwitchTab(7),
        );
        assert!(
            result.is_some(),
            "stale id should fall back to session relay"
        );
        match rx.try_recv() {
            Ok(RelayControl::SwitchTab(7)) => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn does_not_route_when_connection_id_session_mismatches() {
        // A connection_id that exists but is registered under a DIFFERENT
        // session must not route to it. If there's no other relay for the
        // requested session, None is returned.
        let (reg, mut rx) = make_registry("conn-4", "session-C");
        let result = try_route_control(
            &reg,
            "OTHER-SESSION", // mismatch — conn-4 belongs to session-C
            "conn-4",
            RelayControl::SwitchTab(1),
        );
        assert!(result.is_none(), "session mismatch should not route");
        // The relay for session-C must NOT have received anything.
        assert!(
            rx.try_recv().is_err(),
            "mismatched session must not deliver to relay"
        );
    }

    #[test]
    fn returns_none_when_no_relay_for_session() {
        // No relay registered for the requested session at all → None.
        let registry: ControlRegistry = Arc::new(dashmap::DashMap::new());
        let result = try_route_control(
            &registry,
            "nonexistent-session",
            "",
            RelayControl::SwitchTab(1),
        );
        assert!(
            result.is_none(),
            "no relay → None (caller uses ephemeral path)"
        );
    }

    #[test]
    fn per_connection_routing_targets_exact_relay_among_two() {
        // Two relays on the same session. A request with conn-A's id must route
        // to relay A, NOT relay B.
        let reg: ControlRegistry = Arc::new(dashmap::DashMap::new());

        let (tx_a, mut rx_a) = mpsc::unbounded_channel::<RelayControl>();
        let (tx_b, mut rx_b) = mpsc::unbounded_channel::<RelayControl>();

        reg.insert(
            "conn-A".to_owned(),
            ControlEntry {
                session: "shared-session".to_owned(),
                sender: tx_a,
                read_only: false,
            },
        );
        reg.insert(
            "conn-B".to_owned(),
            ControlEntry {
                session: "shared-session".to_owned(),
                sender: tx_b,
                read_only: false,
            },
        );

        let result = try_route_control(
            &reg,
            "shared-session",
            "conn-A",
            RelayControl::SwitchTab(11),
        );
        assert!(result.is_some(), "should route via conn-A");

        // Relay A got the command.
        match rx_a.try_recv() {
            Ok(RelayControl::SwitchTab(11)) => {}
            other => panic!("relay A: unexpected: {other:?}"),
        }
        // Relay B must NOT have received anything.
        assert!(
            rx_b.try_recv().is_err(),
            "relay B must not receive cmd for conn-A"
        );
    }

    // ─── Issue B: read-only fallback filtering ───────────────────────────────

    #[test]
    fn fallback_skips_read_only_and_routes_to_writable() {
        // Two relays on the same session: one read-only, one writable.
        // A mutating command with an empty/stale connection_id must skip the
        // read-only entry and route to the writable one (Issue B fix).
        let reg: ControlRegistry = Arc::new(dashmap::DashMap::new());

        let (tx_ro, mut rx_ro) = mpsc::unbounded_channel::<RelayControl>();
        let (tx_rw, mut rx_rw) = mpsc::unbounded_channel::<RelayControl>();

        // Insert the read-only entry first; DashMap shard iteration order is
        // non-deterministic, but the `!entry.read_only` filter selects the
        // writable entry regardless of visit order.
        reg.insert(
            "conn-ro".to_owned(),
            ControlEntry {
                session: "sess".to_owned(),
                sender: tx_ro,
                read_only: true,
            },
        );
        reg.insert(
            "conn-rw".to_owned(),
            ControlEntry {
                session: "sess".to_owned(),
                sender: tx_rw,
                read_only: false,
            },
        );

        // Empty connection_id → session fallback.
        let result = try_route_control(&reg, "sess", "", RelayControl::SwitchTab(5));
        assert!(
            result.is_some(),
            "should route to the writable relay, not be blocked by read-only entry"
        );

        // Writable relay got the command.
        match rx_rw.try_recv() {
            Ok(RelayControl::SwitchTab(5)) => {}
            other => panic!("writable relay: unexpected: {other:?}"),
        }
        // Read-only relay must NOT have received anything.
        assert!(
            rx_ro.try_recv().is_err(),
            "read-only relay must not receive a mutating command via session fallback"
        );
    }

    #[test]
    fn fallback_returns_none_when_only_read_only_relay_exists() {
        // Only a read-only relay registered for the session. A mutating command
        // with no connection_id must return None so the caller falls through to
        // the ephemeral path — never a false ok:true (Issue B fix).
        let (reg, mut rx_ro) = make_registry_with_flags("conn-ro-only", "sess-ro", true);

        let result = try_route_control(&reg, "sess-ro", "", RelayControl::SwitchTab(9));
        assert!(
            result.is_none(),
            "only read-only relay → None (must fall through to ephemeral)"
        );
        // Confirm nothing was sent to the read-only relay.
        assert!(
            rx_ro.try_recv().is_err(),
            "read-only relay must not receive any command"
        );
    }

    #[test]
    fn stale_id_fallback_skips_read_only_and_routes_to_writable() {
        // Stale connection_id + one read-only relay + one writable relay.
        // The stale-id fallback path must also skip the read-only entry.
        let reg: ControlRegistry = Arc::new(dashmap::DashMap::new());

        let (tx_ro, mut rx_ro) = mpsc::unbounded_channel::<RelayControl>();
        let (tx_rw, mut rx_rw) = mpsc::unbounded_channel::<RelayControl>();

        reg.insert(
            "conn-ro".to_owned(),
            ControlEntry {
                session: "sess2".to_owned(),
                sender: tx_ro,
                read_only: true,
            },
        );
        reg.insert(
            "conn-rw".to_owned(),
            ControlEntry {
                session: "sess2".to_owned(),
                sender: tx_rw,
                read_only: false,
            },
        );

        let result = try_route_control(&reg, "sess2", "stale-xyz", RelayControl::SwitchTab(3));
        assert!(
            result.is_some(),
            "stale id fallback should route to writable relay"
        );
        match rx_rw.try_recv() {
            Ok(RelayControl::SwitchTab(3)) => {}
            other => panic!("writable relay: unexpected: {other:?}"),
        }
        assert!(
            rx_ro.try_recv().is_err(),
            "read-only relay must not receive the command"
        );
    }

    // ─── reject_if_read_only tests ───────────────────────────────────────────

    #[test]
    fn reject_if_read_only_denies_when_extension_absent() {
        // Fail-closed: a mutating RPC with no SessionReadOnly extension (auth-layer
        // bug) must be denied, never allowed.
        let req = Request::new(());
        assert!(reject_if_read_only(&req, "Test").is_err());
    }

    #[test]
    fn reject_if_read_only_denies_read_only_token() {
        let mut req = Request::new(());
        req.extensions_mut().insert(SessionReadOnly(true));
        assert!(reject_if_read_only(&req, "Test").is_err());
    }

    #[test]
    fn reject_if_read_only_allows_writable_token() {
        let mut req = Request::new(());
        req.extensions_mut().insert(SessionReadOnly(false));
        assert!(reject_if_read_only(&req, "Test").is_ok());
    }

    #[test]
    fn validate_session_accepts_a_plain_name() {
        assert!(validate_session("backend-dev_1").is_ok());
    }

    #[test]
    fn validate_session_rejects_path_traversal() {
        assert!(validate_session("../etc").is_err());
        assert!(validate_session("a/b").is_err());
    }

    // ─── Option C: resolve_session / make_id (T04) ───────────────────────────

    /// A single-zellij set (the production default at `MuxrService::new`).
    fn zellij_only_set() -> BackendSet {
        BackendSet::single(BackendKind::Zellij, Arc::new(ZellijBackend))
    }

    /// A two-backend set. `ZellijBackend` stands in for both kinds: `resolve_session`
    /// only uses the kind for registry lookup and never invokes a backend method,
    /// so this exercises multi-backend routing/ambiguity without a live herdr.
    fn two_backend_set() -> BackendSet {
        BackendSet::new(vec![
            (
                BackendKind::Zellij,
                Arc::new(ZellijBackend) as Arc<dyn MuxBackend>,
            ),
            (
                BackendKind::Herdr,
                Arc::new(ZellijBackend) as Arc<dyn MuxBackend>,
            ),
        ])
    }

    #[test]
    fn resolve_session_strips_prefix_to_bare_name() {
        let (_, bare) =
            resolve_session(&zellij_only_set(), "zellij:dev").expect("zellij:dev should resolve");
        assert_eq!(bare, "dev");
    }

    #[test]
    fn resolve_session_rejects_path_traversal_in_bare_name() {
        // SECURITY: the prefix is valid but the BARE name escapes — it must be
        // rejected by the ipc guard, which runs on the stripped bare name
        // ("../etc"), never on the full id.
        let err = resolve_session(&zellij_only_set(), "zellij:../etc")
            .expect_err("zellij:../etc must be rejected");
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn resolve_session_rejects_path_traversal_with_slash() {
        let err = resolve_session(&zellij_only_set(), "zellij:a/b")
            .expect_err("zellij:a/b must be rejected");
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn resolve_session_rejects_unknown_backend_prefix() {
        let err = resolve_session(&zellij_only_set(), "tmux:dev")
            .expect_err("unknown backend prefix must be rejected");
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn resolve_session_not_found_when_backend_absent() {
        // A herdr-qualified id on a zellij-only server → NotFound (client re-lists).
        let err = resolve_session(&zellij_only_set(), "herdr:dev")
            .expect_err("absent backend → NotFound");
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[test]
    fn resolve_session_bare_name_falls_back_on_single_backend() {
        // Back-compat: a legacy bare name resolves to the sole backend.
        let (_, bare) = resolve_session(&zellij_only_set(), "dev")
            .expect("bare name should resolve on a single-backend server");
        assert_eq!(bare, "dev");
    }

    #[test]
    fn resolve_session_bare_name_rejected_on_multi_backend() {
        // Ambiguous: a bare name on a multi-backend server has no determinable owner.
        let err = resolve_session(&two_backend_set(), "dev")
            .expect_err("bare name is ambiguous on a multi-backend server");
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn resolve_session_routes_each_prefix_to_its_backend() {
        let set = two_backend_set();
        let (_, bare_z) = resolve_session(&set, "zellij:a").expect("zellij prefix resolves");
        assert_eq!(bare_z, "a");
        let (_, bare_h) = resolve_session(&set, "herdr:b").expect("herdr prefix resolves");
        assert_eq!(bare_h, "b");
    }

    #[test]
    fn make_id_round_trips_through_resolve_session() {
        let set = two_backend_set();
        let id = make_id(BackendKind::Herdr, "proj");
        assert_eq!(
            id, "herdr:proj",
            "make_id uses the lowercase Display scheme"
        );
        let (_, bare) = resolve_session(&set, &id).expect("a minted id must resolve");
        assert_eq!(bare, "proj");
    }

    #[test]
    fn list_sessions_minted_id_round_trips_with_kind() {
        // T05: an id minted exactly as `list_sessions_impl` mints it
        // (`make_id(kind, bare)`) must resolve back to the SAME backend kind +
        // bare name via the kind-aware resolver. This is the round-trip the
        // ListSessions → AttachTerminal handoff depends on.
        let set = two_backend_set();
        for kind in [BackendKind::Zellij, BackendKind::Herdr] {
            let id = make_id(kind, "dev");
            let (resolved_kind, _, bare) =
                resolve_session_kind(&set, &id).expect("minted id must resolve");
            assert_eq!(resolved_kind, kind, "round-trip must preserve the backend");
            assert_eq!(bare, "dev");
        }
    }

    #[test]
    fn proto_backend_maps_each_kind() {
        assert_eq!(
            proto_backend(BackendKind::Zellij),
            crate::proto::Backend::Zellij
        );
        assert_eq!(
            proto_backend(BackendKind::Herdr),
            crate::proto::Backend::Herdr
        );
    }

    #[test]
    fn kind_from_proto_inverts_proto_backend() {
        assert_eq!(
            kind_from_proto(crate::proto::Backend::Zellij),
            Some(BackendKind::Zellij)
        );
        assert_eq!(
            kind_from_proto(crate::proto::Backend::Herdr),
            Some(BackendKind::Herdr)
        );
        assert_eq!(kind_from_proto(crate::proto::Backend::Unspecified), None);
    }

    #[test]
    fn client_count_is_isolated_per_backend_qualified_id() {
        // Carried T04 fix: two same-name sessions on different backends must NOT
        // share one connected-client bucket. The relay keys `attach` by
        // `make_id(kind, bare)` and `ListSessions` reads `count(make_id(...))`,
        // so `zellij:dev` and `herdr:dev` are independent counters.
        let clients = crate::client_count::SessionClients::new();
        let zellij_dev = make_id(BackendKind::Zellij, "dev");
        let herdr_dev = make_id(BackendKind::Herdr, "dev");
        assert_ne!(zellij_dev, herdr_dev);

        let _g_z = clients.attach(&zellij_dev);
        assert_eq!(clients.count(&zellij_dev), 1);
        assert_eq!(
            clients.count(&herdr_dev),
            0,
            "the herdr:dev bucket must be unaffected by a zellij:dev attach"
        );

        let _g_h1 = clients.attach(&herdr_dev);
        let _g_h2 = clients.attach(&herdr_dev);
        assert_eq!(clients.count(&herdr_dev), 2);
        assert_eq!(
            clients.count(&zellij_dev),
            1,
            "the zellij:dev bucket must be unaffected by herdr:dev attaches"
        );
    }
}
