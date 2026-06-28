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
    /// Routed through the connection's live relay (per-connection, then writable
    /// session-scoped fallback). With no relay attached, returns `ActionAck{ok:false}`.
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

        // Locate the connection's relay control sender (per-connection, then any
        // WRITABLE relay for the session). A read-only relay's inbound task would
        // drop the SwitchSpace at its own guard, so the fallback skips read-only
        // entries to avoid a false success.
        let sender = match self.resolve_space_relay(&session, &connection_id) {
            Some(s) => s,
            None => {
                log::info!("SwitchSpace: no live relay for '{session}' — not attached");
                return Ok(Response::new(ProtoAck {
                    ok: false,
                    error: "SwitchSpace: no live attach for this session — \
                            attach the session before switching spaces"
                        .to_owned(),
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
    /// any. Looks up the per-connection [`RelayViewState`] the same way
    /// `get_layout_impl` does: exact `connection_id` (validated against `session`)
    /// first, then a session-scoped fallback. Returns `None` when no relay is
    /// attached or it has not switched spaces yet (caller falls back to the
    /// backend-reported active).
    ///
    /// [`RelayViewState`]: crate::relay::RelayViewState
    fn connection_current_space(&self, session: &str, connection_id: &str) -> Option<String> {
        // Per-connection lookup (clone out of the DashMap guard — never held across
        // an `.await`; this is a sync helper anyway).
        if !connection_id.is_empty()
            && let Some(space) = self
                .view_state
                .get(connection_id)
                .filter(|entry| entry.session == session)
                .map(|entry| entry.state.current_space.clone())
        {
            return space;
        }
        // Session-scoped fallback: any relay attached to this session.
        self.view_state
            .iter()
            .find(|entry| entry.session == session)
            .and_then(|entry| entry.state.current_space.clone())
    }

    /// Resolve the control sender for the connection's relay for a SwitchSpace.
    ///
    /// Per-connection (validated `connection_id` + session match) first, then a
    /// **writable** session-scoped fallback. Read-only relays are skipped in the
    /// fallback: their inbound task drops `SwitchSpace`, which would otherwise yield
    /// a false success. Returns `None` when no suitable relay is attached.
    fn resolve_space_relay(
        &self,
        session: &str,
        connection_id: &str,
    ) -> Option<tokio::sync::mpsc::UnboundedSender<RelayControl>> {
        if !connection_id.is_empty()
            && let Some(sender) = self
                .control
                .get(connection_id)
                .filter(|entry| entry.session == session)
                .map(|entry| entry.sender.clone())
        {
            return Some(sender);
        }
        self.control
            .iter()
            .find(|entry| entry.session == session && !entry.read_only)
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
}
