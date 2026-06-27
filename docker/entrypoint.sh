#!/usr/bin/env bash
# Entrypoint for the Muxr dev rig.
#
# Boots the selected multiplexer backend + an SSH server, then waits. You SSH in
# (no password) and run `muxrctl` to configure the cert + tokens and start the
# gRPC server:
#
#   ssh -t root@<host> -p <ssh-port>      # no password
#   muxrctl                             # Configure → Cert → Tokens → Server → Pair
#
# Environment variables:
#   BACKEND  — multiplexer backend: `zellij` (default) | `herdr`
#   SESSION  — session / herdr-workspace name  (default: backend-dev)
set -euo pipefail

BACKEND="${BACKEND:-zellij}"
SESSION="${SESSION:-backend-dev}"
HERDR_SOCKET_PATH="${HERDR_SOCKET_PATH:-/root/.config/herdr/herdr.sock}"

# ── 0. Propagate the rig env to SSH login shells ─────────────────────────────
# sshd does NOT pass the container's docker `ENV` to interactive sessions, so
# `muxrctl` run over SSH would not see them — and the muxrctl-spawned `muxrd`
# daemon inherits muxrctl's env, so for the herdr backend MUXRD_BACKEND +
# HERDR_SOCKET_PATH must land here too or the daemon would default to zellij.
# The entrypoint runs WITH the container env, so mirror the relevant vars where
# SSH logins pick them up: /etc/environment (read by PAM for every session) and
# /etc/profile.d (sourced by login shells).
{
  printf 'ZELLIMSERVER_BIND=%s\nZELLIMSERVER_SAN=%s\n' \
    "${ZELLIMSERVER_BIND:-}" "${ZELLIMSERVER_SAN:-}"
  if [ "${BACKEND}" = "herdr" ]; then
    printf 'MUXRD_BACKEND=herdr\nHERDR_SOCKET_PATH=%s\n' "${HERDR_SOCKET_PATH}"
  fi
} > /etc/environment
{
  printf "export ZELLIMSERVER_BIND='%s'\nexport ZELLIMSERVER_SAN='%s'\n" \
    "${ZELLIMSERVER_BIND:-}" "${ZELLIMSERVER_SAN:-}"
  if [ "${BACKEND}" = "herdr" ]; then
    printf "export MUXRD_BACKEND='herdr'\nexport HERDR_SOCKET_PATH='%s'\n" "${HERDR_SOCKET_PATH}"
  fi
} > /etc/profile.d/zellim-env.sh
chmod 0644 /etc/profile.d/zellim-env.sh

# ── 1. Clear root's password so SSH login needs no credential (dev rig) ──────
passwd -d root

# ── 2. Start the selected backend ────────────────────────────────────────────
if [ "${BACKEND}" = "herdr" ]; then
  # ── herdr: a SEPARATE, UNMODIFIED, user-installed binary (AGPL-3.0). muxrd
  # drives it only over its public 0600 sockets; muxrd stays the TLS/bearer
  # boundary. Start a headless herdr server and seed a demo workspace so the app
  # has a session to attach to. muxrd selects herdr via MUXRD_BACKEND (above).
  export HERDR_SOCKET_PATH
  echo "[rig] starting headless herdr server…"
  herdr server > /var/log/herdr-server.log 2>&1 &
  # Wait for the API socket to appear (herdr derives the wire socket alongside).
  for _ in $(seq 1 50); do [ -S "${HERDR_SOCKET_PATH}" ] && break; sleep 0.1; done
  if [ -S "${HERDR_SOCKET_PATH}" ]; then
    herdr status server 2>&1 | sed 's/^/[rig][herdr] /' || true
    # Seed one demo workspace (idempotent across restarts: skip if the label
    # already exists — muxrd treats duplicate workspace labels as ambiguous).
    if ! herdr workspace list 2>/dev/null | grep -q "${SESSION}"; then
      echo "[rig] seeding herdr workspace '${SESSION}'…"
      herdr workspace create --label "${SESSION}" --cwd /root/projects/api --focus 2>/dev/null || true
      herdr tab create --label logs --no-focus 2>/dev/null || true
    fi
  else
    echo "[rig] WARNING: herdr socket ${HERDR_SOCKET_PATH} did not appear — see /var/log/herdr-server.log" >&2
  fi
else
  # ── zellij (default): install the managed config (empty load_plugins so the
  # zellij:link background plugin never loads) into the persisted config volume,
  # then start a backgrounded session muxrd attaches to once started from muxrctl.
  mkdir -p /root/.config/zellij
  cp /usr/local/share/muxr/config.kdl /root/.config/zellij/config.kdl
  echo "[rig] starting zellij session '${SESSION}'…"
  zellij --layout muxr attach --create-background "${SESSION}" || true
fi

# ── 3. Connection banner ─────────────────────────────────────────────────────
cat <<BANNER

╔══════════════════════════════════════════════════════════════════╗
║  Muxr dev rig is up — drive it with muxrctl over SSH     ║
╠══════════════════════════════════════════════════════════════════╣
  1. SSH in — no password (a TTY is required for the TUI — note the -t):
       ssh -t root@<host> -p <ssh-port>

  2. Run the control TUI and start the server:
       muxrctl
     → Configure → Cert → Tokens → Server (start) → Pair (scan the QR)

  backend        : ${BACKEND}$([ "${BACKEND}" = "herdr" ] && printf '  (wire protocol 14; muxrd auto-selects via MUXRD_BACKEND)')
  session/space  : ${SESSION}
  gRPC port      : 50051 (published once you start the server)
╚══════════════════════════════════════════════════════════════════╝

BANNER

# ── 4. Run sshd in the foreground (keeps the container alive) ─────────────────
mkdir -p /run/sshd
echo "[rig] starting sshd (foreground)…"
exec /usr/sbin/sshd -D -e
