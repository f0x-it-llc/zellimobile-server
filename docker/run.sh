#!/usr/bin/env bash
# run.sh — convenience wrapper for the ZelliMobile gRPC rig.
#
# Usage:
#   ./docker/run.sh [OPTIONS] [-- EXTRA_COMPOSE_ARGS...]
#
# Options:
#   --host <IP>     Bind the gRPC port to this host interface and set DEV_HOST
#                   (used for the cert SAN and banner URL).
#                   Default: 127.0.0.1 (loopback — nothing exposed on the network).
#   --host=<IP>     Same, equals-sign form.
#   -h, --help      Show this help and exit.
#
# Examples:
#   # Loopback only — safe for local testing:
#   ./docker/run.sh
#
#   # Expose on the LAN so a phone can connect:
#   ./docker/run.sh --host 192.168.1.50
#
#   # Build without cache, then run:
#   ./docker/run.sh --host 192.168.1.50 -- --no-deps
#
# The token and cert are printed at startup; also in:
#   sudo docker compose -f docker/compose.yaml logs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="${SCRIPT_DIR}/compose.yaml"

usage() {
  grep '^#' "${BASH_SOURCE[0]}" | sed 's/^# \?//'
  exit 0
}

BIND_ADDR="127.0.0.1"
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

# DEV_HOST: the hostname/IP placed in the cert SAN and the banner URL.
# When BIND_ADDR is non-loopback (a real network address), DEV_HOST should
# match it so the phone can validate the self-signed cert.
if [[ -z "${DEV_HOST:-}" ]]; then
  if [[ "${BIND_ADDR}" != "127.0.0.1" && "${BIND_ADDR}" != "::1" && "${BIND_ADDR}" != "localhost" ]]; then
    DEV_HOST="${BIND_ADDR}"
  else
    DEV_HOST="127.0.0.1"
  fi
fi

export BIND_ADDR
export DEV_HOST

echo "[run.sh] BIND_ADDR=${BIND_ADDR}  DEV_HOST=${DEV_HOST}"
echo "[run.sh] gRPC will be published at: ${BIND_ADDR}:${GRPC_PORT:-50051}"
echo "[run.sh] Cert SAN will cover:       ${DEV_HOST}"
echo ""

exec sudo docker compose -f "${COMPOSE_FILE}" up --build "${EXTRA_ARGS[@]}"
