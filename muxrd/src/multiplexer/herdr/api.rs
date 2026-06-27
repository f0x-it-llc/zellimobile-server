//! Independently-authored types matching herdr's public v0.7.1 wire/JSON protocol for interop.
//! Not derived from herdr's AGPL source; herdr runs as a separate, unmodified, user-installed
//! binary driven over its public sockets.
//!
//! # JSON-API — control socket
//!
//! herdr exposes a line-delimited JSON control socket.  Each line is either a
//! **request** or a **response**:
//!
//! ```json
//! // request  → { "id": "<uuid>", "method": "pane.layout", "params": { ... } }
//! // response → { "id": "<uuid>", "result": { "type": "pane_layout", ... } }
//! //          | { "id": "<uuid>", "error":  { "code": "...", "message": "..." } }
//! ```
//!
//! The `result` object uses an internal discriminant field `"type"` with
//! `snake_case` values (e.g. `"pane_layout"`, `"workspace_list"`) — verified
//! against herdr's `ResponseResult` which has `#[serde(tag = "type", rename_all = "snake_case")]`.
//!
//! ## Usage pattern (P2.02 control client)
//!
//! ```ignore
//! let req = ApiRequest::new("pane.layout", serde_json::to_value(PaneLayoutParams { pane_id: None })?);
//! // write req serialized as JSON + "\n"
//! // read response line
//! let resp: ApiResponse = serde_json::from_str(&line)?;
//! let result: ApiResult = resp.into_ok()?.into_result()?;
//! if let ApiResult::PaneLayout { layout } = result { /* use layout */ }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ─── Request envelope ─────────────────────────────────────────────────────────

/// A JSON-API request sent to herdr's control socket.
///
/// Serializes as `{"id":"…","method":"workspace.list","params":{}}`.
#[derive(Debug, Clone, Serialize)]
pub struct ApiRequest {
    /// Caller-assigned request identifier (returned in the response).
    pub id: String,
    /// Method name — lowercase dotted string (e.g. `"pane.layout"`).
    pub method: String,
    /// Method-specific parameters, serialized from a typed param struct.
    pub params: serde_json::Value,
}

impl ApiRequest {
    /// Construct a request with the given method and params value.
    pub fn new(
        id: impl Into<String>,
        method: impl Into<String>,
        params: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

// ─── Response envelope ───────────────────────────────────────────────────────

/// A raw JSON-API response from herdr, before result-type dispatch.
///
/// Parses both success (`{"id","result":{…}}`) and error (`{"id","error":{…}}`)
/// shapes.  Use [`ApiRawResponse::into_result`] to decode the typed result.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiRawResponse {
    /// Echoes the request `id`.
    pub id: String,
    #[serde(flatten)]
    pub body: ApiResponseBody,
}

impl ApiRawResponse {
    /// Convert to a typed [`ApiResult`], returning `Err` on API error or
    /// on unknown / mismatched result type.
    pub fn into_result(self) -> Result<ApiResult, ApiResponseError> {
        match self.body {
            ApiResponseBody::Ok { result } => {
                serde_json::from_value(result).map_err(ApiResponseError::Deserialize)
            }
            ApiResponseBody::Err { error } => Err(ApiResponseError::Api(error)),
        }
    }
}

/// Untagged body — distinguished by presence of `result` vs `error` key.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ApiResponseBody {
    Ok { result: serde_json::Value },
    Err { error: ApiErrorBody },
}

/// Error information returned by herdr for a failed request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

/// Error variants when processing an [`ApiRawResponse`].
#[derive(Debug)]
pub enum ApiResponseError {
    /// herdr returned an API-level error.
    Api(ApiErrorBody),
    /// The result JSON did not match the expected [`ApiResult`] variant.
    Deserialize(serde_json::Error),
}

impl std::fmt::Display for ApiResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(e) => write!(f, "herdr API error {}: {}", e.code, e.message),
            Self::Deserialize(e) => write!(f, "result deserialize error: {e}"),
        }
    }
}

impl std::error::Error for ApiResponseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Deserialize(e) => Some(e),
            _ => None,
        }
    }
}

// ─── Typed result enum ────────────────────────────────────────────────────────

/// Typed result variants for the methods muxrd calls.
///
/// Mirrors the subset of herdr's `ResponseResult` that we consume.  The
/// `"type"` field discriminant and `snake_case` renaming match herdr exactly:
///
/// ```json
/// { "type": "pane_layout", "layout": { … } }
/// { "type": "workspace_list", "workspaces": [ … ] }
/// { "type": "ok" }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiResult {
    WorkspaceList {
        workspaces: Vec<WorkspaceInfo>,
    },
    WorkspaceCreated {
        workspace: WorkspaceInfo,
        tab: TabInfo,
        root_pane: PaneInfo,
    },
    WorkspaceInfo {
        workspace: WorkspaceInfo,
    },
    TabCreated {
        tab: TabInfo,
        root_pane: PaneInfo,
    },
    TabInfo {
        tab: TabInfo,
    },
    TabList {
        tabs: Vec<TabInfo>,
    },
    PaneInfo {
        pane: PaneInfo,
    },
    PaneList {
        panes: Vec<PaneInfo>,
    },
    PaneLayout {
        layout: PaneLayoutSnapshot,
    },
    PaneFocusDirection {
        focus: PaneFocusDirectionResult,
    },
    PaneZoom {
        zoom: PaneZoomResult,
    },
    LayoutExport {
        layout: LayoutDescription,
    },
    /// Generic success with no payload.
    Ok {},
}

// ─── Request param structs ────────────────────────────────────────────────────
//
// One struct per method we call. Serialize with `serde_json::to_value` to
// produce the `params` field of an `ApiRequest`.

/// `workspace.list` — no params.
#[derive(Debug, Default, Serialize)]
pub struct WorkspaceListParams {}

/// `workspace.create`
#[derive(Debug, Default, Serialize)]
pub struct WorkspaceCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default)]
    pub focus: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// `workspace.rename`
#[derive(Debug, Serialize)]
pub struct WorkspaceRenameParams {
    pub workspace_id: String,
    pub label: String,
}

/// `workspace.close`
#[derive(Debug, Serialize)]
pub struct WorkspaceCloseParams {
    pub workspace_id: String,
}

/// `tab.create`
#[derive(Debug, Default, Serialize)]
pub struct TabCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default)]
    pub focus: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// `tab.focus`
#[derive(Debug, Serialize)]
pub struct TabFocusParams {
    pub tab_id: String,
}

/// `tab.close`
#[derive(Debug, Serialize)]
pub struct TabCloseParams {
    pub tab_id: String,
}

/// `tab.rename`
#[derive(Debug, Serialize)]
pub struct TabRenameParams {
    pub tab_id: String,
    pub label: String,
}

/// `pane.split`
#[derive(Debug, Serialize)]
pub struct PaneSplitParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_pane_id: Option<String>,
    pub direction: SplitDirection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default)]
    pub focus: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// `pane.close`
#[derive(Debug, Serialize)]
pub struct PaneCloseParams {
    pub pane_id: String,
}

/// `pane.rename`
#[derive(Debug, Serialize)]
pub struct PaneRenameParams {
    pub pane_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// `pane.focus_direction`
#[derive(Debug, Serialize)]
pub struct PaneFocusDirectionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_id: Option<String>,
    pub direction: PaneDirection,
}

/// `pane.zoom`
#[derive(Debug, Default, Serialize)]
pub struct PaneZoomParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_id: Option<String>,
    #[serde(default)]
    pub mode: PaneZoomMode,
}

/// `pane.layout`
#[derive(Debug, Default, Serialize)]
pub struct PaneLayoutParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_id: Option<String>,
}

/// `layout.export`
#[derive(Debug, Default, Serialize)]
pub struct LayoutExportParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_id: Option<String>,
}

// ─── Shared enums ────────────────────────────────────────────────────────────

/// Pane split direction.  Matches herdr's `SplitDirection` (`snake_case`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDirection {
    Right,
    Down,
}

/// Directional focus / resize target.  Matches herdr's `PaneDirection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Zoom mode for `pane.zoom`.  Matches herdr's `PaneZoomMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaneZoomMode {
    #[default]
    Toggle,
    On,
    Off,
}

/// High-level agent activity status exposed on workspaces, tabs, and panes.
/// Matches herdr's `AgentStatus` (`snake_case`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Working,
    Blocked,
    Done,
    Unknown,
}

/// Kind of agent session reference.  Matches herdr's `AgentSessionRefKind` (`snake_case`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSessionRefKind {
    Id,
    Path,
}

// ─── Data structs ────────────────────────────────────────────────────────────

/// Git-worktree information attached to a workspace, when the workspace was
/// created from a worktree source.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WorkspaceWorktreeInfo {
    pub repo_key: String,
    pub repo_name: String,
    pub repo_root: String,
    pub checkout_path: String,
    pub is_linked_worktree: bool,
}

/// Top-level workspace (herdr's equivalent of a zellij session).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WorkspaceInfo {
    pub workspace_id: String,
    pub number: usize,
    pub label: String,
    pub focused: bool,
    pub pane_count: usize,
    pub tab_count: usize,
    pub active_tab_id: String,
    pub agent_status: AgentStatus,
    #[serde(default)]
    pub worktree: Option<WorkspaceWorktreeInfo>,
}

/// Tab within a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TabInfo {
    pub tab_id: String,
    pub workspace_id: String,
    pub number: usize,
    pub label: String,
    pub focused: bool,
    pub pane_count: usize,
    pub agent_status: AgentStatus,
}

/// Agent-session resume reference attached to a pane.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgentSessionInfo {
    pub source: String,
    pub agent: String,
    pub kind: AgentSessionRefKind,
    pub value: String,
}

/// Individual pane within a tab.
///
/// `terminal_id` is the key used to attach the wire relay socket
/// (`ClientMessage::AttachTerminal { terminal_id }`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PaneInfo {
    pub pane_id: String,
    /// Wire-relay attach key — used in `ClientMessage::AttachTerminal`.
    pub terminal_id: String,
    pub workspace_id: String,
    pub tab_id: String,
    pub focused: bool,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub foreground_cwd: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub display_agent: Option<String>,
    pub agent_status: AgentStatus,
    #[serde(default)]
    pub custom_status: Option<String>,
    #[serde(default)]
    pub state_labels: HashMap<String, String>,
    #[serde(default)]
    pub agent_session: Option<AgentSessionInfo>,
    pub revision: u64,
}

/// Absolute-cell rectangle within a tab's layout area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PaneLayoutRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Position and focus state of a single pane in the layout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PaneLayoutPane {
    pub pane_id: String,
    pub focused: bool,
    pub rect: PaneLayoutRect,
}

/// A split boundary within the layout tree.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PaneLayoutSplit {
    pub id: String,
    pub direction: SplitDirection,
    pub ratio: f32,
    pub rect: PaneLayoutRect,
}

/// Flat snapshot of all pane positions in a tab.
///
/// `area` is the total terminal area in absolute cells.
/// `panes` and `splits` together describe the current layout tree.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PaneLayoutSnapshot {
    pub workspace_id: String,
    pub tab_id: String,
    pub zoomed: bool,
    /// Total terminal area in absolute cells.
    pub area: PaneLayoutRect,
    pub focused_pane_id: String,
    pub panes: Vec<PaneLayoutPane>,
    pub splits: Vec<PaneLayoutSplit>,
}

/// Reason a directional focus change did not take effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneFocusDirectionReason {
    NoNeighbor,
}

/// Result of a `pane.focus_direction` call.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PaneFocusDirectionResult {
    pub changed: bool,
    #[serde(default)]
    pub reason: Option<PaneFocusDirectionReason>,
    pub source_pane_id: String,
    #[serde(default)]
    pub focused_pane_id: Option<String>,
    pub layout: PaneLayoutSnapshot,
}

/// Reason a zoom change did not take effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneZoomReason {
    SinglePane,
    AlreadyZoomed,
    AlreadyUnzoomed,
}

/// Result of a `pane.zoom` call.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PaneZoomResult {
    pub changed: bool,
    pub zoom_changed: bool,
    pub focus_changed: bool,
    #[serde(default)]
    pub reason: Option<PaneZoomReason>,
    pub pane_id: String,
    pub focused_pane_id: String,
    pub zoomed: bool,
    pub layout: PaneLayoutSnapshot,
}

/// A pane node within an exported layout tree.
/// Fields may be absent for placeholder panes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LayoutPane {
    #[serde(default)]
    pub pane_id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub command: Option<Vec<String>>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// A node in herdr's recursive layout tree (exported by `layout.export`).
///
/// Uses `#[serde(tag = "type", rename_all = "snake_case")]` to match herdr's
/// `LayoutNode` serialization: `{ "type": "pane", … }` or
/// `{ "type": "split", … }`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayoutNode {
    Pane {
        #[serde(flatten)]
        pane: LayoutPane,
    },
    Split {
        direction: SplitDirection,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

/// Full layout export for a tab, returned by `layout.export`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LayoutDescription {
    pub workspace_id: String,
    pub tab_id: String,
    pub zoomed: bool,
    pub focused_pane_id: String,
    pub root: LayoutNode,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that a representative `PaneInfo` JSON blob deserializes correctly,
    /// locking the field names and serde attributes against herdr's schema.
    #[test]
    fn pane_info_deserialize() {
        let json = r#"{
            "pane_id": "pane-abc",
            "terminal_id": "term-xyz",
            "workspace_id": "ws-1",
            "tab_id": "tab-1",
            "focused": true,
            "cwd": "/home/user/project",
            "foreground_cwd": null,
            "label": "editor",
            "agent": null,
            "title": null,
            "display_agent": null,
            "agent_status": "idle",
            "custom_status": null,
            "state_labels": {},
            "agent_session": null,
            "revision": 7
        }"#;

        let info: PaneInfo = serde_json::from_str(json).expect("PaneInfo must deserialize");
        assert_eq!(info.pane_id, "pane-abc");
        assert_eq!(info.terminal_id, "term-xyz");
        assert_eq!(info.workspace_id, "ws-1");
        assert_eq!(info.tab_id, "tab-1");
        assert!(info.focused);
        assert_eq!(info.cwd.as_deref(), Some("/home/user/project"));
        assert_eq!(info.label.as_deref(), Some("editor"));
        assert_eq!(info.agent_status, AgentStatus::Idle);
        assert_eq!(info.revision, 7);
        assert!(info.agent_session.is_none());
    }

    /// Verify that a `PaneInfo` with an `agent_session` field deserializes,
    /// exercising the `AgentSessionInfo` + `AgentSessionRefKind` path.
    #[test]
    fn pane_info_with_agent_session_deserialize() {
        let json = r#"{
            "pane_id": "pane-1",
            "terminal_id": "term-1",
            "workspace_id": "ws-1",
            "tab_id": "tab-1",
            "focused": false,
            "agent_status": "working",
            "state_labels": {},
            "agent_session": {
                "source": "herdr:claude",
                "agent": "claude",
                "kind": "id",
                "value": "session-abc123"
            },
            "revision": 3
        }"#;

        let info: PaneInfo =
            serde_json::from_str(json).expect("PaneInfo with session must deserialize");
        assert_eq!(info.agent_status, AgentStatus::Working);
        let session = info.agent_session.expect("agent_session must be present");
        assert_eq!(session.source, "herdr:claude");
        assert_eq!(session.kind, AgentSessionRefKind::Id);
        assert_eq!(session.value, "session-abc123");
    }

    /// Verify that a representative `PaneLayoutSnapshot` deserializes correctly.
    #[test]
    fn pane_layout_snapshot_deserialize() {
        let json = r#"{
            "workspace_id": "ws-1",
            "tab_id": "tab-1",
            "zoomed": false,
            "area": { "x": 0, "y": 0, "width": 220, "height": 50 },
            "focused_pane_id": "pane-left",
            "panes": [
                { "pane_id": "pane-left",  "focused": true,  "rect": { "x": 0,   "y": 0, "width": 110, "height": 50 } },
                { "pane_id": "pane-right", "focused": false, "rect": { "x": 110, "y": 0, "width": 110, "height": 50 } }
            ],
            "splits": [
                {
                    "id": "split-0",
                    "direction": "right",
                    "ratio": 0.5,
                    "rect": { "x": 0, "y": 0, "width": 220, "height": 50 }
                }
            ]
        }"#;

        let snap: PaneLayoutSnapshot =
            serde_json::from_str(json).expect("PaneLayoutSnapshot must deserialize");
        assert_eq!(snap.workspace_id, "ws-1");
        assert_eq!(snap.area.width, 220);
        assert_eq!(snap.area.height, 50);
        assert!(!snap.zoomed);
        assert_eq!(snap.focused_pane_id, "pane-left");
        assert_eq!(snap.panes.len(), 2);
        assert_eq!(snap.panes[0].pane_id, "pane-left");
        assert!(snap.panes[0].focused);
        assert_eq!(snap.panes[0].rect.x, 0);
        assert_eq!(snap.panes[1].rect.x, 110);
        assert_eq!(snap.splits.len(), 1);
        assert_eq!(snap.splits[0].direction, SplitDirection::Right);
        assert!((snap.splits[0].ratio - 0.5).abs() < 1e-6);
    }

    /// Verify that the `ApiResult` enum deserializes with the correct `"type"` tag
    /// and `snake_case` conversion.
    #[test]
    fn api_result_pane_layout_deserialize() {
        let json = r#"{
            "type": "pane_layout",
            "layout": {
                "workspace_id": "ws-1",
                "tab_id": "tab-1",
                "zoomed": false,
                "area": { "x": 0, "y": 0, "width": 80, "height": 24 },
                "focused_pane_id": "pane-a",
                "panes": [
                    { "pane_id": "pane-a", "focused": true, "rect": { "x": 0, "y": 0, "width": 80, "height": 24 } }
                ],
                "splits": []
            }
        }"#;

        let result: ApiResult =
            serde_json::from_str(json).expect("ApiResult::PaneLayout must deserialize");
        if let ApiResult::PaneLayout { layout } = result {
            assert_eq!(layout.panes.len(), 1);
            assert_eq!(layout.panes[0].pane_id, "pane-a");
        } else {
            panic!("expected ApiResult::PaneLayout");
        }
    }

    /// Verify the `"ok"` result type tag.
    #[test]
    fn api_result_ok_deserialize() {
        let json = r#"{"type":"ok"}"#;
        let result: ApiResult = serde_json::from_str(json).expect("ApiResult::Ok must deserialize");
        assert!(matches!(result, ApiResult::Ok {}));
    }

    /// Verify `ApiRawResponse` parses the success shape and routes into `ApiResult`.
    #[test]
    fn api_raw_response_success_round_trip() {
        let json = r#"{
            "id": "req-42",
            "result": {
                "type": "workspace_list",
                "workspaces": []
            }
        }"#;

        let raw: ApiRawResponse = serde_json::from_str(json).expect("ApiRawResponse must parse");
        assert_eq!(raw.id, "req-42");
        let result = raw.into_result().expect("into_result must succeed");
        assert!(matches!(result, ApiResult::WorkspaceList { workspaces } if workspaces.is_empty()));
    }

    /// Verify `ApiRawResponse` parses the error shape correctly.
    #[test]
    fn api_raw_response_error_round_trip() {
        let json = r#"{
            "id": "req-7",
            "error": { "code": "not_found", "message": "workspace not found" }
        }"#;

        let raw: ApiRawResponse = serde_json::from_str(json).expect("ApiRawResponse must parse");
        let err = raw.into_result().unwrap_err();
        if let ApiResponseError::Api(body) = err {
            assert_eq!(body.code, "not_found");
        } else {
            panic!("expected ApiResponseError::Api");
        }
    }

    /// Verify request serialization produces the correct JSON shape.
    #[test]
    fn api_request_serialize() {
        let req = ApiRequest::new(
            "req-1",
            "pane.layout",
            serde_json::to_value(PaneLayoutParams { pane_id: None }).unwrap(),
        );
        let json: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json["id"], "req-1");
        assert_eq!(json["method"], "pane.layout");
        assert!(json["params"].is_object());
    }
}
