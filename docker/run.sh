#!/usr/bin/env bash
# run.sh — convenience wrapper for the Muxr dev rig.
#
# Usage:
#   ./docker/run.sh [OPTIONS] [-- EXTRA_COMPOSE_ARGS...]
#
# Options:
#   --host <IP>     Publish the gRPC + SSH ports on this host interface.
#                   Default: 127.0.0.1 (loopback — nothing exposed on the network).
#   --host=<IP>     Same, equals-sign form.
#   --herdr         Run the herdr-backend rig instead of the default zellij rig
#                   (downloads a pinned, unmodified upstream herdr binary; muxrd
#                   drives it via `--backend herdr`). Run ONE rig at a time.
#   -h, --help      Show this help and exit.
#
# Examples:
#   # Loopback only — safe for local testing (zellij backend):
#   ./docker/run.sh
#
#   # herdr backend rig (loopback):
#   ./docker/run.sh --herdr
#
#   # Expose on the LAN so a phone can connect:
#   ./docker/run.sh --host 192.168.1.50
#
#   # Build without cache, then run:
#   ./docker/run.sh --host 192.168.1.50 -- --no-deps
#
# Once it is up, SSH in (no password) and start the server with muxrctl:
#   ssh -t root@<host> -p 2222
#   muxrctl

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="${SCRIPT_DIR}/compose.yaml"

usage() {
  grep '^#' "${BASH_SOURCE[0]}" | sed 's/^# \?//'
  exit 0
}

BIND_ADDR="127.0.0.1"
HERDR=0
EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host=*)
      BIND_ADDR="${1#--host=}"
      shift
      ;;
    --host)
      shift
      BIND_ADDR="${1:?--host requires an IP/hostname argument}"
      shift
      ;;
    --herdr)
      HERDR=1
      shift
      ;;
    -h|--help)
      usage
      ;;
    --)
      shift
      EXTRA_ARGS+=("$@")
      break
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      ;;
  esac
done

export BIND_ADDR

echo "[run.sh] BIND_ADDR=${BIND_ADDR}  backend=$([ "${HERDR}" -eq 1 ] && echo herdr || echo zellij)"
echo "[run.sh] publishing  gRPC ${BIND_ADDR}:${GRPC_PORT:-50051}  +  SSH ${BIND_ADDR}:${SSH_PORT:-2222}"
echo "[run.sh] after boot:  ssh -t root@${BIND_ADDR} -p ${SSH_PORT:-2222}  then run  muxrctl"
echo ""

if [[ "${HERDR}" -eq 1 ]]; then
  # Profile-gated herdr service; name it explicitly so the default zellij service
  # isn't also started (it would clash on the published ports).
  exec sudo docker compose -f "${COMPOSE_FILE}" --profile herdr up --build muxrd-herdr "${EXTRA_ARGS[@]}"
else
  exec sudo docker compose -f "${COMPOSE_FILE}" up --build "${EXTRA_ARGS[@]}"
fi
