# zellimobile-server

The open-source backend for **ZelliMobile** — a Rust [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)
that lets a mobile client attach to and control remote [Zellij](https://zellij.dev/)
terminal-multiplexer sessions over a TLS, bearer-authenticated gRPC API.

Two binaries:

| Crate | Binary | What it does |
|-------|--------|--------------|
| [`zellimserver`](zellimserver/) | `zellimserver` | gRPC server (protobuf package `zellimserver.v1`) that relays over Zellij's Unix-domain IPC. TLS + per-token auth, read-only tokens, daemonize. |
| [`zellimctl`](zellimctl/) | `zellimctl` | Terminal UI to install, configure, and pair the server: cert/SAN setup, token management, QR-code device pairing, live status. Links `zellimserver` as a library for its pure ops. |

## Build

```bash
cargo build              # both binaries (debug)
cargo build --release    # both binaries (release)
cargo build -p zellimserver   # just the server
cargo build -p zellimctl      # just the TUI
```

## Run

```bash
# Generate the TLS cert, then serve:
cargo run -p zellimserver -- init
ZELLIMSERVER_SKIP_VERSION_CHECK=1 cargo run -p zellimserver -- start

# The configure/pair TUI:
cargo run -p zellimctl
```

`start` opens a control socket for `status`/`stop`; add `--daemonize` to detach.
`ZELLIMSERVER_SKIP_VERSION_CHECK=1` bypasses the Zellij version-match check.

Requires the matching `zellij` binary on `PATH` (the server pins a Zellij
version and refuses to start against a different one).

## Test

```bash
cargo test               # workspace unit + integration tests
cargo fmt                # format before committing
```

## Docker dev rig

`docker/` builds a self-contained container running the server against a
pre-populated Zellij session — useful for on-device testing from a phone on the
same network. See [`docker/README.md`](docker/README.md).

```bash
docker compose -f docker/compose.yaml up --build
```

## gRPC contract

The wire contract is `zellimserver/proto/zellimserver.proto` (package
`zellimserver.v1`). The server compiles it via `build.rs`; clients generate
their own stubs from the same file. A reference Dart client lives in
[`zellimserver/clients/dart_test_client/`](zellimserver/clients/dart_test_client/).

## License

[MIT](LICENSE).
