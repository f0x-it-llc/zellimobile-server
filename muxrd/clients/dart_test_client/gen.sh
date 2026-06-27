#!/usr/bin/env bash
# gen.sh — generate Dart gRPC bindings from muxr.proto
#
# Requirements (all must be on PATH):
#   protoc           — protobuf compiler (apt: protobuf-compiler)
#   protoc-gen-dart  — Dart protoc plugin (dart pub global activate protoc_plugin)
#
# Run from the dart_test_client/ package root:
#   ./gen.sh
#
# Output: lib/src/generated/
#   muxr.pb.dart        — message classes
#   muxr.pbgrpc.dart    — gRPC service stubs
#   muxr.pbenum.dart    — enum classes
#   muxr.pbserver.dart  — server-side stubs (unused here but generated)
#
# Note: the generated files are committed to the repo so users don't need
# protoc installed to run or develop the client.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR/lib/src/generated"
PROTO_DIR="$SCRIPT_DIR/proto"

mkdir -p "$OUT_DIR"

echo "Generating Dart gRPC bindings..."
protoc \
  --dart_out=grpc:"$OUT_DIR" \
  -I"$PROTO_DIR" \
  "$PROTO_DIR/muxr.proto"

echo "Done. Generated files in $OUT_DIR:"
ls -1 "$OUT_DIR"
