//! herdr [`MuxBackend`] — composes the P2.02 control client, the P2.03 wire
//! relay, and the shared id registries into a functionally complete second
//! backend behind the same trait [`ZellijBackend`](crate::multiplexer::ZellijBackend)
//! implements.
//!
//! ## What this ties together
//!
//! - [`HerdrControl`] (P2.02) — the JSON-API control plane: workspace / tab /
//!   pane lifecycle + the neutral [`LayoutSnapshot`] transcode.
//! - [`relay::open_attach`] (P2.03) — the wire data plane: the single-pane ANSI
//!   relay returning a [`DualHandle`] the Phase-1 relay drives unchanged.
//! - [`HerdrPaneRegistry`] / [`HerdrTabRegistry`] — the stable `u32`/`u64` ↔
//!   herdr-`String` id maps, shared (behind `Arc`) by the control client, the
//!   wire relay, and this backend so a `PaneRef`/`tab_id` round-trips identically
//!   across layout polls and actions.
//!
//! ## session ↔ workspace
//!
//! muxrd's "session" is herdr's **workspace** (see `research/RESEARCH.md` §2 — both
//! are the top-level container the Sessions screen lists; they align 1:1). The
//! neutral session *name* is the workspace **label**; [`HerdrBackend::workspace_id_for`]
//! resolves a name → herdr `workspace_id` by matching `WorkspaceInfo.label`.
//!
//! ## Capability gaps (herdr has no equivalent)
//!
//! herdr's JSON API does not cover every zellij ephemeral action. Where there is
//! no honest mapping this backend returns a **failed** [`ActionAck`]
//! (`ok: false`) so the client surfaces it rather than silently dropping the
//! request — except where a **benign no-op** (`ok: true`) is clearly correct
//! because the operation is meaningless or is handled elsewhere (the wire relay):
//!
//! | method | disposition | why |
//! |---|---|---|
//! | `write_to_pane` | `ok:false` | input is wire-socket only ([`relay`]), no JSON equivalent |
//! | `resize_pane` | `ok:false` | resize is wire-socket only ([`relay`]) |
//! | `scroll_pane` | `ok:false` | scrollback is wire-socket only ([`relay`]) |
//! | `focus_pane` | `ok:true` no-op | ephemeral path only; the relay's re-attach is the real focus |
//! | `toggle_pane_floating` | `ok:true` no-op | herdr has no floating layer |
//!
//! The ephemeral `focus_pane`/`write`/`resize`/`scroll` paths are used **only when
//! no relay is attached**; with a live attach the gRPC layer routes input, focus,
//! and scroll through the relay's persistent wire connection (`grpc/pane_ops.rs`),
//! so none of the `ok:false` gaps sits on the core interactive hot path.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};

use crate::multiplexer::types::{
    ActionAck, LayoutSnapshot, PaneRef, ResizeDir, ResizeKind, ScrollDir,
};
use crate::multiplexer::{DualHandle, MuxBackend};

use super::api::{PaneLayoutRect, PaneZoomMode, SplitDirection};
use super::control::HerdrControl;
use super::paths::HerdrSocketPaths;
use super::registry::{HerdrPaneRegistry, HerdrTabRegistry};
use super::relay;
use super::wire::HERDR_PROTOCOL_VERSION;

/// Default `(rows, cols)` reported when herdr exposes no usable layout area for a
/// session (no panes yet, or a zero-sized area). A conventional 24×80 keeps the
/// gRPC `GetSessionSize` reply sane rather than `0×0`.
const DEFAULT_SESSION_SIZE: (u16, u16) = (24, 80);

/// Default split direction for [`MuxBackend::new_pane`]. herdr only models
/// `Right` / `Down`; `Right` mirrors zellij's default "split vertically".
const DEFAULT_SPLIT_DIRECTION: SplitDirection = SplitDirection::Right;

/// The herdr-backed [`MuxBackend`].
///
/// Holds the control client and the two id registries (shared behind `Arc` with
/// the control client and every wire relay it spawns), plus the resolved socket
/// pair so both planes target the same herdr instance. Cheap to construct; holds
/// no live connection (every control call is a fresh JSON round-trip, every attach
/// a fresh wire socket).
#[derive(Debug)]
pub struct HerdrBackend {
    /// JSON-API control plane (workspace/tab/pane lifecycle + layout transcode).
    control: Arc<HerdrControl>,
    /// Shared pane-id registry (`u32 ↔ herdr pane_id`/`terminal_id`).
    panes: Arc<HerdrPaneRegistry>,
    /// Shared tab-id registry (`u64 ↔ herdr tab_id`).
    tabs: Arc<HerdrTabRegistry>,
    /// Resolved herdr socket pair (JSON-API + wire). The wire path is handed to
    /// every [`relay::open_attach`].
    paths: HerdrSocketPaths,
}

impl HerdrBackend {
    /// Construct a backend for the herdr instance reachable at `paths`, building a
    /// control client that shares freshly-created registries with this backend
    /// (and, transitively, with every wire relay it opens).
    pub fn new(paths: HerdrSocketPaths) -> Self {
        let panes = Arc::new(HerdrPaneRegistry::new());
        let tabs = Arc::new(HerdrTabRegistry::new());
        let control = Arc::new(HerdrControl::new(
            paths.api.clone(),
            Arc::clone(&panes),
            Arc::clone(&tabs),
        ));
        Self {
            control,
            panes,
            tabs,
            paths,
        }
    }

    /// Construct a backend resolving herdr's socket pair from the process
    /// environment (`HERDR_SOCKET_PATH` / XDG default). This is the entry point
    /// the P2.05 selector will call when the operator selects the herdr backend.
    pub fn from_env() -> Self {
        Self::new(HerdrSocketPaths::resolve())
    }

    /// The shared pane registry (same `Arc` the control client + relays hold).
    pub fn pane_registry(&self) -> &Arc<HerdrPaneRegistry> {
        &self.panes
    }

    /// The shared tab registry.
    pub fn tab_registry(&self) -> &Arc<HerdrTabRegistry> {
        &self.tabs
    }

    /// Resolve a neutral session **name** to its herdr `workspace_id` by matching
    /// the workspace **label** (the name muxrd shows).
    ///
    /// herdr `label` is a *display name*, not a unique key (`workspace_id` is) — so
    /// the match may be zero, one, or many. Exactly-one is the only safe case:
    /// - zero matches → `Err` (no such session);
    /// - **more than one** match → `Err` rather than silently binding to whichever
    ///   is first (M1: that risked wrong-workspace targeting / destruction).
    ///
    /// muxrd's own [`create_session`](MuxBackend::create_session) refuses to
    /// manufacture a duplicate label, so the ambiguous case only arises from
    /// out-of-band herdr usage; we fail closed when it does.
    fn workspace_id_for(&self, session: &str) -> Result<String> {
        let mut matching = self
            .control
            .list_workspaces()?
            .into_iter()
            .filter(|w| w.label == session);
        let first = matching
            .next()
            .ok_or_else(|| anyhow!("herdr: no workspace with label '{session}'"))?;
        if matching.next().is_some() {
            return Err(anyhow!(
                "herdr: ambiguous session: multiple herdr workspaces labeled '{session}'"
            ));
        }
        Ok(first.workspace_id)
    }
}

/// Derive a `(rows, cols)` session size from a herdr layout `area` rectangle,
/// falling back to [`DEFAULT_SESSION_SIZE`] for any zero dimension. Pure so it is
/// unit-testable from a fixture rect.
fn session_size_from_area(area: PaneLayoutRect) -> (u16, u16) {
    let rows = if area.height == 0 {
        DEFAULT_SESSION_SIZE.0
    } else {
        area.height
    };
    let cols = if area.width == 0 {
        DEFAULT_SESSION_SIZE.1
    } else {
        area.width
    };
    (rows, cols)
}

/// A benign no-op acknowledgement for an operation that is meaningless on herdr
/// (and so must not be reported as a failure).
fn noop_ack() -> ActionAck {
    ActionAck {
        ok: true,
        error: None,
        info: None,
    }
}

/// A failed acknowledgement for a real capability gap, so the client surfaces it.
fn unsupported_ack(op: &str) -> ActionAck {
    ActionAck {
        ok: false,
        error: Some(format!("{op} is unsupported on the herdr backend")),
        info: None,
    }
}

impl MuxBackend for HerdrBackend {
    // ── Session lifecycle ───────────────────────────────────────────────────

    fn list_sessions(&self) -> Result<Vec<(String, Duration)>> {
        // herdr exposes no workspace age/uptime → Duration::ZERO for every entry.
        Ok(self
            .control
            .list_workspaces()?
            .into_iter()
            .map(|w| (w.label, Duration::ZERO))
            .collect())
    }

    fn list_sessions_with_resurrectables(&self) -> Result<Vec<(String, u64, bool)>> {
        // herdr has no resurrectable/dead-session concept → resurrectable=false,
        // age=0 (no uptime exposed).
        Ok(self
            .control
            .list_workspaces()?
            .into_iter()
            .map(|w| (w.label, 0u64, false))
            .collect())
    }

    fn validate_session_name(&self, name: &str) -> std::result::Result<(), String> {
        // Reuse muxrd's strict `[A-Za-z0-9_-]` guard (the security invariant —
        // path-traversal / metacharacter defence). herdr labels may be looser, but
        // muxrd's boundary stays strict regardless of backend.
        crate::ipc::validate_session_name(name)
    }

    fn create_session(&self, name: &str, layout: Option<String>) -> Result<ActionAck> {
        // herdr has no zellij `--layout`; the `layout` path is ignored (documented).
        // The name becomes the workspace label.
        if let Some(layout) = layout {
            log::debug!("HerdrBackend::create_session: ignoring layout '{layout}' (unsupported)");
        }
        // M1: refuse to manufacture a duplicate label. herdr labels are not unique,
        // and `workspace_id_for` resolves a session by label — so creating a second
        // workspace with an existing label would make every later
        // kill/rename/attach/query for that name ambiguous (and fail closed). Block
        // it here so muxrd never creates the ambiguous condition in the first place.
        if self
            .control
            .list_workspaces()?
            .iter()
            .any(|w| w.label == name)
        {
            return Ok(ActionAck {
                ok: false,
                error: Some(format!("session '{name}' already exists")),
                info: None,
            });
        }
        self.control.create_workspace(Some(name.to_string()))
    }

    fn kill_session(&self, session: &str) -> Result<()> {
        let workspace_id = self.workspace_id_for(session)?;
        let ack = self.control.close_workspace(&workspace_id)?;
        if ack.ok {
            Ok(())
        } else {
            Err(anyhow!(
                "herdr workspace.close failed: {}",
                ack.error.unwrap_or_else(|| "unknown error".into())
            ))
        }
    }

    fn rename_session(&self, session: &str, new_name: String) -> Result<ActionAck> {
        let workspace_id = self.workspace_id_for(session)?;
        self.control.rename_workspace(&workspace_id, &new_name)
    }

    // ── Ephemeral control actions ───────────────────────────────────────────

    fn write_to_pane(&self, _session: &str, _pane: PaneRef, _bytes: Vec<u8>) -> Result<ActionAck> {
        // Input is wire-socket only on herdr (the relay's persistent connection);
        // there is no JSON-API equivalent. Honest failure so the client surfaces it.
        Ok(unsupported_ack("write_to_pane"))
    }

    fn focus_pane(&self, _session: &str, _pane: PaneRef) -> Result<ActionAck> {
        // Ephemeral path only (no relay attached). herdr has no focus-by-id over
        // JSON; with a live attach the relay re-attaches the wire stream to the
        // target terminal (the real focus). Benign no-op here.
        Ok(noop_ack())
    }

    fn close_pane(&self, _session: &str, pane: PaneRef) -> Result<ActionAck> {
        self.control.close_pane(pane.id)
    }

    fn new_pane(&self, session: &str, floating: bool, name: Option<String>) -> Result<ActionAck> {
        // herdr has no floating layer → `floating` is ignored. herdr's `pane.split`
        // carries no label → `name` is not applied (documented gap).
        if floating {
            log::debug!("HerdrBackend::new_pane: ignoring floating=true (no floating layer)");
        }
        if let Some(name) = &name {
            log::debug!("HerdrBackend::new_pane: ignoring name '{name}' (pane.split has no label)");
        }
        let workspace_id = self.workspace_id_for(session)?;
        self.control.split_pane(
            Some(&workspace_id),
            None, // split the focused pane
            DEFAULT_SPLIT_DIRECTION,
            true, // focus the new pane
        )
    }

    fn rename_pane(&self, _session: &str, pane: PaneRef, name: String) -> Result<ActionAck> {
        self.control.rename_pane(pane.id, Some(name))
    }

    fn resize_pane(
        &self,
        _session: &str,
        _pane: PaneRef,
        _kind: ResizeKind,
        _dir: Option<ResizeDir>,
    ) -> Result<ActionAck> {
        // Resize is wire-socket only on herdr (`ClientMessage::Resize`, driven by
        // the relay from the client's terminal geometry). No JSON-API equivalent.
        Ok(unsupported_ack("resize_pane"))
    }

    fn toggle_pane_floating(&self, _session: &str, _pane: PaneRef) -> Result<ActionAck> {
        // herdr has no floating layer; toggling is meaningless → benign no-op.
        Ok(noop_ack())
    }

    fn toggle_pane_fullscreen(&self, _session: &str, pane: PaneRef) -> Result<ActionAck> {
        // herdr's pane.zoom is the analogue of zellij pane fullscreen.
        self.control.zoom_pane(Some(pane.id), PaneZoomMode::Toggle)
    }

    fn scroll_pane(&self, _session: &str, _pane: PaneRef, _dir: ScrollDir) -> Result<ActionAck> {
        // Scrollback is wire-socket only on herdr (handled by the relay). No
        // JSON-API equivalent → honest failure.
        Ok(unsupported_ack("scroll_pane"))
    }

    fn new_tab(&self, session: &str, name: Option<String>) -> Result<ActionAck> {
        let workspace_id = self.workspace_id_for(session)?;
        self.control.create_tab(Some(&workspace_id), name)
    }

    fn close_tab(&self, _session: &str, tab_id: u64) -> Result<ActionAck> {
        self.control.close_tab(tab_id)
    }

    fn go_to_tab(&self, _session: &str, tab_id: u64) -> Result<ActionAck> {
        self.control.focus_tab(tab_id)
    }

    fn rename_tab(&self, _session: &str, tab_id: u64, name: String) -> Result<ActionAck> {
        self.control.rename_tab(tab_id, name)
    }

    // ── Read-only queries ───────────────────────────────────────────────────

    fn query_layout(&self, session: &str) -> Result<LayoutSnapshot> {
        let workspace_id = self.workspace_id_for(session)?;
        self.control.query_layout(&workspace_id)
    }

    fn query_session_size(&self, session: &str) -> Result<(u16, u16)> {
        // Derive from the workspace's focused-tab layout `area`. Pick the focused
        // pane (or the first listed) as the `pane.layout` addressing handle, then
        // read `area.height`/`area.width` → `(rows, cols)` (the order
        // `session_size_from_area` returns). Fall back to a sane default when the
        // workspace has no panes / no usable area.
        let workspace_id = self.workspace_id_for(session)?;
        let panes = self.control.list_panes(&workspace_id)?;
        let representative = panes.iter().find(|p| p.focused).or_else(|| panes.first());
        match representative {
            Some(pane) => {
                let layout = self.control.pane_layout(Some(&pane.pane_id))?;
                Ok(session_size_from_area(layout.area))
            }
            None => Ok(DEFAULT_SESSION_SIZE),
        }
    }

    fn pane_is_floating_with_visibility(
        &self,
        _session: &str,
        _pane: PaneRef,
    ) -> Result<(bool, bool, Option<PaneRef>)> {
        // herdr has no floating layer: nothing floats, nothing is floating-visible,
        // and there is no focused floating pane.
        Ok((false, false, None))
    }

    // ── Attach (the relay seam) ─────────────────────────────────────────────

    fn open_attach(
        &self,
        session: &str,
        rows: u16,
        cols: u16,
        read_only: bool,
    ) -> Result<DualHandle> {
        let workspace_id = self.workspace_id_for(session)?;
        relay::open_attach(
            Arc::clone(&self.control),
            self.paths.wire.clone(),
            workspace_id,
            session.to_string(),
            rows,
            cols,
            read_only,
        )
    }

    // ── Backend identity ────────────────────────────────────────────────────

    fn backend_version(&self) -> String {
        // The wire protocol version is the meaningful compat marker between muxrd
        // and a given herdr release.
        format!("herdr-wire-v{HERDR_PROTOCOL_VERSION}")
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> HerdrBackend {
        // Points at a nonexistent socket: every method that does I/O will error,
        // but the pure / no-op dispositions below need no live herdr.
        HerdrBackend::new(HerdrSocketPaths::from_api_socket("/nonexistent/herdr.sock"))
    }

    // ── session_size_from_area derivation ─────────────────────────────────────

    #[test]
    fn session_size_derives_rows_cols_from_area() {
        let (rows, cols) = session_size_from_area(PaneLayoutRect {
            x: 0,
            y: 0,
            width: 220,
            height: 50,
        });
        assert_eq!((rows, cols), (50, 220));
    }

    #[test]
    fn session_size_falls_back_on_zero_dimensions() {
        let (rows, cols) = session_size_from_area(PaneLayoutRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });
        assert_eq!((rows, cols), DEFAULT_SESSION_SIZE);
    }

    #[test]
    fn session_size_falls_back_per_dimension() {
        // A zero in only one axis still substitutes just that axis.
        let (rows, cols) = session_size_from_area(PaneLayoutRect {
            x: 0,
            y: 0,
            width: 100,
            height: 0,
        });
        assert_eq!((rows, cols), (DEFAULT_SESSION_SIZE.0, 100));
    }

    // ── capability-gap dispositions (no live herdr required) ──────────────────

    #[test]
    fn write_resize_scroll_are_honest_failures() {
        let b = backend();
        let p = PaneRef::terminal(1);
        let w = b.write_to_pane("s", p, b"x".to_vec()).unwrap();
        assert!(!w.ok);
        assert!(w.error.as_deref().unwrap().contains("write_to_pane"));

        let r = b
            .resize_pane("s", p, ResizeKind::Increase, Some(ResizeDir::Left))
            .unwrap();
        assert!(!r.ok);
        assert!(r.error.as_deref().unwrap().contains("resize_pane"));

        let s = b.scroll_pane("s", p, ScrollDir::HalfPageUp).unwrap();
        assert!(!s.ok);
        assert!(s.error.as_deref().unwrap().contains("scroll_pane"));
    }

    #[test]
    fn focus_and_toggle_floating_are_benign_noops() {
        let b = backend();
        let p = PaneRef::terminal(1);
        // focus_pane: relay re-attach is the real focus; ephemeral path no-ops ok.
        assert!(b.focus_pane("s", p).unwrap().ok);
        // toggle_pane_floating: herdr has no floating layer → meaningless, ok no-op.
        assert!(b.toggle_pane_floating("s", p).unwrap().ok);
    }

    // ── pure trait dispositions ───────────────────────────────────────────────

    #[test]
    fn pane_floating_visibility_is_always_false() {
        let b = backend();
        let (is_floating, visible, focused) = b
            .pane_is_floating_with_visibility("s", PaneRef::terminal(1))
            .unwrap();
        assert!(!is_floating);
        assert!(!visible);
        assert!(focused.is_none());
    }

    #[test]
    fn backend_version_encodes_wire_protocol() {
        let b = backend();
        assert_eq!(
            b.backend_version(),
            format!("herdr-wire-v{HERDR_PROTOCOL_VERSION}")
        );
    }

    // ── validate_session_name keeps the strict security guard ─────────────────

    #[test]
    fn validate_session_name_is_strict() {
        let b = backend();
        assert!(b.validate_session_name("dev").is_ok());
        assert!(b.validate_session_name("my-work_1").is_ok());
        assert!(b.validate_session_name("../foo").is_err());
        assert!(b.validate_session_name("foo/bar").is_err());
        assert!(b.validate_session_name("").is_err());
        assert!(b.validate_session_name("with space").is_err());
    }

    // ── shared registries are the same Arcs the control client holds ──────────

    #[test]
    fn registries_are_shared_with_control_client() {
        let b = backend();
        assert!(Arc::ptr_eq(b.pane_registry(), b.control.pane_registry()));
        assert!(Arc::ptr_eq(b.tab_registry(), b.control.tab_registry()));
    }
}
