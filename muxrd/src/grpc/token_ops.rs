//! Auth and token-management RPC implementations.

use tonic::{Request, Response, Status};
use zellij_utils::web_authentication_tokens::create_session_token;

use crate::proto::{
    ActionAck as ProtoAck, CreateTokenReq, Empty, LoginRequest, LoginResponse, RevokeTokenReq,
    TokenInfo, TokenList, VersionInfo,
};

use super::SERVER_VERSION;
use super::MuxrService;
use super::helpers::reject_if_read_only;

impl MuxrService {
    // ── GetVersion ──────────────────────────────────────────────────────────

    pub(super) async fn get_version_impl(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<VersionInfo>, Status> {
        let info = VersionInfo {
            server_version: SERVER_VERSION.to_owned(),
            zellij_version: zellij_utils::consts::VERSION.to_owned(),
        };
        log::debug!(
            "GetVersion → server={} zellij={}",
            info.server_version,
            info.zellij_version
        );
        Ok(Response::new(info))
    }

    // ── Login ───────────────────────────────────────────────────────────────

    pub(super) async fn login_impl(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        log::info!("Login attempt (remember_me={})", req.remember_me);

        let session_token =
            create_session_token(&req.auth_token, req.remember_me).map_err(|e| {
                log::info!("Login rejected: {e}");
                Status::unauthenticated(format!("invalid auth token: {e}"))
            })?;

        // Surface the read-only scope of the issued session so the client can
        // disable mutating controls up-front. Enforcement stays server-side;
        // this is advisory. Reuse the same DB-backed check the auth layer uses.
        let is_read_only =
            zellij_utils::web_authentication_tokens::is_session_token_read_only(&session_token)
                .unwrap_or(false);

        log::info!("Login succeeded — issued session token (read_only={is_read_only})");
        Ok(Response::new(LoginResponse {
            session_token,
            is_read_only,
        }))
    }

    // ── Token management (Phase F) ────────────────────────────────────────────
    //
    // Thin wrappers over the same `web_authentication_tokens` ops the CLI uses,
    // against zellij's shared tokens.db.  All three are ADMIN-gated: a read-only
    // session token is rejected (`reject_if_read_only`) so observers cannot mint
    // or revoke credentials.  The token DB is shared with real zellij — these
    // operate on the same tokens the `zellij web`/`muxrd` CLI manage.

    /// Create a new auth token. MUTATING (read-only rejected).  The secret is
    /// returned ONCE in `TokenInfo.token`.
    pub(super) async fn create_token_impl(
        &self,
        request: Request<CreateTokenReq>,
    ) -> Result<Response<TokenInfo>, Status> {
        reject_if_read_only(&request, "CreateToken")?;
        let req = request.into_inner();
        // An empty name lets zellij auto-generate one (CLI parity: Option<String>).
        let name = {
            let n = req.name.trim();
            if n.is_empty() {
                None
            } else {
                Some(n.to_owned())
            }
        };
        let read_only = req.read_only;
        log::info!("CreateToken: name={name:?} read_only={read_only}");

        let (token, actual_name) = tokio::task::spawn_blocking(move || {
            zellij_utils::web_authentication_tokens::create_token(name, read_only)
        })
        .await
        .map_err(|e| Status::internal(format!("CreateToken task panicked: {e}")))?
        .map_err(|e| {
            log::warn!("CreateToken: failed: {e:#}");
            Status::internal(format!("create token failed: {e:#}"))
        })?;

        Ok(Response::new(TokenInfo {
            name: actual_name,
            token, // secret — returned only here, never on ListTokens
            read_only,
            created_at: String::new(), // not surfaced by create_token; fetch via ListTokens
        }))
    }

    /// List existing auth tokens (metadata only — never the secret).
    /// Read-only rejected (token names are sensitive).
    pub(super) async fn list_tokens_impl(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<TokenList>, Status> {
        reject_if_read_only(&request, "ListTokens")?;

        let tokens =
            tokio::task::spawn_blocking(zellij_utils::web_authentication_tokens::list_tokens)
                .await
                .map_err(|e| Status::internal(format!("ListTokens task panicked: {e}")))?
                .map_err(|e| {
                    log::warn!("ListTokens: failed: {e:#}");
                    Status::internal(format!("list tokens failed: {e:#}"))
                })?;

        let proto_tokens: Vec<TokenInfo> = tokens
            .into_iter()
            .map(|t| TokenInfo {
                name: t.name,
                token: String::new(), // never expose existing secrets
                read_only: t.read_only,
                created_at: t.created_at,
            })
            .collect();

        log::info!("ListTokens: returning {} token(s)", proto_tokens.len());
        Ok(Response::new(TokenList {
            tokens: proto_tokens,
        }))
    }

    /// Revoke an auth token by name. MUTATING (read-only rejected).
    pub(super) async fn revoke_token_impl(
        &self,
        request: Request<RevokeTokenReq>,
    ) -> Result<Response<ProtoAck>, Status> {
        reject_if_read_only(&request, "RevokeToken")?;
        let req = request.into_inner();
        if req.name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "RevokeToken: name must not be empty",
            ));
        }
        let name = req.name.clone();
        log::info!("RevokeToken: name='{name}'");

        let removed = tokio::task::spawn_blocking(move || {
            zellij_utils::web_authentication_tokens::revoke_token(&name)
        })
        .await
        .map_err(|e| Status::internal(format!("RevokeToken task panicked: {e}")))?
        .map_err(|e| {
            log::warn!("RevokeToken: failed: {e:#}");
            Status::internal(format!("revoke token failed: {e:#}"))
        })?;

        Ok(Response::new(ProtoAck {
            ok: removed,
            error: if removed {
                String::new()
            } else {
                format!(
                    "token '{}' not found (already revoked or never existed)",
                    req.name
                )
            },
            info: String::new(),
        }))
    }
}
