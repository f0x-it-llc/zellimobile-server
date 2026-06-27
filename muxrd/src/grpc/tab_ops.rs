//! Tab operation RPC implementations: new, close, go_to, rename.

use tonic::{Request, Response, Status};

use crate::proto::{ActionAck as ProtoAck, NewTabReq, RenameTabReq, TabTarget};

use super::MuxrService;
use super::helpers::{
    reject_if_read_only, resolve_tab_target, run_action, try_route_control, validate_session,
};

impl MuxrService {
    // ── Tab ops (D2) ──────────────────────────────────────────────────────────

    /// Open a new tab; new tab id/name surface in ActionAck.info. MUTATING.
    pub(super) async fn new_tab_impl(
        &self,
        request: Request<NewTabReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "NewTab")?;
        let req = request.into_inner();
        validate_session(&req.session)?;
        let session = req.session;
        let tab_name = if req.tab_name.is_empty() {
            None
        } else {
            Some(req.tab_name)
        };
        log::info!("NewTab: session='{session}' name={tab_name:?}");
        let backend = self.backend.clone();
        run_action("NewTab", move || backend.new_tab(&session, tab_name)).await
    }

    /// Close a tab by id. MUTATING (read-only rejected).
    pub(super) async fn close_tab_impl(
        &self,
        request: Request<TabTarget>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "CloseTab")?;
        let req = request.into_inner();
        let (session, tab_id) = resolve_tab_target(&req)?;
        log::info!("CloseTab: session='{session}' tab_id={tab_id}");
        let backend = self.backend.clone();
        run_action("CloseTab", move || backend.close_tab(&session, tab_id)).await
    }

    /// Switch focus to a tab by id. MUTATING (read-only rejected).
    pub(super) async fn go_to_tab_impl(
        &self,
        request: Request<TabTarget>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "GoToTab")?;
        let req = request.into_inner();
        let connection_id = req.connection_id.clone();
        let (session, tab_id) = resolve_tab_target(&req)?;
        log::info!("GoToTab: session='{session}' tab_id={tab_id} connection_id='{connection_id}'");
        // Route through the live relay client if attached, so the tab switch
        // applies to the *rendering* client (deterministic, no ephemeral).
        // connection_id targets the exact relay that sent the request; falls
        // back to any relay for the session when id is absent/stale.
        if let Some(resp) = try_route_control(
            &self.control,
            &session,
            &connection_id,
            crate::relay::RelayControl::SwitchTab(tab_id),
        ) {
            log::info!("GoToTab: routed via relay client (session='{session}', tab_id={tab_id})");
            return Ok(resp);
        }
        let backend = self.backend.clone();
        run_action("GoToTab", move || backend.go_to_tab(&session, tab_id)).await
    }

    /// Rename a tab by id. MUTATING (read-only rejected).
    pub(super) async fn rename_tab_impl(
        &self,
        request: Request<RenameTabReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "RenameTab")?;
        let req = request.into_inner();
        validate_session(&req.session)?;
        let session = req.session;
        let tab_id = req.tab_id;
        let name = req.name;
        if name.is_empty() {
            return Err(Status::invalid_argument("tab name must not be empty"));
        }
        log::info!("RenameTab: session='{session}' tab_id={tab_id} name='{name}'");
        let backend = self.backend.clone();
        run_action("RenameTab", move || {
            backend.rename_tab(&session, tab_id, name)
        })
        .await
    }
}
