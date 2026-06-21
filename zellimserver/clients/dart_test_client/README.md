# zelli_client — Dart gRPC test client

Headless Dart CLI that exercises the full zellimserver gRPC API end-to-end
against the dockerized development rig. Seeds the Phase-F Flutter client:
the generated proto bindings, channel/TLS setup, and auth interceptor code
are designed to be lifted directly into the Flutter app.

## What it tests

1. **GetVersion** — no auth, verifies server is up and version matches
2. **Login** — exchanges an auth token for a session token
3. **ListSessions** — lists live zellij sessions
4. **GetLayout** — prints the tab/pane tree for the first session
5. **AttachTerminal** — opens a bidi stream, sends AttachReq, reads render frames
6. **NewTab** (optional) — creates a new tab and verifies +1 tab count

Exit 0 if all mandatory steps pass, non-zero otherwise.

## Prerequisites

- Dart SDK ≥ 3.0 (`dart --version`)
- The `zellimserver` Docker rig running on a host (`docker compose -f docker/compose.yaml up -d`)

## Quickstart (loopback — same machine as the rig)

```bash
# 1. Get deps
cd zellimserver/clients/dart_test_client
dart pub get

# 2. Get the auth token from the rig startup logs
TOKEN=$(sudo docker compose -f docker/compose.yaml logs | grep -i 'auth token' | tail -1 | grep -oE '[0-9a-f-]{36}')

# 3. Copy the self-signed cert out of the container
sudo docker compose -f docker/compose.yaml exec -T zellimserver \
  cat /root/.local/share/zellij/zellimserver/server.crt > /tmp/rig-server.crt

# 4. Run against loopback
dart run bin/zelli_client.dart \
  --host 127.0.0.1 \
  --port 50051 \
  --token "$TOKEN" \
  --cert /tmp/rig-server.crt \
  --server-name localhost
```

## Insecure dev shortcut (--insecure)

Skips certificate validation — useful when the cert is unavailable or you're
iterating on the server without re-copying the cert.

```bash
dart run bin/zelli_client.dart \
  --host 127.0.0.1 \
  --port 50051 \
  --token "$TOKEN" \
  --insecure
```

**Phase-F note:** on Android/iOS, the equivalent is a custom
`HttpOverrides.global` with `badCertificateCallback: (_, __, ___) => true`,
or a `SecurityContext(withTrustedRoots: false)..setTrustedCertificatesBytes(pem)`
loaded from app assets for the cert-pinning path.

## Connecting to the rig over the LAN (from a remote machine)

When the rig is started with `BIND_ADDR=<lan-ip>`:

```bash
# 1. Copy the cert from the rig
scp <rig-host>:/tmp/rig-server.crt /tmp/

# 2. Run pointing at the LAN IP
dart run bin/zelli_client.dart \
  --host <lan-ip> \
  --port 50051 \
  --token "$TOKEN" \
  --cert /tmp/rig-server.crt \
  --server-name <lan-ip>
```

The `--server-name` must match a SAN in the cert. The rig generates the cert
with `--san <DEV_HOST>` so the LAN IP is included.

## Getting the token

The auth token is printed in the container startup banner:

```
sudo docker compose -f docker/compose.yaml logs | grep -i token
```

It is also the same token every time until the container is recreated
(the rig revokes and re-creates the "rig" token on each start).

## Regenerating bindings

The generated files in `lib/src/generated/` are committed and do not need
to be regenerated unless `proto/zellimserver.proto` changes.

To regenerate:

```bash
# Requires: protoc + protoc-gen-dart (dart pub global activate protoc_plugin)
./gen.sh
```

## Phase-F Flutter integration notes

- **Channel**: copy the `ClientChannel` construction (host/port/credentials) into
  a Dart service class initialized at app start.
- **Auth interceptor**: `BearerInterceptor` is a drop-in `ClientInterceptor`
  usable in Flutter. Replace the static token with a `ValueNotifier<String>` or
  Riverpod provider that updates after `Login`.
- **TLS — cert pinning**: load the PEM from Flutter assets (`rootBundle.load`)
  then `SecurityContext()..setTrustedCertificatesBytes(bytes)`. Pass via
  `ChannelCredentials.secure(certificates: bytes, authority: serverName)`.
- **AttachTerminal**: the `StreamController<ClientFrame>` pattern maps directly
  to a Flutter terminal widget's keyboard/resize event stream. The
  `ResponseStream<ServerFrame>` is the render source.
- **Generated files**: copy `lib/src/generated/*.pb*.dart` into the Flutter
  app's proto package (or use a shared Dart package).
