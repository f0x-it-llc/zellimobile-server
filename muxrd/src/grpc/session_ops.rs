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

/// Per-backend ceiling for the concurrent ListSessions fan-out (S-M3).
///
/// Must be strictly above herdr's control-socket `READ_TIMEOUT` (3 s) so a
/// well-behaved-but-slow backend can still complete within the budget, while
/// bounding a truly hung one. Value mirrors `VERSION_CHECK_TIMEOUT` /
/// `RECV_TIMEOUT` in the codebase (5 s = conservative local-IPC budget).
const LIST_SESSIONS_BACKEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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
        //
        // S-M2: concurrent fan-out — all backend tasks are spawned up-front and
        // joined together via `futures::future::join_all` so latency =
        // max(per-backend) not Σ.
        // S-M3: each backend query is wrapped in `tokio::time::timeout` so a hung
        // (non-erroring) backend (e.g. herdr 3 s control-socket READ_TIMEOUT)
        // can't stall the whole RPC; on timeout → warn + skip (partial results).
        // S-M4: a JoinError (task panic or cancellation) degrades to warn + skip,
        // same as a normal `Err`, rather than propagating via `?` and failing the
        // entire ListSessions RPC.
        let clients = self.clients.clone();

        // Build concurrent futures — one per backend, timeout-wrapped. The
        // `spawn_blocking` keeps the blocking IPC call off the async executor
        // (CODE_STANDARDS: never `.await` blocking IPC directly).
        let backend_futures: Vec<_> = self
            .backends
            .iter()
            .map(|(kind, backend)| {
                let backend = backend.clone();
                async move {
                    let result = tokio::time::timeout(
                        LIST_SESSIONS_BACKEND_TIMEOUT,
                        tokio::task::spawn_blocking(move || {
                            backend.list_sessions_with_resurrectables()
                        }),
                    )
                    .await;
                    (kind, result)
                }
            })
            .collect();

        // Await all backends concurrently; latency = max(per-backend).
        let backend_results = futures::future::join_all(backend_futures).await;

        // Merge sessions from all backends that succeeded within the timeout.
        let mut proto_sessions: Vec<crate::proto::SessionInfo> = Vec::new();
        for (kind, result) in backend_results {
            let sessions = match result {
                // S-M3: timeout — warn + skip, return whatever other backends had.
                Err(_elapsed) => {
                    log::warn!(
                        "backend {kind:?} timed out after {:?} in ListSessions, skipping",
                        LIST_SESSIONS_BACKEND_TIMEOUT
                    );
                    continue;
                }
                // S-M4: spawn_blocking task panicked or was cancelled — warn + skip.
                Ok(Err(join_err)) => {
                    log::warn!(
                        "backend {kind:?} task panicked in ListSessions: {join_err}, skipping"
                    );
                    continue;
                }
                // Mid-run-unreachable tolerance: skip this backend, keep the
                // others. The client still sees every reachable backend's sessions.
                Ok(Ok(Err(e))) => {
                    log::warn!("backend {kind:?} unreachable, skipping in ListSessions: {e:#}");
                    continue;
                }
                // Happy path: backend returned a session list.
                Ok(Ok(Ok(sessions))) => sessions,
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
        // Decision 5 (S-M3): the herdr session is singular and collapsed — it has no
        // killable session object (manage its workspaces via CloseSpace). Reject
        // cleanly with `invalid_argument` BEFORE resolving, so the client sees a
        // meaningful error instead of the backend's opaque "no workspace with label
        // 'herdr'" internal error. zellij sessions are never collapsed, so this never
        // affects a zellij KillSession (e.g. a zellij session literally named
        // "herdr" resolves as `zellij:herdr`, which is not a collapsed id).
        if crate::multiplexer::is_collapsed_backend_session(&req.session) {
            return Err(Status::invalid_argument(
                "the herdr session cannot be killed — manage its workspaces via CloseSpace",
            ));
        }
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

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    //! Tests for the concurrent ListSessions fan-out + resilience paths (S-M2/S-M3/S-M4).
    //!
    //! Uses fake `MuxBackend` implementations to exercise:
    //!   - An erroring backend → skipped, other backend's sessions returned (Err path).
    //!   - A panicking backend → skipped via JoinError (panic/cancel path, S-M4).
    //!   - Two healthy backends → sessions from both merged.
    //!   - All backends erroring → empty list (no RPC error).

    use std::sync::Arc;
    use std::time::Duration;

    use tonic::Request;

    use crate::cli::BackendKind;
    use crate::grpc::MuxrService;
    use crate::multiplexer::{
        ActionAck, BackendSet, DualHandle, LayoutSnapshot, MuxBackend, PaneRef, ResizeDir,
        ResizeKind, ScrollDir,
    };
    use crate::proto::Empty;

    // ── Fake MuxBackend implementations ──────────────────────────────────────

    /// Returns a fixed session list from `list_sessions_with_resurrectables`.
    #[derive(Debug)]
    struct StubBackend {
        sessions: Vec<(String, u64, bool)>,
    }

    impl MuxBackend for StubBackend {
        fn list_sessions(&self) -> anyhow::Result<Vec<(String, Duration)>> {
            Ok(self
                .sessions
                .iter()
                .map(|(n, a, _)| (n.clone(), Duration::from_secs(*a)))
                .collect())
        }

        fn list_sessions_with_resurrectables(&self) -> anyhow::Result<Vec<(String, u64, bool)>> {
            Ok(self.sessions.clone())
        }

        fn validate_session_name(&self, _: &str) -> Result<(), String> {
            unimplemented!()
        }
        fn create_session(&self, _: &str, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn kill_session(&self, _: &str) -> anyhow::Result<()> {
            unimplemented!()
        }
        fn rename_session(&self, _: &str, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn write_to_pane(&self, _: &str, _: PaneRef, _: Vec<u8>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn focus_pane(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn close_pane(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn new_pane(&self, _: &str, _: bool, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn rename_pane(&self, _: &str, _: PaneRef, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn resize_pane(
            &self,
            _: &str,
            _: PaneRef,
            _: ResizeKind,
            _: Option<ResizeDir>,
        ) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn toggle_pane_floating(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn toggle_pane_fullscreen(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn scroll_pane(&self, _: &str, _: PaneRef, _: ScrollDir) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn new_tab(&self, _: &str, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn close_tab(&self, _: &str, _: u64) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn go_to_tab(&self, _: &str, _: u64) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn rename_tab(&self, _: &str, _: u64, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn query_layout(&self, _: &str) -> anyhow::Result<LayoutSnapshot> {
            unimplemented!()
        }
        fn query_session_size(&self, _: &str) -> anyhow::Result<(u16, u16)> {
            unimplemented!()
        }
        fn pane_is_floating_with_visibility(
            &self,
            _: &str,
            _: PaneRef,
        ) -> anyhow::Result<(bool, bool, Option<PaneRef>)> {
            unimplemented!()
        }
        fn open_attach(&self, _: &str, _: u16, _: u16, _: bool) -> anyhow::Result<DualHandle> {
            unimplemented!()
        }
        fn backend_version(&self) -> String {
            "stub".to_string()
        }
    }

    /// Always errors from `list_sessions_with_resurrectables` (simulates an
    /// unreachable backend — e.g. herdr socket not present).
    #[derive(Debug)]
    struct ErrorBackend;

    impl MuxBackend for ErrorBackend {
        fn list_sessions(&self) -> anyhow::Result<Vec<(String, Duration)>> {
            anyhow::bail!("connection refused")
        }
        fn list_sessions_with_resurrectables(&self) -> anyhow::Result<Vec<(String, u64, bool)>> {
            anyhow::bail!("connection refused")
        }
        fn validate_session_name(&self, _: &str) -> Result<(), String> {
            unimplemented!()
        }
        fn create_session(&self, _: &str, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn kill_session(&self, _: &str) -> anyhow::Result<()> {
            unimplemented!()
        }
        fn rename_session(&self, _: &str, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn write_to_pane(&self, _: &str, _: PaneRef, _: Vec<u8>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn focus_pane(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn close_pane(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn new_pane(&self, _: &str, _: bool, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn rename_pane(&self, _: &str, _: PaneRef, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn resize_pane(
            &self,
            _: &str,
            _: PaneRef,
            _: ResizeKind,
            _: Option<ResizeDir>,
        ) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn toggle_pane_floating(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn toggle_pane_fullscreen(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn scroll_pane(&self, _: &str, _: PaneRef, _: ScrollDir) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn new_tab(&self, _: &str, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn close_tab(&self, _: &str, _: u64) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn go_to_tab(&self, _: &str, _: u64) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn rename_tab(&self, _: &str, _: u64, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn query_layout(&self, _: &str) -> anyhow::Result<LayoutSnapshot> {
            unimplemented!()
        }
        fn query_session_size(&self, _: &str) -> anyhow::Result<(u16, u16)> {
            unimplemented!()
        }
        fn pane_is_floating_with_visibility(
            &self,
            _: &str,
            _: PaneRef,
        ) -> anyhow::Result<(bool, bool, Option<PaneRef>)> {
            unimplemented!()
        }
        fn open_attach(&self, _: &str, _: u16, _: u16, _: bool) -> anyhow::Result<DualHandle> {
            unimplemented!()
        }
        fn backend_version(&self) -> String {
            "error".to_string()
        }
    }

    /// Panics inside `list_sessions_with_resurrectables` to exercise S-M4
    /// (JoinError tolerance — a panic inside `spawn_blocking` surfaces as a
    /// JoinError rather than an anyhow Err).
    #[derive(Debug)]
    struct PanickingBackend;

    impl MuxBackend for PanickingBackend {
        fn list_sessions(&self) -> anyhow::Result<Vec<(String, Duration)>> {
            panic!("simulated backend panic")
        }
        fn list_sessions_with_resurrectables(&self) -> anyhow::Result<Vec<(String, u64, bool)>> {
            panic!("simulated backend panic")
        }
        fn validate_session_name(&self, _: &str) -> Result<(), String> {
            unimplemented!()
        }
        fn create_session(&self, _: &str, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn kill_session(&self, _: &str) -> anyhow::Result<()> {
            unimplemented!()
        }
        fn rename_session(&self, _: &str, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn write_to_pane(&self, _: &str, _: PaneRef, _: Vec<u8>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn focus_pane(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn close_pane(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn new_pane(&self, _: &str, _: bool, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn rename_pane(&self, _: &str, _: PaneRef, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn resize_pane(
            &self,
            _: &str,
            _: PaneRef,
            _: ResizeKind,
            _: Option<ResizeDir>,
        ) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn toggle_pane_floating(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn toggle_pane_fullscreen(&self, _: &str, _: PaneRef) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn scroll_pane(&self, _: &str, _: PaneRef, _: ScrollDir) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn new_tab(&self, _: &str, _: Option<String>) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn close_tab(&self, _: &str, _: u64) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn go_to_tab(&self, _: &str, _: u64) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn rename_tab(&self, _: &str, _: u64, _: String) -> anyhow::Result<ActionAck> {
            unimplemented!()
        }
        fn query_layout(&self, _: &str) -> anyhow::Result<LayoutSnapshot> {
            unimplemented!()
        }
        fn query_session_size(&self, _: &str) -> anyhow::Result<(u16, u16)> {
            unimplemented!()
        }
        fn pane_is_floating_with_visibility(
            &self,
            _: &str,
            _: PaneRef,
        ) -> anyhow::Result<(bool, bool, Option<PaneRef>)> {
            unimplemented!()
        }
        fn open_attach(&self, _: &str, _: u16, _: u16, _: bool) -> anyhow::Result<DualHandle> {
            unimplemented!()
        }
        fn backend_version(&self) -> String {
            "panic".to_string()
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    /// S-M4 (Err path): an erroring backend is skipped; the reachable backend's
    /// sessions are returned. The RPC must not fail.
    #[tokio::test]
    async fn list_sessions_skips_erroring_backend_returns_other() {
        let error_backend: Arc<dyn MuxBackend> = Arc::new(ErrorBackend);
        let stub_backend: Arc<dyn MuxBackend> = Arc::new(StubBackend {
            sessions: vec![("my-session".to_string(), 100, false)],
        });
        let backends = BackendSet::new(vec![
            (BackendKind::Zellij, error_backend),
            (BackendKind::Herdr, stub_backend),
        ]);
        let service = MuxrService::with_backends(backends);

        let sessions = service
            .list_sessions_impl(Request::new(Empty {}))
            .await
            .expect("RPC must not error when one backend is reachable")
            .into_inner()
            .sessions;

        assert_eq!(sessions.len(), 1, "only the reachable backend's session");
        assert_eq!(sessions[0].name, "my-session");
        assert_eq!(sessions[0].id, "herdr:my-session");
    }

    /// S-M4 (JoinError path): a panicking backend's `spawn_blocking` task yields
    /// a JoinError; the healthy backend's sessions must still be returned and the
    /// RPC must not propagate the panic as an error.
    #[tokio::test]
    async fn list_sessions_tolerates_panicking_backend() {
        let panicking: Arc<dyn MuxBackend> = Arc::new(PanickingBackend);
        let stub: Arc<dyn MuxBackend> = Arc::new(StubBackend {
            sessions: vec![("live-session".to_string(), 0, false)],
        });
        let backends = BackendSet::new(vec![
            (BackendKind::Zellij, panicking),
            (BackendKind::Herdr, stub),
        ]);
        let service = MuxrService::with_backends(backends);

        let sessions = service
            .list_sessions_impl(Request::new(Empty {}))
            .await
            .expect("RPC must not propagate a backend panic as an error")
            .into_inner()
            .sessions;

        assert_eq!(
            sessions.len(),
            1,
            "non-panicking backend's session survives"
        );
        assert_eq!(sessions[0].name, "live-session");
    }

    /// S-M2 (concurrent fan-out, happy path): both backends succeed; sessions from
    /// both are merged. The opaque `id` fields carry the correct backend prefix.
    #[tokio::test]
    async fn list_sessions_both_backends_ok_returns_all() {
        let zellij_backend: Arc<dyn MuxBackend> = Arc::new(StubBackend {
            sessions: vec![("zellij-session".to_string(), 0, false)],
        });
        let herdr_backend: Arc<dyn MuxBackend> = Arc::new(StubBackend {
            sessions: vec![("herdr-session".to_string(), 0, false)],
        });
        let backends = BackendSet::new(vec![
            (BackendKind::Zellij, zellij_backend),
            (BackendKind::Herdr, herdr_backend),
        ]);
        let service = MuxrService::with_backends(backends);

        let sessions = service
            .list_sessions_impl(Request::new(Empty {}))
            .await
            .expect("RPC must succeed when both backends are reachable")
            .into_inner()
            .sessions;

        assert_eq!(sessions.len(), 2, "sessions from both backends merged");
        let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"zellij:zellij-session"));
        assert!(ids.contains(&"herdr:herdr-session"));
    }

    /// All backends error → RPC still succeeds with an empty list (not an error).
    #[tokio::test]
    async fn list_sessions_all_erroring_returns_empty() {
        let backends = BackendSet::new(vec![
            (
                BackendKind::Zellij,
                Arc::new(ErrorBackend) as Arc<dyn MuxBackend>,
            ),
            (
                BackendKind::Herdr,
                Arc::new(ErrorBackend) as Arc<dyn MuxBackend>,
            ),
        ]);
        let service = MuxrService::with_backends(backends);

        let sessions = service
            .list_sessions_impl(Request::new(Empty {}))
            .await
            .expect("RPC must not error even when all backends fail")
            .into_inner()
            .sessions;

        assert!(sessions.is_empty(), "no reachable backends → empty list");
    }
}
