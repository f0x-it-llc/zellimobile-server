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

# ── 0. Propagate the zellim env to SSH login shells ──────────────────────────
# sshd does NOT pass the container's docker `ENV` (ZELLIMSERVER_BIND /
# ZELLIMSERVER_SAN) to interactive sessions, so `zellimctl` run over SSH would
# not see them — the cert SAN and the pairing-QR advertise host would silently
# fall back to the container-internal bridge IP instead of the advertised
# tailnet/LAN address. The entrypoint runs WITH the container env, so mirror the
# relevant vars where SSH logins pick them up: /etc/environment (read by PAM for
# every session) and /etc/profile.d (sourced by login shells).
printf 'ZELLIMSERVER_BIND=%s\nZELLIMSERVER_SAN=%s\n' \
  "${ZELLIMSERVER_BIND:-}" "${ZELLIMSERVER_SAN:-}" > /etc/environment
printf "export ZELLIMSERVER_BIND='%s'\nexport ZELLIMSERVER_SAN='%s'\n" \
  "${ZELLIMSERVER_BIND:-}" "${ZELLIMSERVER_SAN:-}" > /etc/profile.d/zellim-env.sh
chmod 0644 /etc/profile.d/zellim-env.sh

# ── 1. Clear root's password so SSH login needs no credential (dev rig) ──────
passwd -d root

# ── 1b. Install the managed zellij config ────────────────────────────────────
# zellij's bundled default config ships `load_plugins { "zellij:link" }`, a
# background plugin that loads on every new session and showed up as a stray pane
# in app-created sessions. We pin a minimal config with an empty `load_plugins` so
# no background plugin loads (zellij fills its built-in defaults for everything the
# minimal config omits). /root/.config/zellij is a persisted named volume, so we
# (re)install on every boot rather than relying on the build-time COPY, which only
# seeds a fresh volume.
mkdir -p /root/.config/zellij
cp /usr/local/share/zellimobile/config.kdl /root/.config/zellij/config.kdl

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
