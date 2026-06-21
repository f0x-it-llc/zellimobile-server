//! Cross-cutting free helpers used across grpc submodules.

use tonic::{Request, Response, Status};

use crate::actions::{self, ActionAck};
use crate::proto::{ActionAck as ProtoAck, PaneTarget, TabTarget};

// ─── Control routing ──────────────────────────────────────────────────────────

/// Try to route a control command through a live relay's AttachClient (W2.0a/b
/// spike). Returns `Some(ok-ack)` if a relay is registered for `session` and the
/// command was queued; `None` → caller falls back to the ephemeral CLI path.
pub(super) fn try_route_control(
    control: &crate::relay::ControlRegistry,
    session: &str,
    cmd: crate::relay::RelayControl,
) -> Option<Response<ProtoAck>> {
    // Clone the sender out so the DashMap Ref (shard read-lock) is released
    // before we send — never hold a shard guard across the channel send.
    let tx = control.get(session).map(|r| r.value().clone())?;
    if tx.send(cmd).is_ok() {
        Some(Response::new(ProtoAck {
            ok: true,
            error: String::new(),
            info: "routed via relay client (spike)".to_owned(),
        }))
    } else {
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

/// Fetch tabs JSON and panes JSON via the original ephemeral-AttachClient path.
///
/// Each query opens its own short-lived IPC connection. Used by `get_layout`
/// when no relay is attached for the session (e.g. Sessions screen querying a
/// non-active session), or as a fallback when the relay query fails/times out.
///
/// Returns `(tabs_json, panes_json, via_relay=false)`.
pub(super) async fn ephemeral_query(session: &str) -> Result<(String, String, bool), Status> {
    let session_tabs = session.to_owned();
    let session_panes = session.to_owned();

    let tabs_json =
        tokio::task::spawn_blocking(move || crate::query::query_list_tabs_json(&session_tabs))
            .await
            .map_err(|e| Status::internal(format!("GetLayout tabs task panicked: {e}")))?
            .map_err(|e| {
                log::warn!("GetLayout tabs query failed: {e:#}");
                Status::internal(format!("ListTabs query failed: {e:#}"))
            })?;

    let panes_json =
        tokio::task::spawn_blocking(move || crate::query::query_list_panes_json(&session_panes))
            .await
            .map_err(|e| Status::internal(format!("GetLayout panes task panicked: {e}")))?
            .map_err(|e| {
                log::warn!("GetLayout panes query failed: {e:#}");
                Status::internal(format!("ListPanes query failed: {e:#}"))
            })?;

    Ok((tabs_json, panes_json, false))
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

/// Validate a [`PaneTarget`] (non-empty session) and map it to `(session, PaneId)`.
pub(super) fn resolve_pane_target(
    target: &PaneTarget,
) -> Result<(String, zellij_utils::data::PaneId), Status> {
    validate_session(&target.session)?;
    let pane = actions::pane_id_from_target(target.pane_id, target.is_plugin);
    Ok((target.session.clone(), pane))
}

/// Validate a [`TabTarget`] (valid session) and return `(session, tab_id)`.
pub(super) fn resolve_tab_target(target: &TabTarget) -> Result<(String, u64), Status> {
    validate_session(&target.session)?;
    Ok((target.session.clone(), target.tab_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::SessionReadOnly;

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
}
