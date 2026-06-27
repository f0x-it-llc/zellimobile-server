# muxr-core

The open-source backend for **Muxr** — a Rust [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)
that lets a mobile client attach to and control remote [Zellij](https://zellij.dev/)
terminal-multiplexer sessions over a TLS, bearer-authenticated gRPC API.

Two binaries:

| Crate | Binary | What it does |
|-------|--------|--------------|
| [`muxrd`](muxrd/) | `muxrd` | gRPC server (protobuf package `muxr.v1`) that relays over Zellij's Unix-domain IPC. TLS (self-signed, an external CA cert, or plaintext h2c behind a proxy) + per-token auth, read-only tokens, daemonize. |
| [`muxrctl`](muxrctl/) | `muxrctl` | Terminal UI to install, configure, and pair the server: cert/SAN setup, token management, QR-code device pairing (fingerprint-pinned or system-CA), live status. Links `muxrd` as a library for its pure ops. |

## Build

```bash
cargo build              # both binaries (debug)
cargo build --release    # both binaries (release)
cargo build -p muxrd   # just the server
cargo build -p muxrctl      # just the TUI
```

## Run

```bash
# Generate the TLS cert, then serve:
cargo run -p muxrd -- init
ZELLIMSERVER_SKIP_VERSION_CHECK=1 cargo run -p muxrd -- start

# The configure/pair TUI:
cargo run -p muxrctl
```

`start` opens a control socket for `status`/`stop`; add `--daemonize` to detach.
`ZELLIMSERVER_SKIP_VERSION_CHECK=1` bypasses the Zellij version-match check.

Requires the matching `zellij` binary on `PATH` (the server pins a Zellij
version and refuses to start against a different one).

## TLS modes & deployment

The server resolves its TLS identity by precedence **h2c > external cert > self-signed**:

| Mode | Flags / env | Use for |
|------|-------------|---------|
| **Self-signed** (default) | _(none)_ — generated for `127.0.0.1` + `localhost` + any `--san` extras | Direct / LAN connections. The mobile client pins the cert's SHA-256 fingerprint, distributed out-of-band in the pairing QR. |
| **External cert** | `--tls-cert <pem> --tls-key <pem>` (or `ZELLIMSERVER_TLS_CERT` / `ZELLIMSERVER_TLS_KEY`) | Serving a real, publicly-trusted cert directly — Let's Encrypt, a Cloudflare Origin CA cert, or a corporate CA. The client trusts it via the system CA store; no pinning. Both files are validated at `init`/`start`. |
| **Plaintext h2c** | `--insecure-h2c` (or `ZELLIMSERVER_H2C=1`) | Sitting behind a TLS-terminating reverse proxy (Traefik / Dokploy / Cloudflare) that owns the public cert. Serves **unencrypted** HTTP/2, so it **refuses a non-loopback bind** unless you also pass `--i-know-this-is-behind-a-proxy` (env `ZELLIMSERVER_H2C_ALLOW_PUBLIC`). |

External and h2c are mutually exclusive with each other. `muxrd init` validates the
chosen mode (e.g. parses the external key) so misconfigurations surface before `start`.

`muxrctl` detects the active mode over the control socket and builds the pairing QR to match:
a **fingerprint-pinned** pairing (`tm=pin`) for self-signed, or a **system-CA** pairing (`tm=ca`,
no fingerprint) for external/h2c. Press **`t`** on the Cert screen to override the advertised trust
(**Auto → CA → Pin**) — needed when a *self-signed* origin sits behind a CA-terminating proxy. The
choice persists across restarts.

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

**Tailnet / LAN exposure:** set `BIND_ADDR` to the host IP you want to publish
on — the cert's SAN is automatically set to that IP so clients connecting on it
get a valid TLS cert. Override `ZELLIMSERVER_SAN` explicitly to cover a
different or additional address (comma-separated).

```bash
# Publish + cert-valid on a tailnet IP:
BIND_ADDR=100.x.y.z docker compose -f docker/compose.yaml up --build
```

## gRPC contract

The wire contract is `muxrd/proto/muxr.proto` (package
`muxr.v1`). The server compiles it via `build.rs`; clients generate
their own stubs from the same file. A reference Dart client lives in
[`muxrd/clients/dart_test_client/`](muxrd/clients/dart_test_client/).

## License

[MIT](LICENSE).
