//! Space (herdr workspace) RPC implementations: get / switch / create / rename / close.
//!
//! Spaces are a herdr-only navigation axis (its workspaces, surfaced as in-place
//! switchable sub-navigation within the single collapsed herdr session). zellij —
//! and any backend without a space concept — returns the empty list for `GetSpaces`
//! and a graceful failure ack for the mutating ops (the [`MuxBackend`] /
//! [`MuxSender`] defaults flow through unchanged; no special-casing here).
//!
//! Routing:
//! - **GetSpaces** is a read: it resolves the owning backend, lists its spaces, and
//!   marks the **connection-active** space using the relay's tracked
//!   `current_space` (per-connection view; see [`RelayViewState`]). With no relay,
//!   it falls back to the backend-reported active.
//! - **SwitchSpace** is relay-routed (like `GetLayout`/`GoToTab`): it sends
//!   [`RelayControl::SwitchSpace`] to the connection's relay and awaits the oneshot
//!   ack — the relay re-points its wire stream at the target workspace with no
//!   daemon-global focus change.
//! - **CreateSpace / RenameSpace / CloseSpace** are control-plane: they mutate the
//!   daemon's globally-shared workspaces directly through the backend (spaces are
//!   daemon-global objects). After a create the client issues GetSpaces +
//!   SwitchSpace.
//!
//! [`MuxBackend`]: crate::multiplexer::MuxBackend
//! [`MuxSender`]: crate::multiplexer::MuxSender
//! [`RelayViewState`]: crate::relay::RelayViewState

use tonic::{Request, Response, Status};

use crate::proto::{
    ActionAck as ProtoAck, CloseSpaceReq, CreateSpaceReq, RenameSpaceReq, SessionRef, Space,
    SpaceList, SwitchSpaceReq,
};
use crate::relay::RelayControl;

use super::MuxrService;
use super::helpers::{reject_if_read_only, run_action};

/// Timeout for the oneshot reply when routing a `SwitchSpace` through the relay.
///
/// Mirrors `RELAY_QUERY_TIMEOUT` in `grpc/layout.rs`: a space switch is a re-attach
/// (resolve the target workspace's focused pane + re-point the wire stream), bounded
/// at the backend by herdr's per-call control timeout; 18 s comfortably covers it
/// plus channel overhead.
const SWITCH_SPACE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(18);

impl MuxrService {
    // ── GetSpaces ─────────────────────────────────────────────────────────────

    /// List the spaces for a session, marking the connection-active one.
    ///
    /// zellij sessions return an empty list (the `MuxBackend::list_spaces` default).
    pub(super) async fn get_spaces_impl(
        &self,
        request: Request<SessionRef>,
    ) -> Result<Response<SpaceList>, Status> {
        let req = request.into_inner();
        let session = req.session;
        let connection_id = req.connection_id;
        let (backend, bare) = self.resolve_session(&session)?;
        log::info!("GetSpaces: session='{session}' connection_id='{connection_id}'");

        // Blocking IPC (herdr `workspace.list`) → spawn_blocking.
        let snapshots = {
            let backend = backend.clone();
            let bare = bare.clone();
            tokio::task::spawn_blocking(move || backend.list_spaces(&bare))
                .await
                .map_err(|e| Status::internal(format!("GetSpaces: list task panicked: {e}")))?
                .map_err(|e| {
                    log::warn!("GetSpaces: list_spaces failed for '{session}': {e:#}");
                    Status::internal(format!("GetSpaces: {e:#}"))
                })?
        };

        // Per-connection active override: the relay tracks the workspace it switched
        // to (the daemon-global focus is intentionally left untouched on switch, so
        // the backend-reported `active` would otherwise be wrong for this client).
        // When no relay is attached (or it has not switched yet), fall back to the
        // backend-reported active.
        let relay_space = self.connection_current_space(&session, &connection_id);
        if let Some(ref ws) = relay_space {
            log::debug!("GetSpaces: connection-active space override → '{ws}'");
        }

        let spaces: Vec<Space> = snapshots
            .into_iter()
            .map(|s| {
                let active = match relay_space {
                    Some(ref ws) => &s.id == ws,
                    None => s.active,
                };
                Space {
                    id: s.id,
                    name: s.name,
                    active,
                }
            })
            .collect();

        log::info!("GetSpaces: session='{session}' → {} space(s)", spaces.len());
        Ok(Response::new(SpaceList { spaces }))
    }

    // ── SwitchSpace ───────────────────────────────────────────────────────────

    /// Switch the connection's relay to a different space. MUTATING.
    ///
    /// Routed through the connection's live relay by an **exact** connection_id match
    /// (fail-closed — no session-scoped fallback; see `resolve_space_relay`). With no
    /// matching connection, returns `ActionAck{ok:false, "reattach required …"}`.
    pub(super) async fn switch_space_impl(
        &self,
        request: Request<SwitchSpaceReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "SwitchSpace")?;
        let req = request.into_inner();
        let session = req.session;
        let connection_id = req.connection_id;
        let space_id = req.space_id;
        if space_id.is_empty() {
            return Err(Status::invalid_argument("space_id must not be empty"));
        }
        // Resolve to validate the session id / owning backend exists (the actual
        // switch is relay-routed, but a bad id must still be a clean error).
        let _ = self.resolve_session(&session)?;
        log::info!(
            "SwitchSpace: session='{session}' space_id='{space_id}' connection_id='{connection_id}'"
        );

        // Locate the connection's relay control sender by an EXACT connection_id
        // match (fail-closed; see `resolve_space_relay`). No session-scoped fallback:
        // on a collapsed herdr session that would re-point a co-attached client's
        // stream (S-M2/S-M4). On no match return ok:false — never steer an arbitrary
        // relay.
        let sender = match self.resolve_space_relay(&session, &connection_id) {
            Some(s) => s,
            None => {
                log::info!(
                    "SwitchSpace: no matching connection for '{session}' \
                     (connection_id='{connection_id}') — fail-closed"
                );
                return Ok(Response::new(ProtoAck {
                    ok: false,
                    error: "reattach required (no matching connection)".to_owned(),
                    info: String::new(),
                }));
            }
        };

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel::<anyhow::Result<()>>();
        if sender
            .send(RelayControl::SwitchSpace {
                workspace_id: space_id.clone(),
                reply: reply_tx,
            })
            .is_err()
        {
            log::warn!("SwitchSpace: relay sender closed for '{session}'");
            return Ok(Response::new(ProtoAck {
                ok: false,
                error: "SwitchSpace: relay unavailable (tearing down)".to_owned(),
                info: String::new(),
            }));
        }

        match tokio::time::timeout(SWITCH_SPACE_TIMEOUT, reply_rx).await {
            Ok(Ok(Ok(()))) => {
                log::info!("SwitchSpace: session='{session}' space_id='{space_id}' ok");
                Ok(Response::new(ProtoAck {
                    ok: true,
                    error: String::new(),
                    info: String::new(),
                }))
            }
            Ok(Ok(Err(e))) => {
                log::warn!("SwitchSpace: relay reported failure for '{session}': {e:#}");
                Ok(Response::new(ProtoAck {
                    ok: false,
                    error: format!("SwitchSpace failed: {e:#}"),
                    info: String::new(),
                }))
            }
            Ok(Err(_cancelled)) => {
                log::warn!("SwitchSpace: relay oneshot cancelled for '{session}'");
                Ok(Response::new(ProtoAck {
                    ok: false,
                    error: "SwitchSpace: relay cancelled the request".to_owned(),
                    info: String::new(),
                }))
            }
            Err(_elapsed) => {
                log::warn!(
                    "SwitchSpace: relay timed out for '{session}' after {SWITCH_SPACE_TIMEOUT:?}"
                );
                Ok(Response::new(ProtoAck {
                    ok: false,
                    error: "SwitchSpace: timed out waiting for the relay".to_owned(),
                    info: String::new(),
                }))
            }
        }
    }

    // ── CreateSpace ───────────────────────────────────────────────────────────

    /// Create a new space (herdr workspace). MUTATING. Control-plane (daemon-global).
    pub(super) async fn create_space_impl(
        &self,
        request: Request<CreateSpaceReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "CreateSpace")?;
        let req = request.into_inner();
        let (backend, _bare) = self.resolve_session(&req.session)?;
        let label = if req.label.is_empty() {
            None
        } else {
            Some(req.label)
        };
        log::info!("CreateSpace: session='{}' label={label:?}", req.session);
        run_action("CreateSpace", move || backend.create_space(label)).await
    }

    // ── RenameSpace ───────────────────────────────────────────────────────────

    /// Rename an existing space. MUTATING. Control-plane (daemon-global).
    pub(super) async fn rename_space_impl(
        &self,
        request: Request<RenameSpaceReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "RenameSpace")?;
        let req = request.into_inner();
        let (backend, _bare) = self.resolve_session(&req.session)?;
        let space_id = req.space_id;
        let label = req.label;
        if space_id.is_empty() {
            return Err(Status::invalid_argument("space_id must not be empty"));
        }
        if label.is_empty() {
            return Err(Status::invalid_argument("space label must not be empty"));
        }
        log::info!(
            "RenameSpace: session='{}' space_id='{space_id}' label='{label}'",
            req.session
        );
        run_action("RenameSpace", move || {
            backend.rename_space(&space_id, &label)
        })
        .await
    }

    // ── CloseSpace ────────────────────────────────────────────────────────────

    /// Close (delete) a space. MUTATING. Control-plane (daemon-global).
    pub(super) async fn close_space_impl(
        &self,
        request: Request<CloseSpaceReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "CloseSpace")?;
        let req = request.into_inner();
        let (backend, _bare) = self.resolve_session(&req.session)?;
        let space_id = req.space_id;
        if space_id.is_empty() {
            return Err(Status::invalid_argument("space_id must not be empty"));
        }
        log::info!(
            "CloseSpace: session='{}' space_id='{space_id}'",
            req.session
        );
        run_action("CloseSpace", move || backend.close_space(&space_id)).await
    }

    // ── Private routing helpers ─────────────────────────────────────────────────

    /// The space (herdr workspace) the connection's relay is currently viewing, if
    /// any. Looks up the per-connection [`RelayViewState`] by an **exact**
    /// `connection_id` match (validated against `session`).
    ///
    /// S-M2/S-M4: spaces are herdr-only, and herdr collapses every connection onto
    /// the single `herdr:herdr` session — so a session-scoped fallback here would
    /// read **another** connection's `current_space` and mark the wrong space active
    /// for this caller. We therefore drop the fallback: on an absent/mismatched
    /// connection_id we return `None`, and `get_spaces_impl` falls back to the
    /// backend-reported active (GetSpaces) rather than a sibling relay's view-state.
    ///
    /// [`RelayViewState`]: crate::relay::RelayViewState
    fn connection_current_space(&self, session: &str, connection_id: &str) -> Option<String> {
        // Exact per-connection lookup only (clone out of the DashMap guard — never
        // held across an `.await`; this is a sync helper anyway). No session-scoped
        // fallback: it would leak a sibling connection's current_space.
        if connection_id.is_empty() {
            return None;
        }
        self.view_state
            .get(connection_id)
            .filter(|entry| entry.session == session)
            .and_then(|entry| entry.state.current_space.clone())
    }

    /// Resolve the control sender for the connection's relay for a SwitchSpace.
    ///
    /// SwitchSpace is herdr-only and MUTATING, and herdr collapses every connection
    /// onto the single `herdr:herdr` session. A session-scoped fallback would
    /// re-point a **co-attached** connection's wire stream when this caller's
    /// connection_id is empty/stale (the S-M2/S-M4 isolation violation). So this is
    /// **fail-closed**: an exact `connection_id` match (validated against `session`)
    /// is required, with no fallback. Returns `None` when connection_id is empty or
    /// does not match a live relay (caller returns `ActionAck{ok:false}`).
    fn resolve_space_relay(
        &self,
        session: &str,
        connection_id: &str,
    ) -> Option<tokio::sync::mpsc::UnboundedSender<RelayControl>> {
        if connection_id.is_empty() {
            return None;
        }
        self.control
            .get(connection_id)
            .filter(|entry| entry.session == session)
            .map(|entry| entry.sender.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::multiplexer::SpaceSnapshot;
    use crate::proto::Space;

    /// The proto mapping marks the relay-current space active and clears the
    /// backend-reported active when a per-connection override is present.
    fn map_with_override(snaps: Vec<SpaceSnapshot>, relay_space: Option<&str>) -> Vec<Space> {
        snaps
            .into_iter()
            .map(|s| {
                let active = match relay_space {
                    Some(ws) => s.id == ws,
                    None => s.active,
                };
                Space {
                    id: s.id,
                    name: s.name,
                    active,
                }
            })
            .collect()
    }

    fn snap(id: &str, name: &str, active: bool) -> SpaceSnapshot {
        SpaceSnapshot {
            id: id.to_owned(),
            name: name.to_owned(),
            active,
        }
    }

    #[test]
    fn override_marks_relay_space_active() {
        // Backend reports "a" active, but the relay switched to "b": "b" wins.
        let mapped =
            map_with_override(vec![snap("a", "A", true), snap("b", "B", false)], Some("b"));
        assert!(!mapped[0].active, "backend-active 'a' must be cleared");
        assert!(mapped[1].active, "relay-current 'b' must be active");
    }

    #[test]
    fn no_override_uses_backend_active() {
        // No relay-current space → the backend-reported active is preserved.
        let mapped = map_with_override(vec![snap("a", "A", true), snap("b", "B", false)], None);
        assert!(mapped[0].active, "backend-active 'a' must be preserved");
        assert!(!mapped[1].active);
    }

    #[test]
    fn empty_backend_list_maps_to_empty() {
        // zellij path: list_spaces returns empty → no spaces, regardless of override.
        assert!(map_with_override(vec![], None).is_empty());
        assert!(map_with_override(vec![], Some("x")).is_empty());
    }

    // ─── S-M2/S-M4: fail-closed relay/view-state resolution ──────────────────

    use crate::grpc::MuxrService;
    use crate::relay::{ControlEntry, RelayControl, RelayViewState, ViewStateEntry};
    use tokio::sync::mpsc;

    #[test]
    fn resolve_space_relay_requires_exact_connection_id() {
        // SwitchSpace is herdr-only + mutating: an empty/guessed connection_id must
        // NOT resolve to the victim's relay (no session-scoped fallback — S-M2/S-M4).
        let service = MuxrService::new();
        let (tx, _rx) = mpsc::unbounded_channel::<RelayControl>();
        service.control.insert(
            "victim-conn".to_owned(),
            ControlEntry {
                session: "herdr:herdr".to_owned(),
                sender: tx,
                read_only: false,
            },
        );
        // Exact match resolves.
        assert!(
            service
                .resolve_space_relay("herdr:herdr", "victim-conn")
                .is_some(),
            "exact connection_id must resolve"
        );
        // Empty connection_id → None (fail-closed; no steer onto the victim).
        assert!(
            service.resolve_space_relay("herdr:herdr", "").is_none(),
            "empty connection_id must fail closed (no session fallback)"
        );
        // Guessed/stale connection_id → None.
        assert!(
            service
                .resolve_space_relay("herdr:herdr", "guessed-1")
                .is_none(),
            "wrong connection_id must fail closed"
        );
    }

    #[test]
    fn connection_current_space_requires_exact_connection_id() {
        // GetSpaces read fallback: an empty/wrong connection_id must NOT read the
        // victim connection's current_space (it falls back to backend-active instead).
        let service = MuxrService::new();
        let state = RelayViewState {
            current_space: Some("ws-2".to_owned()),
            ..RelayViewState::default()
        };
        service.view_state.insert(
            "victim-conn".to_owned(),
            ViewStateEntry {
                session: "herdr:herdr".to_owned(),
                state,
            },
        );
        // Exact match reads the connection's space.
        assert_eq!(
            service
                .connection_current_space("herdr:herdr", "victim-conn")
                .as_deref(),
            Some("ws-2"),
            "exact connection_id reads the connection's current_space"
        );
        // Empty / wrong connection_id → None (won't leak the victim's view-state).
        assert!(
            service
                .connection_current_space("herdr:herdr", "")
                .is_none(),
            "empty connection_id must not read a sibling's current_space"
        );
        assert!(
            service
                .connection_current_space("herdr:herdr", "other-conn")
                .is_none(),
            "wrong connection_id must not read a sibling's current_space"
        );
    }
}
