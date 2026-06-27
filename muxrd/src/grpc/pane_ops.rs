//! Pane mutation RPC implementations: write, focus, close, new, rename, resize,
//! toggle floating/fullscreen, scroll.

use tonic::{Request, Response, Status};

use crate::actions::{self, ScrollDir};
use crate::proto::{
    ActionAck as ProtoAck, NewPaneReq, PaneTarget, RenamePaneReq, ResizeKind, ResizePaneReq,
    ScrollDirection, ScrollReq, ToggleFullscreenReq, WriteToPaneReq,
};

use super::MuxrService;
use super::helpers::{
    reject_if_read_only, resolve_pane_target, run_action, try_route_control, validate_session,
};

/// Upper bound on a single `WriteToPane` payload (1 MiB).  Guards against a
/// client pushing an unbounded write into the session IPC channel.
const MAX_WRITE_TO_PANE_BYTES: usize = 1024 * 1024;

impl MuxrService {
    // ── Pane ops (D1) ─────────────────────────────────────────────────────────

    /// Write raw bytes to a specific pane. MUTATING (read-only rejected).
    pub(super) async fn write_to_pane_impl(
        &self,
        request: Request<WriteToPaneReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "WriteToPane")?;
        let req = request.into_inner();
        // Cap payload size to avoid a single RPC pushing an unbounded write into
        // the session IPC channel (review minor).
        if req.data.len() > MAX_WRITE_TO_PANE_BYTES {
            return Err(Status::invalid_argument(format!(
                "WriteToPane: payload {} bytes exceeds the {} byte limit",
                req.data.len(),
                MAX_WRITE_TO_PANE_BYTES
            )));
        }
        let target = req
            .target
            .ok_or_else(|| Status::invalid_argument("WriteToPane: target is required"))?;
        let (session, pane) = resolve_pane_target(&target)?;
        log::info!(
            "WriteToPane: session='{session}' pane={pane:?} ({} bytes)",
            req.data.len()
        );
        let data = req.data;
        run_action("WriteToPane", move || {
            actions::write_to_pane(&session, pane, data)
        })
        .await
    }

    /// Focus a specific pane. Allowed for read-only tokens.
    pub(super) async fn focus_pane_impl(
        &self,
        request: Request<PaneTarget>,
    ) -> Result<Response<ProtoAck>, Status> {
        // Focus is a read — no read-only gate.
        let target = request.into_inner();
        let connection_id = target.connection_id.clone();
        let (session, pane) = resolve_pane_target(&target)?;
        log::info!("FocusPane: session='{session}' pane={pane:?} connection_id='{connection_id}'");
        // Route through the live relay client if attached, so focus applies to
        // the rendering client (and re-points the single-pane sub).
        // connection_id targets the exact relay that sent the request.
        if let Some(resp) = try_route_control(
            &self.control,
            &session,
            &connection_id,
            crate::relay::RelayControl::FocusPane(pane),
        ) {
            log::info!("FocusPane: routed via relay client (session='{session}')");
            return Ok(resp);
        }
        run_action("FocusPane", move || actions::focus_pane(&session, pane)).await
    }

    /// Close a specific pane. MUTATING (read-only rejected).
    pub(super) async fn close_pane_impl(
        &self,
        request: Request<PaneTarget>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "ClosePane")?;
        let target = request.into_inner();
        let (session, pane) = resolve_pane_target(&target)?;
        log::info!("ClosePane: session='{session}' pane={pane:?}");
        run_action("ClosePane", move || actions::close_pane(&session, pane)).await
    }

    /// Open a new pane; the new pane id surfaces in `ActionAck.info`. MUTATING.
    pub(super) async fn new_pane_impl(
        &self,
        request: Request<NewPaneReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "NewPane")?;
        let req = request.into_inner();
        validate_session(&req.session)?;
        let session = req.session;
        let floating = req.floating;
        let pane_name = if req.pane_name.is_empty() {
            None
        } else {
            Some(req.pane_name)
        };
        log::info!("NewPane: session='{session}' floating={floating} name={pane_name:?}");
        run_action("NewPane", move || {
            actions::new_pane(&session, floating, pane_name)
        })
        .await
    }

    /// Rename a specific pane. MUTATING (read-only rejected).
    pub(super) async fn rename_pane_impl(
        &self,
        request: Request<RenamePaneReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "RenamePane")?;
        let req = request.into_inner();
        let target = req
            .target
            .ok_or_else(|| Status::invalid_argument("RenamePane: target is required"))?;
        let (session, pane) = resolve_pane_target(&target)?;
        let name = req.name;
        log::info!("RenamePane: session='{session}' pane={pane:?} name='{name}'");
        run_action("RenamePane", move || {
            actions::rename_pane(&session, pane, name)
        })
        .await
    }

    /// Resize a specific pane. MUTATING (read-only rejected).
    pub(super) async fn resize_pane_impl(
        &self,
        request: Request<ResizePaneReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "ResizePane")?;
        let req = request.into_inner();
        let target = req
            .target
            .ok_or_else(|| Status::invalid_argument("ResizePane: target is required"))?;
        let (session, pane) = resolve_pane_target(&target)?;

        use zellij_utils::data::{Direction, Resize};
        let resize = match ResizeKind::try_from(req.resize) {
            Ok(ResizeKind::Decrease) => Resize::Decrease,
            _ => Resize::Increase,
        };
        // ResizeDirection: 0 = UNSPECIFIED → None (uniform resize).
        let direction = match req.direction {
            1 => Some(Direction::Left),
            2 => Some(Direction::Right),
            3 => Some(Direction::Up),
            4 => Some(Direction::Down),
            _ => None,
        };
        log::info!(
            "ResizePane: session='{session}' pane={pane:?} resize={resize:?} dir={direction:?}"
        );
        run_action("ResizePane", move || {
            actions::resize_pane(&session, pane, resize, direction)
        })
        .await
    }

    /// Toggle a pane between floating and embedded. MUTATING (read-only rejected).
    pub(super) async fn toggle_pane_floating_impl(
        &self,
        request: Request<PaneTarget>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "TogglePaneFloating")?;
        let target = request.into_inner();
        let (session, pane) = resolve_pane_target(&target)?;
        log::info!("TogglePaneFloating: session='{session}' pane={pane:?}");
        run_action("TogglePaneFloating", move || {
            actions::toggle_pane_floating(&session, pane)
        })
        .await
    }

    /// Toggle fullscreen for a pane. MUTATING (read-only rejected).
    pub(super) async fn toggle_pane_fullscreen_impl(
        &self,
        request: Request<ToggleFullscreenReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "TogglePaneFullscreen")?;
        let req = request.into_inner();
        let target = req
            .target
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("TogglePaneFullscreen: missing target"))?;
        let connection_id = target.connection_id.clone();
        let (session, pane) = resolve_pane_target(target)?;
        // Bug 2c: forward the client's floating hint so the relay can skip a
        // synchronous IPC query on its hot path. Only trust the hint when the
        // caller explicitly attests it via `has_floating_hint` — proto3 bools
        // default to false, so an all-false hint from a target-only request must
        // NOT be read as "definitely tiled" (that would mis-route a floating
        // pane). Without the flag we pass `None`, and the relay runs the live
        // query as a safety net.
        let hint = if req.has_floating_hint {
            Some(crate::relay::FloatingHint {
                target_is_floating: req.target_is_floating,
                floating_visible: req.floating_visible,
                target_is_focused_floating: req.target_is_focused_floating,
            })
        } else {
            None
        };
        log::info!(
            "TogglePaneFullscreen: session='{session}' pane={pane:?} hint={hint:?} \
             connection_id='{connection_id}'"
        );
        // Route through the live relay client if attached so the fullscreen
        // toggle applies to the *rendering* client (is_cli_client:false).
        // connection_id targets the exact relay that sent the request.
        if let Some(resp) = try_route_control(
            &self.control,
            &session,
            &connection_id,
            crate::relay::RelayControl::ToggleFullscreen { pane, hint },
        ) {
            log::info!("TogglePaneFullscreen: routed via relay client (session='{session}')");
            return Ok(resp);
        }
        run_action("TogglePaneFullscreen", move || {
            actions::toggle_pane_fullscreen(&session, pane)
        })
        .await
    }

    // ── Scroll (D2) ───────────────────────────────────────────────────────────

    /// Scroll a specific pane. Allowed for read-only tokens.
    pub(super) async fn scroll_pane_impl(
        &self,
        request: Request<ScrollReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        // NOTE: scroll is explicitly allowed for read-only tokens — no gate here.
        let req = request.into_inner();
        let target = req
            .target
            .ok_or_else(|| Status::invalid_argument("ScrollPane: target is required"))?;
        let (session, pane) = resolve_pane_target(&target)?;
        let dir = match ScrollDirection::try_from(req.direction) {
            Ok(ScrollDirection::Down) => ScrollDir::Down,
            Ok(ScrollDirection::ToTop) => ScrollDir::ToTop,
            Ok(ScrollDirection::ToBottom) => ScrollDir::ToBottom,
            Ok(ScrollDirection::PageUp) => ScrollDir::PageUp,
            Ok(ScrollDirection::PageDown) => ScrollDir::PageDown,
            Ok(ScrollDirection::HalfPageUp) => ScrollDir::HalfPageUp,
            Ok(ScrollDirection::HalfPageDown) => ScrollDir::HalfPageDown,
            // Up = 0 (default) and anything unrecognised → Up
            _ => ScrollDir::Up,
        };
        log::info!("ScrollPane: session='{session}' pane={pane:?} dir={dir:?}");
        run_action("ScrollPane", move || {
            actions::scroll_pane(&session, pane, dir)
        })
        .await
    }
}
