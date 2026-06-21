# ZelliMobile gRPC rig (Docker)

A Debian container running **zellimserver** — the ZelliMobile gRPC backend —
pre-loaded with a realistic Zellij session and a full set of terminal tools so
the mobile client has a real, interesting target.

**Zellij is pinned to v0.44.3** (the version zellimserver was compiled against;
it refuses to start on any other version).

## What's inside

- **zellimserver** (static musl binary built from the repo's `zellimserver/` crate)
  — TLS gRPC server on port **50051** with self-signed cert + bearer-token auth.
- **Zellij v0.44.3** running a pre-populated `backend-dev` session (see `layout.kdl`):
  an `editor` tab (nvim + shell + btop), a `shell` tab (shell + htop), and a
  `logs` tab (live log stream).
- Terminal tooling: **Neovim + NvChad**, **btop**, htop, lazygit, ripgrep, fd,
  fzf, bat, tree, jq, ncdu, tmux, git, node/npm, python3, plus toys.

## Quickstart — loopback (local testing)

```bash
# From the repo root — binds 127.0.0.1:50051 (nothing exposed on the network)
docker compose -f docker/compose.yaml up --build
# or via the helper:
./docker/run.sh
```

On boot the container prints a **connection banner** with:
- **gRPC URL** — `https://127.0.0.1:50051`
- **Token** — the bearer token (also saved to `rig-token.txt` inside the container)
- **Cert path** — the self-signed PEM cert for your client to trust

Grab them from the startup output or with:
```bash
sudo docker compose -f docker/compose.yaml logs
```

## LAN / phone access

To test from a real Android phone the container must bind to a **LAN IP**
reachable by both the host and the phone (same network).

```bash
# Replace with the host's actual LAN IP
LAN_IP="192.168.1.50"

# Option A — helper wrapper (recommended)
BIND_ADDR="${LAN_IP}" ./docker/run.sh --host "${LAN_IP}"

# Option B — compose directly
BIND_ADDR="${LAN_IP}" DEV_HOST="${LAN_IP}" \
  docker compose -f docker/compose.yaml up --build
```

`BIND_ADDR` controls which host interface the port is published on.  
`DEV_HOST` is placed in the cert's **Subject Alternative Name** so the phone
can validate the self-signed cert (or use the app's insecure-dev mode).

The cert SAN must match the IP/hostname the phone uses to connect — `DEV_HOST`
drives this. Both env vars default to `127.0.0.1` (loopback, nothing exposed).

## Connecting the mobile client / Dart test client

| Field    | Value                                                                          |
|----------|--------------------------------------------------------------------------------|
| Host     | `DEV_HOST` (e.g. `127.0.0.1` or your LAN IP)                            |
| Port     | `50051`                                                                        |
| Token    | The UUID printed in the startup banner                                         |
| TLS      | Self-signed cert — trust the PEM or use `--insecure` / the app's dev TLS mode |

Copy the cert out of the container:
```bash
docker compose -f docker/compose.yaml exec zellimserver \
  cat /root/.local/share/zellij/zellimserver/server.crt > /tmp/rig-server.crt
```

## Auth token & TLS cert — where to get them

The container name is **`zellimobile-grpc-rig`**. Locally (your user in the `docker`
group) no `sudo` is needed; otherwise prefix every `docker` command with `sudo`.

```bash
# The bearer token (UUID) the mobile / Dart client logs in with.
# NOTE: the entrypoint REVOKES + RECREATES the 'rig' token on every boot,
# so this UUID ROTATES each time the container restarts.
docker exec zellimobile-grpc-rig cat /root/.local/share/zellimserver/rig-token.txt

# List token names + read-only flag (does NOT print the secret UUID):
docker exec zellimobile-grpc-rig zellimserver list-tokens

# Mint an extra token (prints a fresh UUID on stdout):
docker exec zellimobile-grpc-rig zellimserver create-token --name mytoken
# read-only variant:
docker exec zellimobile-grpc-rig zellimserver create-token --name viewer --read-only

# The self-signed TLS cert (PEM) — for clients that pin/trust it:
docker exec zellimobile-grpc-rig \
  cat /root/.local/share/zellij/zellimserver/server.crt > /tmp/rig-server.crt
```

The token + URL + cert path are also printed in the **startup banner**:
`docker compose -f docker/compose.yaml logs zellimserver`.

## Shell into the container / view the live Zellij session

The pre-populated session is named **`backend-dev`** (the `SESSION` env var). To watch /
drive the exact session the mobile client is attached to, exec a TTY into the container
and attach a real Zellij client:

```bash
# Interactive shell in the container:
docker exec -it zellimobile-grpc-rig bash

# …or attach straight to the live session (TERM must be set; -it gives a TTY):
docker exec -it zellimobile-grpc-rig env TERM=xterm-256color zellij attach backend-dev
# Detach (leave it running):  Ctrl-o then d
```

Inspect without attaching a full client (no geometry impact):
```bash
docker exec zellimobile-grpc-rig zellij --session backend-dev action list-tabs
docker exec zellimobile-grpc-rig zellij --session backend-dev action list-panes
```

> ⚠️ **Heads-up for single-pane on-device testing:** attaching your own Zellij client
> adds a second client to the session. Zellij sizes the session to the **smallest**
> attached client, so your terminal's size will resize what the phone sees; and the
> server's single-pane **fullscreen is gated to the sole-client case**, so it is
> disabled while you're attached. **Detach (`Ctrl-o d`) when done** to restore the
> phone's view.

## Running the Dart / gRPC test client

```bash
# From zellimserver/clients/dart_test_client/ (task 3)
dart run bin/zelli_client.dart \
  --host 127.0.0.1 --port 50051 \
  --token <paste-token-here> \
  --cert /tmp/rig-server.crt
```

## Using `read_client` (Rust example)

```bash
# From zellimserver/
cargo run --example read_client -- \
  --addr 127.0.0.1:50051 \
  --auth-token <token> \
  --cert /tmp/rig-server.crt
```

## run.sh flags

```
./docker/run.sh [--host <IP>] [-- EXTRA_COMPOSE_ARGS...]
```

| Flag           | Default       | Description                                       |
|----------------|---------------|---------------------------------------------------|
| `--host <IP>`  | `127.0.0.1`   | Bind port + cert SAN to this address              |

## Notes

- **Named volumes** persist the token DB, cert, and zellij config across
  restarts. Use `docker compose down -v` to reset everything.
- **Rotating the token:** `docker compose exec zellimserver zellimserver create-token --name rig2`
- **Zellij version:** must remain 0.44.3. Upgrading zellij without recompiling
  zellimserver will cause a version-mismatch error at startup.
- **Security:** this is a **dev/test rig** — the self-signed cert and the
  `BIND_ADDR` LAN exposure are intentional dev affordances. Do not expose
  this container on an untrusted network.
