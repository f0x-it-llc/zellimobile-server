//! spike — Phase-A IPC attach + render streaming example.
//!
//! Preserves the A1/A2 proof: attaches to a running zellij session over IPC,
//! streams live renders to stdout, and optionally sends keystrokes.
//!
//! This is the same behaviour as the original `src/main.rs` Phase-A spike,
//! now using the clean `ipc` module API.
//!
//! Usage:
//!     cargo run --example spike -- [SESSION_NAME] [--type <TEXT>]
//!
//! Examples:
//!     cargo run --example spike                              # attach + stream renders
//!     cargo run --example spike -- a2demo                   # named session
//!     cargo run --example spike -- a2demo --type "echo hi\n"  # type into session

use std::io::Write as _;
use std::time::Duration;

use anyhow::Result;
use zellimserver::ipc::{self, AttachHandle};

struct Args {
    session_name: Option<String>,
    type_text: Option<String>,
}

fn parse_args() -> Args {
    let mut raw: Vec<String> = std::env::args().skip(1).collect();
    let type_text = if let Some(pos) = raw.iter().position(|a| a == "--type") {
        if pos + 1 < raw.len() {
            let val = raw.remove(pos + 1);
            raw.remove(pos);
            Some(val.replace("\\n", "\n"))
        } else {
            raw.remove(pos);
            None
        }
    } else {
        None
    };
    let session_name = raw.into_iter().next();
    Args {
        session_name,
        type_text,
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = parse_args();
    let session_name = match args.session_name {
        Some(n) => n,
        None => ipc::pick_first_session()?,
    };
    log::info!("attaching to session: {session_name}");

    let mut handle = AttachHandle::open(&session_name, 24, 80)?;
    log::info!("AttachClient sent; waiting for first render…");

    let mut stdout = std::io::stdout();
    let mut msg_count: u64 = 0;
    let mut render_count: u64 = 0;
    let mut input_sent = false;

    loop {
        let Some(msg) = handle.recv() else {
            log::warn!(
                "recv returned None after {msg_count} messages ({render_count} renders); exiting"
            );
            break;
        };
        msg_count += 1;

        use zellij_utils::ipc::ServerToClientMsg;
        match msg {
            ServerToClientMsg::Render { content } => {
                render_count += 1;
                if render_count == 1 {
                    log::info!(
                        "first Render ({} bytes) — streaming ANSI to stdout",
                        content.len()
                    );
                }
                stdout.write_all(content.as_bytes())?;
                stdout.flush()?;

                if render_count == 1 && !input_sent {
                    if let Some(ref text) = args.type_text {
                        std::thread::sleep(Duration::from_millis(500));
                        ipc::dismiss_overlay(&mut handle)?;
                        std::thread::sleep(Duration::from_millis(500));
                        log::info!("sending text via WriteChars: {:?}", text);
                        handle.send_chars(text)?;
                        log::info!("WriteChars sent; continuing to stream renders…");
                        input_sent = true;
                    }
                }
            }
            ServerToClientMsg::Exit { exit_reason } => {
                log::info!("Exit received (msg #{msg_count}): {exit_reason:?}");
                break;
            }
            other => {
                log::debug!("msg #{msg_count}: {other:?}");
            }
        }

        if input_sent && render_count >= 5 {
            log::info!("input sent + {render_count} renders received — exiting cleanly");
            break;
        }
    }

    log::info!("done: {msg_count} messages, {render_count} renders");
    Ok(())
}
