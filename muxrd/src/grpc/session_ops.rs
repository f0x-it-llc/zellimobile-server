//! Session lifecycle RPC implementations: list, attach, rename, kill, create.

use tonic::{Request, Response, Status, Streaming};

use crate::proto::{
    ActionAck as ProtoAck, ClientFrame, CreateSessionReq, Empty, RenameSessionReq, SessionList,
    SessionRef,
};

use crate::multiplexer::{make_id, validate_session};

use super::MuxrService;
use super::helpers::{
    kind_from_proto, proto_backend, reject_if_read_only, run_action, validate_layout_name,
};

impl MuxrService {
    // ── ListSessions (C1) ───────────────────────────────────────────────────

    pub(super) async fn list_sessions_impl(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<SessionList>, Status> {
        log::debug!("ListSessions: scanning session sockets across all backends");

        // Phase 3: fan ListSessions across EVERY backend this server drives and
        // merge the results. Each session is tagged with its owning backend and
        // assigned an opaque, backend-qualified `id` (`<backend>:<bare>`, via
        // `make_id`) that round-trips through `resolve_session`. A backend that
        // errors mid-run is skipped (warn-logged) rather than failing the whole
        // RPC — one unreachable backend must not blind the client to the other.
        //
        // `connected_clients` is sourced from this server's own relay registry
        // (cheap, no IPC), keyed by the SAME opaque `id` the relay's attach site
        // counts under — so two same-name sessions on different backends
        // (`zellij:dev` + `herdr:dev`) no longer share one count bucket. The
        // layout-derived enrichment fields (tab_count/pane_count/has_bell/
        // is_current) are left at their zero defaults here: filling them requires
        // a per-session layout query whose transient AttachClient can briefly
        // resize the session (Phase F G4b). Clients fall back to GetLayout.
        let clients = self.clients.clone();
        let mut proto_sessions: Vec<crate::proto::SessionInfo> = Vec::new();

        for (kind, backend) in self.backends.iter() {
            let backend = backend.clone();
            let listed =
                tokio::task::spawn_blocking(move || backend.list_sessions_with_resurrectables())
                    .await
                    .map_err(|e| {
                        Status::internal(format!("ListSessions task panicked ({kind:?}): {e}"))
                    })?;

            let sessions = match listed {
                Ok(sessions) => sessions,
                Err(e) => {
                    // Mid-run-unreachable tolerance: skip this backend, keep the
                    // others. The client still sees every reachable backend's
                    // sessions instead of a blanket Internal error.
                    log::warn!("backend {kind:?} unreachable, skipping in ListSessions: {e:#}");
                    continue;
                }
            };

            let proto_kind = proto_backend(kind) as i32;
            for (name, age_secs, resurrectable) in sessions {
                let id = make_id(kind, &name);
                let connected_clients = clients.count(&id);
                proto_sessions.push(crate::proto::SessionInfo {
                    name,
                    age_secs,
                    resurrectable,
                    tab_count: 0,
                    pane_count: 0,
                    has_bell: false,
                    is_current: false,
                    connected_clients,
                    backend: proto_kind,
                    id,
                });
            }
        }

        let count = proto_sessions.len();
        log::info!("ListSessions: returning {count} session(s) across all backends");
        Ok(Response::new(SessionList {
            sessions: proto_sessions,
        }))
    }

    // ── AttachTerminal — bidi relay (B2) ────────────────────────────────────

    pub(super) async fn attach_terminal_impl(
        &self,
        request: Request<Streaming<ClientFrame>>,
    ) -> Result<Response<crate::relay::ServerFrameStream>, Status> {
        // ── Major A: enforce the read-only flag on the terminal stream ────────
        // A read-only token must NOT be able to inject input/resize.  Policy:
        // **render-only** — a read-only attach still receives the live render
        // stream (good UX for observers), but the relay drops every inbound
        // input/resize frame.  (We deliberately do not reject the whole stream
        // so RO viewers can still watch.)
        let read_only = request
            .extensions()
            .get::<crate::auth::SessionReadOnly>()
            .map(|ro| ro.0)
            .unwrap_or(true); // fail closed if somehow unset
        log::debug!("AttachTerminal: session read_only={read_only}");

        // ── Major H: stash the token so the relay can re-validate it mid-stream.
        // If the extension is missing (shouldn't happen post-auth) we still
        // proceed but with no token → the relay treats that as already-revoked
        // on its first re-check and tears the stream down.
        let token = request
            .extensions()
            .get::<crate::auth::SessionToken>()
            .map(|t| t.0.clone());

        let inbound = request.into_inner();
        // Option C: the relay resolves the opaque session id in the first
        // `AttachReq` frame against the whole backend set (the id isn't known
        // until that frame is read), so pass the full set rather than a single
        // primary backend.
        let stream = crate::relay::attach_relay(
            inbound,
            read_only,
            token,
            self.clients.clone(),
            self.control.clone(),
            self.view_state.clone(),
            self.backends.clone(),
        )
        .await?;
        Ok(Response::new(stream))
    }

    // ── Session lifecycle (D2) ────────────────────────────────────────────────

    /// Rename the session. MUTATING (read-only rejected).
    pub(super) async fn rename_session_impl(
        &self,
        request: Request<RenameSessionReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "RenameSession")?;
        let req = request.into_inner();
        let (backend, session) = self.resolve_session(&req.session)?;
        // The new name becomes a session name too — validate it the same way.
        // (It is a bare name, not an opaque id: the renamed session stays on the
        // same backend, so only the bare name is supplied/validated here.)
        validate_session(&req.name)?;
        let new_name = req.name;
        log::info!("RenameSession: session='{session}' → '{new_name}'");
        run_action("RenameSession", move || {
            backend.rename_session(&session, new_name)
        })
        .await
    }

    /// Kill a named session. MUTATING (read-only rejected).
    ///
    /// Sends `ClientToServerMsg::KillSession` directly (not via send_action —
    /// KillSession is not a zellij Action variant).
    pub(super) async fn kill_session_impl(
        &self,
        request: Request<SessionRef>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "KillSession")?;
        let req = request.into_inner();
        let (backend, session) = self.resolve_session(&req.session)?;
        log::info!("KillSession: session='{session}'");
        tokio::task::spawn_blocking(move || backend.kill_session(&session))
            .await
            .map_err(|e| Status::internal(format!("KillSession task panicked: {e}")))?
            .map_err(|e| {
                log::warn!("KillSession: failed: {e:#}");
                Status::internal(format!("KillSession: {e:#}"))
            })?;
        Ok(Response::new(ProtoAck {
            ok: true,
            error: String::new(),
            info: String::new(),
        }))
    }

    /// Create a new detached session. MUTATING (read-only rejected).
    pub(super) async fn create_session_impl(
        &self,
        request: Request<CreateSessionReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "CreateSession")?;
        let req = request.into_inner();

        // ── Phase 3: route the create to the requested backend ────────────────
        // `req.backend` (proto Backend) selects which multiplexer owns the new
        // session. Resolution rules:
        //   - a named backend that is running        → use it
        //   - a named backend that is NOT running    → InvalidArgument
        //   - BACKEND_UNSPECIFIED + exactly 1 backend → default to it (ergonomics)
        //   - BACKEND_UNSPECIFIED + >1 backend       → InvalidArgument (ambiguous)
        let req_backend = crate::proto::Backend::try_from(req.backend).map_err(|_| {
            Status::invalid_argument(format!(
                "CreateSession: unknown backend tag {}",
                req.backend
            ))
        })?;
        let kind = match kind_from_proto(req_backend) {
            Some(kind) => kind,
            None => {
                // Unspecified: default only when there is a single unambiguous backend.
                let mut kinds = self.backends.kinds();
                let sole = kinds.next();
                if kinds.next().is_some() {
                    return Err(Status::invalid_argument(
                        "CreateSession: this server runs multiple backends — specify which \
                         backend to create the session on",
                    ));
                }
                sole.expect("BackendSet invariant: at least one backend")
            }
        };
        // Error-code mapping (matches `resolve_session` — see `multiplexer::routing`):
        //   recognised kind but not running on this server → NotFound (client re-lists).
        let backend = self.backends.get(kind).cloned().ok_or_else(|| {
            Status::not_found(format!(
                "CreateSession: backend '{kind}' is recognised but not running on this \
                 server — re-list to see available backends"
            ))
        })?;

        validate_session(&req.name)?;
        let name = req.name;

        // ── Major I: constrain --layout to a simple layout NAME ───────────────
        // An arbitrary client-supplied --layout path is forwarded to `zellij`,
        // which would happily load an attacker-controlled layout file — and
        // zellij layouts can carry `command`/`run` directives → host code
        // execution.  We therefore accept only a bare layout *name* (the same
        // [A-Za-z0-9_-] allowlist as session names): no absolute path, no `/`,
        // no `..`.  zellij resolves a bare name against its own layout dir.
        //
        // When the client supplies NO layout (the normal case — the Muxr
        // app always sends an empty layout), `actions::create_session` defaults
        // to the bundled BAR-LESS layout so app-created sessions hide zellij's
        // tab-bar/status-bar (the app renders those controls). That default is a
        // server-authored file at a fixed path — not client input — so it is
        // exempt from this name allowlist.
        let layout = if req.layout.is_empty() {
            None
        } else {
            validate_layout_name(&req.layout)?;
            Some(req.layout)
        };
        log::info!("CreateSession: backend='{kind}' name='{name}' layout={layout:?}");
        run_action("CreateSession", move || {
            backend.create_session(&name, layout)
        })
        .await
    }
}
