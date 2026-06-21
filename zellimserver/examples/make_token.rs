//! make_token — create (or show) a zellij authentication token.
//!
//! This is a B3 setup helper: since the CLI token-management commands are
//! Phase E, this example calls `web_authentication_tokens::create_token`
//! directly and prints the plaintext token to stdout.
//!
//! The printed token is the **auth token** (long-lived, stored hashed in
//! tokens.db).  Pass it to `Login(auth_token, remember_me)` to get a
//! short-lived session token for gRPC bearer auth.
//!
//! Usage:
//!     cargo run --example make_token -- [--name <token_name>] [--read-only]
//!
//! Flags:
//!     --name <name>   Name for the token (default: "b3test").
//!     --read-only     Mark the token as read-only (default: false).
//!
//! The token is written to the shared zellij tokens DB
//! (`ZELLIJ_PROJ_DIR.data_dir()/tokens*.db`).  Clean up afterwards with
//! `revoke_token("b3test")` or re-run `examples/make_token.rs` with the
//! same name to see the duplicate-name error (safe — won't overwrite).

use anyhow::{Context, Result};
use zellij_utils::web_authentication_tokens::{create_token, revoke_token};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let get = |key: &str| -> Option<String> {
        args.iter()
            .position(|a| a == key)
            .and_then(|p| args.get(p + 1).cloned())
    };

    let name = get("--name").unwrap_or_else(|| "b3test".to_owned());
    let read_only = args.iter().any(|a| a == "--read-only");

    // --revoke revokes the named token instead of creating one.
    if args.iter().any(|a| a == "--revoke") {
        let removed = revoke_token(&name).with_context(|| format!("revoke_token('{name}')"))?;
        if removed {
            println!("revoked token '{name}' (and its session tokens)");
        } else {
            println!("token '{name}' not found (already gone or never created)");
        }
        return Ok(());
    }

    let (token, token_name) = create_token(Some(name.clone()), read_only)
        .with_context(|| format!("create_token('{name}', read_only={read_only})"))?;

    println!("created auth token:");
    println!("  name      : {token_name}");
    println!("  read_only : {read_only}");
    println!("  token     : {token}");
    println!();
    println!("Pass `token` to Login(auth_token=...) to get a session token.");
    println!("To revoke: cargo run --example make_token -- --name {token_name} --revoke");

    Ok(())
}
