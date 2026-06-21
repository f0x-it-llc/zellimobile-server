#!/usr/bin/env bash
# Entrypoint for the ZelliMobile gRPC rig.
#
# Boots a backgrounded Zellij session (pre-populated via layout.kdl), initialises
# the zellimserver cert (with the configured SAN), creates an API token, prints a
# connection banner, then starts the gRPC server in the foreground.
#
# Environment variables:
#   SESSION    — zellij session name  (default: backend-dev)
#   DEV_HOST   — hostname/IP used in the cert SAN + banner URLs
#                (default: 127.0.0.1; set to a LAN IP for phone access)
#   GRPC_PORT  — gRPC listen port inside the container  (default: 50051)
set -euo pipefail

SESSION="${SESSION:-backend-dev}"
GRPC_PORT="${GRPC_PORT:-50051}"
DEV_HOST="${DEV_HOST:-127.0.0.1}"

TOKEN_FILE="/root/.local/share/zellimserver/rig-token.txt"

# ── 1. Start a backgrounded zellij session from the layout ───────────────────
# The session must exist before zellimserver start (it queries live sessions).
echo "[rig] starting zellij session '${SESSION}'…"
zellij --layout zellimobile attach --create-background "${SESSION}" || true

# ── 2. Init zellimserver cert (idempotent; covers DEV_HOST in SAN) ───────────
echo "[rig] running: zellimserver init --san '${DEV_HOST}'"
zellimserver init --san "${DEV_HOST}"

# ── 3. Create an API token for the rig ───────────────────────────────────────
# Revoke any existing 'rig' token first (idempotent — ignore errors if absent).
# revoke-token takes NAME as a positional argument (not --name).
echo "[rig] revoking any previous 'rig' token (idempotent)…"
zellimserver revoke-token rig 2>/dev/null || true

# create-token prints a UUID on stdout (possibly with surrounding text);
# extract the UUID robustly.
echo "[rig] creating API token 'rig'…"
TOKEN_RAW="$(zellimserver create-token --name rig 2>&1)"
TOKEN="$(printf '%s' "${TOKEN_RAW}" \
  | grep -oiE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' \
  | head -1)"

if [[ -z "${TOKEN}" ]]; then
  echo "[rig] WARNING: could not parse a UUID from create-token output:"
  echo "${TOKEN_RAW}"
  echo "[rig] Proceeding — token may already exist under this name."
  TOKEN="${TOKEN_RAW}"
fi

# Persist so the user can retrieve it later without inspecting logs.
mkdir -p "$(dirname "${TOKEN_FILE}")"
echo "${TOKEN}" > "${TOKEN_FILE}"

# ── 4. Print the connection banner ───────────────────────────────────────────
# zellimserver's data dir is under zellij's XDG share dir.
# The cert is always at this path (init created it just above).
CERT_PATH="/root/.local/share/zellij/zellimserver/server.crt"

cat <<BANNER

╔══════════════════════════════════════════════════════════════════╗
║  ZelliMobile gRPC rig is starting                                ║
╠══════════════════════════════════════════════════════════════════╣
  gRPC URL  : https://${DEV_HOST}:${GRPC_PORT}
  Session   : ${SESSION}
  Token     : ${TOKEN}
  Cert      : ${CERT_PATH}

  (token also written to ${TOKEN_FILE})

  TLS is a self-signed cert — the phone / Dart client must either:
    • Trust the cert PEM explicitly, OR
    • Use the app's insecure-dev / --insecure mode.
╚══════════════════════════════════════════════════════════════════╝

BANNER

# ── 5. Start the gRPC server in the foreground (keeps the container alive) ───
# Bind on all interfaces so the host can reach it via the mapped port.
# DEV_HOST / --san ensures the cert covers the LAN IP for the phone.
echo "[rig] starting: zellimserver start --bind 0.0.0.0:${GRPC_PORT} --san '${DEV_HOST}'"
exec zellimserver start --bind "0.0.0.0:${GRPC_PORT}" --san "${DEV_HOST}"
