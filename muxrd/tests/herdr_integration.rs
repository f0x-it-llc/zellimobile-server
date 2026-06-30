//! herdr integration smoke-test (P2.05).
//!
//! All tests here are `#[ignore]`-gated so `cargo test -p muxrd` stays green
//! without a live herdr instance.  Run them against a real herdr:
//!
//! ```text
//! # 1.  Start herdr (user-installed binary, unmodified).
//! #     herdr defaults to $HOME/.config/herdr/herdr.sock; or set HERDR_SOCKET_PATH:
//! herdr &
//!
//! # 2.  Point muxrd at it and run the ignored tests:
//! HERDR_SOCKET_PATH=/path/to/herdr.sock \
//!   cargo test -p muxrd --test herdr_integration -- --ignored
//! ```
//!
//! ## What each test exercises
//!
//! | test | exercises |
//! |------|-----------|
//! | `smoke_list_sessions`         | `HerdrBackend::list_sessions()` — JSON-API workspace list |
//! | `smoke_create_query_kill`     | `create_session` / `query_layout` / `kill_session` round-trip |
//! | `smoke_open_attach_render_input` | `open_attach` → read `Render` frames → send input → teardown |
//!
//! ## AGPL note
//!
//! These tests drive herdr solely through its public Unix-domain sockets
//! (the JSON-API control socket and the binary wire relay socket).  herdr runs
//! as a separate, unmodified, user-installed binary; no herdr source is linked.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use muxrd::multiplexer::{HerdrBackend, MuxBackend, MuxServerMsg};

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Construct a `HerdrBackend` from the process environment.
///
/// Panics with an actionable message when `HERDR_SOCKET_PATH` is not set
/// *and* the XDG default does not exist — so test failures are diagnosed
/// immediately rather than producing a confusing socket-connect error.
fn backend() -> HerdrBackend {
    HerdrBackend::from_env()
}

/// A unique session name for tests that create a workspace.
///
/// `subsec_millis()` wraps every 1000 ms, so two creations within the same second
/// could collide; combine the full epoch-millis with a process-wide atomic counter
/// so every call is distinct regardless of timing.
fn test_session_name() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("muxrd-smoke-{millis}-{seq}")
}

// ─── smoke_list_sessions ──────────────────────────────────────────────────────

/// Verify `list_sessions()` returns without error against a live herdr.
///
/// Does NOT assert on a specific workspace list (the operator's herdr may have
/// zero or many); just confirms the JSON-API round-trip succeeds.
///
/// # Run
/// ```text
/// HERDR_SOCKET_PATH=/path/to/herdr.sock \
///   cargo test -p muxrd --test herdr_integration smoke_list_sessions -- --ignored
/// ```
#[test]
#[ignore = "requires a live herdr instance (set HERDR_SOCKET_PATH)"]
fn smoke_list_sessions() {
    let b = backend();
    let sessions = b
        .list_sessions()
        .expect("list_sessions() failed against live herdr");
    println!(
        "[herdr smoke] list_sessions → {} workspace(s)",
        sessions.len()
    );
    for (name, age) in &sessions {
        println!("  workspace: {name:?}  age: {age:?}");
    }
}

// ─── smoke_create_query_kill ──────────────────────────────────────────────────

/// Create a workspace, query its layout, then kill it.
///
/// Exercises the full JSON-API control round-trip:
/// `create_session` → `query_layout` → `kill_session`.
///
/// # Run
/// ```text
/// HERDR_SOCKET_PATH=/path/to/herdr.sock \
///   cargo test -p muxrd --test herdr_integration smoke_create_query_kill -- --ignored
/// ```
#[test]
#[ignore = "requires a live herdr instance (set HERDR_SOCKET_PATH); creates + destroys a workspace"]
fn smoke_create_query_kill() {
    let b = backend();
    let name = test_session_name();

    // Create.
    let ack = b
        .create_session(&name, None)
        .expect("create_session() failed");
    assert!(ack.ok, "create_session returned ok:false — {ack:?}");
    println!("[herdr smoke] created workspace {name:?}  ack={ack:?}");

    // Give herdr a moment to settle (workspace may not be immediately queryable).
    std::thread::sleep(Duration::from_millis(200));

    // Verify it appears in the session list.
    let sessions = b
        .list_sessions()
        .expect("list_sessions() failed after create");
    let found = sessions.iter().any(|(n, _)| n == &name);
    assert!(
        found,
        "workspace {name:?} not found in list after create: {sessions:?}"
    );

    // Query layout.
    let layout = b
        .query_layout(&name)
        .expect("query_layout() failed for newly created workspace");
    let total_panes: usize = layout.tabs.iter().map(|t| t.panes.len()).sum();
    println!(
        "[herdr smoke] layout tabs={} panes={}",
        layout.tabs.len(),
        total_panes,
    );

    // Kill.
    b.kill_session(&name).expect("kill_session() failed");
    println!("[herdr smoke] workspace {name:?} killed");

    // Confirm it is gone.
    std::thread::sleep(Duration::from_millis(100));
    let after = b
        .list_sessions()
        .expect("list_sessions() failed after kill");
    let still_present = after.iter().any(|(n, _)| n == &name);
    assert!(
        !still_present,
        "workspace {name:?} still listed after kill: {after:?}"
    );
}

// ─── smoke_open_attach_render_input ───────────────────────────────────────────

/// `open_attach` the focused pane of an existing workspace, read a few
/// `MuxServerMsg::Render` frames, send a test string, and tear down cleanly.
///
/// Requires at least one workspace to be present in herdr (create one manually
/// before running, or run `smoke_create_query_kill` first to prove creation works
/// then let a session linger).  The test selects the first listed workspace.
///
/// # Run
/// ```text
/// HERDR_SOCKET_PATH=/path/to/herdr.sock \
///   cargo test -p muxrd --test herdr_integration smoke_open_attach_render_input -- --ignored
/// ```
#[test]
#[ignore = "requires a live herdr instance with at least one workspace (set HERDR_SOCKET_PATH)"]
fn smoke_open_attach_render_input() {
    let b = backend();

    // Pick the first available workspace.
    let sessions = b.list_sessions().expect("list_sessions() failed");
    assert!(
        !sessions.is_empty(),
        "no workspaces found in herdr — create one before running this test"
    );
    let (session_name, _) = &sessions[0];
    println!("[herdr smoke] attaching to workspace {session_name:?}");

    // Open attach (24 rows × 80 cols, read-write).
    let handle = b
        .open_attach(session_name, 24, 80, false)
        .expect("open_attach() failed");
    println!(
        "[herdr smoke] attach open — session={:?}",
        handle.session_name
    );

    // Split into sender + receiver.
    let (mut sender, mut receiver) = handle.split();

    // Read up to 5 Render frames (or until EOF / 3-second wall clock).
    //
    // We move the receiver to a background thread so the wall-clock timeout
    // can be enforced on the main thread without blocking it indefinitely.
    let (frame_tx, frame_rx) = std::sync::mpsc::channel::<MuxServerMsg>();
    std::thread::spawn(move || {
        while let Some(msg) = receiver.recv() {
            let _ = frame_tx.send(msg);
        }
    });

    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    let mut render_count = 0usize;
    while render_count < 5 && std::time::Instant::now() < deadline {
        match frame_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(MuxServerMsg::Render(bytes)) => {
                render_count += 1;
                println!(
                    "[herdr smoke] Render frame #{render_count}: {} bytes",
                    bytes.len()
                );
            }
            Ok(other) => {
                println!("[herdr smoke] non-Render frame: {other:?}");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                println!("[herdr smoke] recv timeout after {render_count} Render frames");
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                println!("[herdr smoke] receiver thread finished");
                break;
            }
        }
    }

    assert!(
        render_count > 0,
        "expected at least one Render frame from herdr attach; got none"
    );

    // Send some input.
    sender
        .send_input_chars("echo herdr-smoke-ok\r")
        .expect("send_input_chars() failed");
    println!("[herdr smoke] sent test input");

    // Clean teardown.
    sender.send_client_exited().ok();
    println!("[herdr smoke] client exited; test complete");
}
