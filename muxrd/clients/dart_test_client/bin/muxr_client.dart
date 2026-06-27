/// muxr_client.dart — End-to-end gRPC test client for muxrd.
///
/// Exercises GetVersion → Login → ListSessions → GetLayout →
/// AttachTerminal (bidi stream) → NewTab (optional) against the dockerized rig.
///
/// The channel/auth/stream patterns here are the seed for the Phase-F
/// Flutter client.
///
/// Usage:
///   dart run bin/muxr_client.dart \
///     --token <auth-token> \
///     [--host 127.0.0.1] [--port 50051] \
///     [--cert /path/to/server.crt] [--server-name localhost] \
///     [--insecure]
library;

import 'dart:io';
import 'dart:async';

import 'package:args/args.dart';
import 'package:grpc/grpc.dart';

// pbgrpc.dart re-exports pb.dart so a single import suffices.
import 'package:muxr_client/src/generated/muxr.pbgrpc.dart';

// ─── result tracking ─────────────────────────────────────────────────────────

final List<String> _passed = [];
final List<String> _failed = [];

void pass(String step, [String? detail]) {
  _passed.add(step);
  final msg = detail != null ? '  PASS [$step]: $detail' : '  PASS [$step]';
  print('\x1B[32m$msg\x1B[0m');
}

void fail(String step, Object err) {
  _failed.add(step);
  print('\x1B[31m  FAIL [$step]: $err\x1B[0m');
}

void heading(String s) => print('\n--- $s ---');

// ─── auth interceptor ─────────────────────────────────────────────────────────

/// A [ClientInterceptor] that appends a Bearer token to every call's metadata.
///
/// Phase-F Flutter note: this is a drop-in interceptor. Replace the static
/// token with a ValueNotifier<String> or Riverpod provider that updates after
/// a Login refresh.
class BearerInterceptor extends ClientInterceptor {
  BearerInterceptor(this._token);

  final String _token;

  @override
  ResponseFuture<R> interceptUnary<Q, R>(
    ClientMethod<Q, R> method,
    Q request,
    CallOptions options,
    ClientUnaryInvoker<Q, R> invoker,
  ) {
    return invoker(method, request, _withBearer(options));
  }

  @override
  ResponseStream<R> interceptStreaming<Q, R>(
    ClientMethod<Q, R> method,
    Stream<Q> requests,
    CallOptions options,
    ClientStreamingInvoker<Q, R> invoker,
  ) {
    return invoker(method, requests, _withBearer(options));
  }

  CallOptions _withBearer(CallOptions base) {
    return base.mergedWith(
      CallOptions(metadata: {'authorization': 'Bearer $_token'}),
    );
  }
}

// ─── main ─────────────────────────────────────────────────────────────────────

Future<void> main(List<String> argv) async {
  final parser = ArgParser()
    ..addOption('host',
        defaultsTo: '127.0.0.1', help: 'gRPC server host (IP or hostname)')
    ..addOption('port', defaultsTo: '50051', help: 'gRPC server port')
    ..addOption('token',
        mandatory: true, help: 'Auth token from rig banner (UUID)')
    ..addOption('cert',
        help: 'Path to server PEM cert (for self-signed trust)')
    ..addFlag('insecure',
        negatable: false,
        help: 'Skip TLS cert validation — accepts any certificate (dev only)')
    ..addOption('server-name',
        defaultsTo: 'localhost',
        help: 'TLS SNI hostname; must match a DNS SAN in the cert')
    ..addFlag('help', abbr: 'h', negatable: false, help: 'Show usage');

  final ArgResults args;
  try {
    args = parser.parse(argv);
  } on FormatException catch (e) {
    stderr.writeln('Error: ${e.message}\n${parser.usage}');
    exit(2);
  }

  if (args['help'] as bool) {
    print('Usage: dart run bin/muxr_client.dart --token <UUID> [options]\n');
    print(parser.usage);
    exit(0);
  }

  final host = args['host'] as String;
  final port = int.parse(args['port'] as String);
  final authToken = args['token'] as String;
  final certPath = args['cert'] as String?;
  final isInsecure = args['insecure'] as bool;
  final serverName = args['server-name'] as String;

  print('=== muxrd gRPC test client ===');
  print('  Target : $host:$port');
  print('  TLS SNI: $serverName');
  print(
    '  Auth   : token=${authToken.length > 8 ? "${authToken.substring(0, 8)}…" : authToken}'
    '  (${isInsecure ? "insecure/trust-all" : certPath != null ? "cert=$certPath" : "system-CAs"})',
  );

  // ── Build channel credentials ─────────────────────────────────────────────
  //
  // Three modes, matching the Phase-F Flutter use cases:
  //
  // 1. --cert <pem>   (recommended for the dev rig)
  //    Trust only the provided self-signed cert via a pinned SecurityContext.
  //    Phase-F: load the PEM from Flutter assets with rootBundle.load() and
  //    pass the bytes to SecurityContext()..setTrustedCertificatesBytes(bytes).
  //
  // 2. --insecure     (quick dev bypass — never production)
  //    onBadCertificate always returns true → accept any cert.
  //    Phase-F: equivalent to HttpOverrides with badCertificateCallback.
  //
  // 3. (neither)      trust system certificate store
  //    ChannelCredentials.secure() with no override — works against CA-signed certs.

  final ChannelCredentials creds;

  if (isInsecure) {
    print('\n[TLS] insecure mode — accepting any certificate');
    // Phase-F: allowBadCertificates is re-exported by package:grpc/grpc.dart.
    creds = ChannelCredentials.secure(
      authority: serverName,
      onBadCertificate: allowBadCertificates,
    );
  } else if (certPath != null) {
    print('\n[TLS] pinning cert: $certPath');
    final certFile = File(certPath);
    if (!certFile.existsSync()) {
      stderr.writeln('[TLS] cert file not found: $certPath');
      exit(1);
    }
    final pemBytes = certFile.readAsBytesSync();
    // ChannelCredentials.secure(certificates: bytes) creates a SecurityContext
    // with ONLY these bytes trusted (withTrustedRoots: false implicitly).
    // Perfect for pinning a self-signed cert without disabling validation.
    creds = ChannelCredentials.secure(
      certificates: pemBytes,
      authority: serverName,
    );
  } else {
    print('\n[TLS] using system trusted CAs');
    creds = ChannelCredentials.secure(authority: serverName);
  }

  // ── Open channel ──────────────────────────────────────────────────────────
  final channel = ClientChannel(
    host,
    port: port,
    options: ChannelOptions(credentials: creds),
  );

  var sessionToken = '';
  var firstSession = '';

  try {
    // ── Step 1: GetVersion (no auth) ─────────────────────────────────────────
    heading('GetVersion');
    final unauthStub = MuxrClient(channel);
    try {
      final version = await unauthStub.getVersion(Empty());
      print('  server_version : ${version.serverVersion}');
      print('  zellij_version : ${version.zellijVersion}');
      pass('GetVersion',
          'server=${version.serverVersion} zellij=${version.zellijVersion}');
    } catch (e) {
      fail('GetVersion', e);
    }

    // ── Step 2: Login ─────────────────────────────────────────────────────────
    heading('Login');
    try {
      final loginResp = await unauthStub.login(
        LoginRequest(authToken: authToken, rememberMe: false),
      );
      sessionToken = loginResp.sessionToken;
      print('  session_token: ${sessionToken.length > 8 ? "${sessionToken.substring(0, 8)}…" : sessionToken}');
      pass('Login', 'got session token');
    } catch (e) {
      fail('Login', e);
      // Cannot proceed without a session token.
      await _summarize();
      await channel.shutdown();
      exit(1);
    }

    // All subsequent RPCs use the session token via the interceptor.
    final authStub = MuxrClient(
      channel,
      interceptors: [BearerInterceptor(sessionToken)],
    );

    // ── Step 3: ListSessions ──────────────────────────────────────────────────
    heading('ListSessions');
    try {
      final sessionList = await authStub.listSessions(Empty());
      final sessions = sessionList.sessions;
      print('  ${sessions.length} session(s):');
      for (final s in sessions) {
        final age = '${s.ageSecs ~/ 60}m${s.ageSecs % 60}s';
        final flag = s.resurrectable ? ' [resurrectable]' : '';
        print('    - "${s.name}" age=$age$flag');
      }
      if (sessions.isNotEmpty) {
        firstSession = sessions.first.name;
        pass('ListSessions', '${sessions.length} session(s) — first="${sessions.first.name}"');
      } else {
        fail('ListSessions', 'no sessions (is the rig session running?)');
      }
    } catch (e) {
      fail('ListSessions', e);
    }

    // ── Step 4: GetLayout ─────────────────────────────────────────────────────
    heading('GetLayout');
    if (firstSession.isEmpty) {
      fail('GetLayout', 'skipped — no session from ListSessions');
    } else {
      try {
        final layout = await authStub.getLayout(
          SessionRef(session: firstSession),
        );
        final tabs = layout.tabs;
        final totalPanes = tabs.fold<int>(0, (s, t) => s + t.panes.length);
        print('  ${tabs.length} tab(s), $totalPanes pane(s) total:');

        String? sampleCwd;
        String? sampleCmd;

        for (final tab in tabs) {
          final active = tab.active ? ' [active]' : '';
          print(
            '    Tab #${tab.position} "${tab.name}" id=${tab.tabId}$active'
            ' — ${tab.panes.length} pane(s)',
          );
          for (final p in tab.panes) {
            final kind = p.isPlugin ? 'plugin' : 'terminal';
            final focus = p.isFocused ? ' [focused]' : '';
            print(
              '      Pane ${p.id} "${p.title}"$focus [$kind]'
              '${p.command.isNotEmpty ? " cmd=${p.command}" : ""}'
              '${p.cwd.isNotEmpty ? " cwd=${p.cwd}" : ""}',
            );
            if (sampleCwd == null && p.cwd.isNotEmpty) sampleCwd = p.cwd;
            if (sampleCmd == null && p.command.isNotEmpty) sampleCmd = p.command;
          }
        }

        if (tabs.isEmpty) {
          fail('GetLayout', 'no tabs in layout');
        } else {
          pass(
            'GetLayout',
            '${tabs.length} tab(s), $totalPanes pane(s)'
            '${sampleCwd != null ? ", sample cwd=$sampleCwd" : ""}'
            '${sampleCmd != null ? ", sample cmd=$sampleCmd" : ""}',
          );
        }
      } catch (e) {
        fail('GetLayout', e);
      }
    }

    // ── Step 5: AttachTerminal (bidi stream) ──────────────────────────────────
    heading('AttachTerminal');
    if (firstSession.isEmpty) {
      fail('AttachTerminal', 'skipped — no session from ListSessions');
    } else {
      await _runAttachTerminal(authStub, firstSession);
    }

    // ── Step 6: NewTab (optional action test) ─────────────────────────────────
    heading('NewTab (optional action test)');
    if (firstSession.isEmpty) {
      print('  Skipped — no session.');
    } else {
      await _runNewTabTest(authStub, firstSession);
    }
  } finally {
    await _summarize();
    await channel.shutdown();
    exit(_failed.isEmpty ? 0 : 1);
  }
}

// ─── AttachTerminal bidi stream ───────────────────────────────────────────────

Future<void> _runAttachTerminal(MuxrClient stub, String session) async {
  // grpc-dart bidi streaming pattern:
  //   1. Create a StreamController<ClientFrame> for outbound (client→server) messages.
  //   2. Pass its stream to the stub method — returns a ResponseStream<ServerFrame>.
  //   3. Add messages to the controller; listen to the ResponseStream for server frames.
  //   4. Close the controller when done.
  //
  // Phase-F Flutter: the StreamController lives for the session lifetime.
  // Keyboard/resize events are added to the sink; ServerFrames drive the
  // terminal widget's render buffer. The pattern is identical.

  final controller = StreamController<ClientFrame>();

  final responseStream = stub.attachTerminal(controller.stream);

  int renderFrames = 0;
  int controlEvents = 0;
  final completer = Completer<void>();

  late StreamSubscription<ServerFrame> sub;
  sub = responseStream.listen(
    (frame) {
      if (frame.hasRender()) {
        renderFrames++;
        final bytes = frame.render;
        print('  ServerFrame #$renderFrames render: ${bytes.length} bytes');
        if (renderFrames >= 3) {
          // Received enough frames — stop.
          sub.cancel();
          if (!completer.isCompleted) completer.complete();
        }
      } else if (frame.hasControl()) {
        controlEvents++;
        final c = frame.control;
        print('  ServerFrame control: kind=${c.kind} payload="${c.payload}"');
      }
    },
    onDone: () {
      print('  Stream closed by server.');
      if (!completer.isCompleted) completer.complete();
    },
    onError: (Object e) {
      if (!completer.isCompleted) completer.completeError(e);
    },
    cancelOnError: false,
  );

  // First ClientFrame: AttachReq identifies the session + terminal dimensions.
  controller.add(
    ClientFrame(attach: AttachReq(session: session, rows: 24, cols: 80)),
  );

  // Wait up to 8 s for render frames from the live zellij session.
  try {
    await completer.future.timeout(const Duration(seconds: 8));
  } on TimeoutException {
    print('  (timeout after 8s — $renderFrames render frame(s) received)');
  } catch (e) {
    fail('AttachTerminal', e);
    await controller.close();
    await sub.cancel();
    return;
  }

  await controller.close();
  await sub.cancel();

  if (renderFrames >= 1) {
    pass('AttachTerminal',
        '$renderFrames render frame(s), $controlEvents control event(s)');
  } else {
    fail('AttachTerminal',
        'no render frames in 8s (controlEvents=$controlEvents) — '
        'check that the rig session is actively running');
  }
}

// ─── NewTab optional test ─────────────────────────────────────────────────────

Future<void> _runNewTabTest(MuxrClient stub, String session) async {
  // Snapshot tab count before.
  final int beforeCount;
  try {
    final before = await stub.getLayout(SessionRef(session: session));
    beforeCount = before.tabs.length;
    print('  Tab count before NewTab: $beforeCount');
  } catch (e) {
    print('  Skipped (GetLayout before failed): $e');
    return;
  }

  // Create a new tab named 'dart-test'.
  try {
    final ack = await stub.newTab(
      NewTabReq(session: session, tabName: 'dart-test'),
    );
    if (!ack.ok) {
      print('  NewTab returned ok=false error="${ack.error}"');
      return;
    }
    print('  NewTab: ok=${ack.ok} info="${ack.info}"');
  } catch (e) {
    print('  NewTab RPC failed: $e');
    return;
  }

  // Brief pause for zellij to register the new tab.
  await Future<void>.delayed(const Duration(milliseconds: 600));

  // Verify +1 tab.
  try {
    final after = await stub.getLayout(SessionRef(session: session));
    final afterCount = after.tabs.length;
    print('  Tab count after NewTab: $afterCount');
    if (afterCount > beforeCount) {
      pass('NewTab', '+${afterCount - beforeCount} tab(s) confirmed ($beforeCount → $afterCount)');
    } else {
      // Not a hard FAIL — timing can affect zellij's layout response.
      print('  WARN [NewTab]: tab count $afterCount not greater than $beforeCount'
          ' (may be timing — check the rig session manually)');
    }
  } catch (e) {
    print('  GetLayout after NewTab failed: $e');
  }
}

// ─── final summary ────────────────────────────────────────────────────────────

Future<void> _summarize() async {
  final bar = '=' * 52;
  print('\n$bar');
  print('SUMMARY');
  print(bar);
  for (final s in _passed) {
    print('\x1B[32m  PASS: $s\x1B[0m');
  }
  for (final s in _failed) {
    print('\x1B[31m  FAIL: $s\x1B[0m');
  }
  print(bar);
  final total = _passed.length + _failed.length;
  if (_failed.isEmpty) {
    print('\x1B[32mAll $total step(s) PASSED\x1B[0m');
  } else {
    print('\x1B[31m${_failed.length}/$total step(s) FAILED\x1B[0m');
  }
}
