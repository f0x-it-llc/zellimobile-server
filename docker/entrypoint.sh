#!/usr/bin/env bash
# Entrypoint for the ZelliMobile dev rig.
#
# Boots a backgrounded Zellij session (pre-populated via layout.kdl) and an SSH
# server, then waits. You SSH in (no password) and run `zellimctl` to configure
# the cert + tokens and start the gRPC server:
#
#   ssh -t root@<host> -p <ssh-port>      # no password
#   zellimctl                             # Configure → Cert → Tokens → Server → Pair
#
# Environment variables:
#   SESSION  — zellij session name  (default: backend-dev)
set -euo pipefail

SESSION="${SESSION:-backend-dev}"

# ── 1. Clear root's password so SSH login needs no credential (dev rig) ──────
passwd -d root

# ── 2. Start a backgrounded zellij session from the layout ───────────────────
# zellimserver attaches to this live session once you start it from zellimctl.
echo "[rig] starting zellij session '${SESSION}'…"
zellij --layout zellimobile attach --create-background "${SESSION}" || true

# ── 3. Connection banner ─────────────────────────────────────────────────────
cat <<BANNER

╔══════════════════════════════════════════════════════════════════╗
║  ZelliMobile dev rig is up — drive it with zellimctl over SSH     ║
╠══════════════════════════════════════════════════════════════════╣
  1. SSH in — no password (a TTY is required for the TUI — note the -t):
       ssh -t root@<host> -p <ssh-port>

  2. Run the control TUI and start the server:
       zellimctl
     → Configure → Cert → Tokens → Server (start) → Pair (scan the QR)

  zellij session : ${SESSION}
  gRPC port      : 50051 (published once you start the server)
╚══════════════════════════════════════════════════════════════════╝

BANNER

# ── 4. Run sshd in the foreground (keeps the container alive) ─────────────────
mkdir -p /run/sshd
echo "[rig] starting sshd (foreground)…"
exec /usr/sbin/sshd -D -e
