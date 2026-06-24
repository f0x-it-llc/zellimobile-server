//! Session lifecycle RPC implementations: list, attach, rename, kill, create.

use tonic::{Request, Response, Status, Streaming};

use crate::actions;
use crate::proto::{
    ActionAck as ProtoAck, ClientFrame, CreateSessionReq, Empty, RenameSessionReq, SessionList,
    SessionRef,
};

use super::ZelliService;
use super::helpers::{reject_if_read_only, run_action, validate_layout_name, validate_session};

impl ZelliService {
    // ── ListSessions (C1) ───────────────────────────────────────────────────

    pub(super) async fn list_sessions_impl(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<SessionList>, Status> {
        log::debug!("ListSessions: scanning session sockets");

        let sessions = tokio::task::spawn_blocking(crate::query::list_sessions_with_resurrectables)
            .await
            .map_err(|e| Status::internal(format!("ListSessions task panicked: {e}")))?
            .map_err(|e| {
                log::warn!("ListSessions: error enumerating sessions: {e:#}");
                Status::internal(format!("failed to list sessions: {e:#}"))
            })?;

        // `connected_clients` is sourced from this server's own relay registry
        // (cheap, no IPC). The layout-derived enrichment fields
        // (tab_count/pane_count/has_bell/is_current) are left at their zero
        // defaults here: filling them requires a per-session layout query whose
        // transient AttachClient can briefly resize the session, so polling them
        // for every session is deferred pending the on-rig resize check (Phase
        // F G4b). Clients fall back to GetLayout on demand for counts.
        let clients = self.clients.clone();
        let proto_sessions: Vec<crate::proto::SessionInfo> = sessions
            .into_iter()
            .map(|(name, age_secs, resurrectable)| {
                let connected_clients = clients.count(&name);
                crate::proto::SessionInfo {
                    name,
                    age_secs,
                    resurrectable,
                    tab_count: 0,
                    pane_count: 0,
                    has_bell: false,
                    is_current: false,
                    connected_clients,
                }
            })
            .collect();

        let count = proto_sessions.len();
        log::info!("ListSessions: returning {count} session(s)");
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
        let stream = crate::relay::attach_relay(
            inbound,
            read_only,
            token,
            self.clients.clone(),
            self.control.clone(),
            self.view_state.clone(),
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
        validate_session(&req.session)?;
        // The new name becomes a session name too — validate it the same way.
        validate_session(&req.name)?;
        let session = req.session;
        let new_name = req.name;
        log::info!("RenameSession: session='{session}' → '{new_name}'");
        run_action("RenameSession", move || {
            actions::rename_session(&session, new_name)
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
        validate_session(&req.session)?;
        let session = req.session;
        log::info!("KillSession: session='{session}'");
        tokio::task::spawn_blocking(move || actions::kill_session(&session))
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
        // When the client supplies NO layout (the normal case — the ZelliMobile
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
        log::info!("CreateSession: name='{name}' layout={layout:?}");
        run_action("CreateSession", move || {
            actions::create_session(&name, layout)
        })
        .await
    }
}
