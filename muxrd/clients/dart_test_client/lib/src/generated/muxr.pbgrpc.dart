// This is a generated file - do not edit.
//
// Generated from muxr.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'muxr.pb.dart' as $0;

export 'muxr.pb.dart';

/// Muxr — the muxrd gRPC service.
///
/// Phase B coverage:
///   GetVersion     — plaintext; no auth required
///   Login          — stub (B3)
///   AttachTerminal — stub (B2)
///
/// Phase C coverage:
///   ListSessions   — enumerate live + resurrectable sessions (C1)
///   GetLayout      — typed tab/pane tree for a session (C1)
@$pb.GrpcServiceName('muxr.v1.Muxr')
class MuxrClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  MuxrClient(super.channel, {super.options, super.interceptors});

  /// Returns server + linked zellij version info. No auth.
  $grpc.ResponseFuture<$0.VersionInfo> getVersion(
    $0.Empty request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getVersion, request, options: options);
  }

  /// Exchange an auth_token for a session_token. No auth. (B3)
  $grpc.ResponseFuture<$0.LoginResponse> login(
    $0.LoginRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$login, request, options: options);
  }

  /// Bidirectional stream: mobile → server (ClientFrame), server → mobile
  /// (ServerFrame). Requires bearer auth. (B2)
  $grpc.ResponseStream<$0.ServerFrame> attachTerminal(
    $async.Stream<$0.ClientFrame> request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(_$attachTerminal, request, options: options);
  }

  /// List all live and resurrectable zellij sessions. Requires bearer auth. (C1)
  $grpc.ResponseFuture<$0.SessionList> listSessions(
    $0.Empty request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listSessions, request, options: options);
  }

  /// Return the typed tab/pane layout for a specific session. Requires bearer auth. (C1)
  $grpc.ResponseFuture<$0.Layout> getLayout(
    $0.SessionRef request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getLayout, request, options: options);
  }

  /// Write raw bytes to a specific pane. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> writeToPane(
    $0.WriteToPaneReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$writeToPane, request, options: options);
  }

  /// Focus a specific pane. Allowed for read-only tokens.
  $grpc.ResponseFuture<$0.ActionAck> focusPane(
    $0.PaneTarget request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$focusPane, request, options: options);
  }

  /// Close a specific pane. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> closePane(
    $0.PaneTarget request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$closePane, request, options: options);
  }

  /// Open a new pane; the new pane id is returned in ActionAck.info. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> newPane(
    $0.NewPaneReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$newPane, request, options: options);
  }

  /// Rename a specific pane. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> renamePane(
    $0.RenamePaneReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$renamePane, request, options: options);
  }

  /// Resize a specific pane. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> resizePane(
    $0.ResizePaneReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$resizePane, request, options: options);
  }

  /// Toggle a pane between floating and embedded. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> togglePaneFloating(
    $0.PaneTarget request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$togglePaneFloating, request, options: options);
  }

  /// Toggle fullscreen for a pane. MUTATING.
  /// Carries an optional floating-pane hint so the relay can skip a live IPC
  /// query on the hot path (Bug 2c — interaction latency).
  $grpc.ResponseFuture<$0.ActionAck> togglePaneFullscreen(
    $0.ToggleFullscreenReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$togglePaneFullscreen, request, options: options);
  }

  /// Open a new tab; the new tab id/name surface in ActionAck.info. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> newTab(
    $0.NewTabReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$newTab, request, options: options);
  }

  /// Close a tab by id. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> closeTab(
    $0.TabTarget request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$closeTab, request, options: options);
  }

  /// Switch focus to a tab by id. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> goToTab(
    $0.TabTarget request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$goToTab, request, options: options);
  }

  /// Rename a tab by id. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> renameTab(
    $0.RenameTabReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$renameTab, request, options: options);
  }

  /// Scroll a pane. Allowed for read-only tokens.
  $grpc.ResponseFuture<$0.ActionAck> scrollPane(
    $0.ScrollReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$scrollPane, request, options: options);
  }

  /// Rename the current session. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> renameSession(
    $0.RenameSessionReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$renameSession, request, options: options);
  }

  /// Kill a named session (removes it). MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> killSession(
    $0.SessionRef request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$killSession, request, options: options);
  }

  /// Create a new detached session by name. MUTATING.
  $grpc.ResponseFuture<$0.ActionAck> createSession(
    $0.CreateSessionReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createSession, request, options: options);
  }

  /// Create a new auth token; the secret is returned ONCE in TokenInfo.token.
  $grpc.ResponseFuture<$0.TokenInfo> createToken(
    $0.CreateTokenReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createToken, request, options: options);
  }

  /// List existing auth tokens (metadata only — never the secret).
  $grpc.ResponseFuture<$0.TokenList> listTokens(
    $0.Empty request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTokens, request, options: options);
  }

  /// Revoke an auth token by name.
  $grpc.ResponseFuture<$0.ActionAck> revokeToken(
    $0.RevokeTokenReq request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$revokeToken, request, options: options);
  }

  // method descriptors

  static final _$getVersion = $grpc.ClientMethod<$0.Empty, $0.VersionInfo>(
      '/muxr.v1.Muxr/GetVersion',
      ($0.Empty value) => value.writeToBuffer(),
      $0.VersionInfo.fromBuffer);
  static final _$login = $grpc.ClientMethod<$0.LoginRequest, $0.LoginResponse>(
      '/muxr.v1.Muxr/Login',
      ($0.LoginRequest value) => value.writeToBuffer(),
      $0.LoginResponse.fromBuffer);
  static final _$attachTerminal =
      $grpc.ClientMethod<$0.ClientFrame, $0.ServerFrame>(
          '/muxr.v1.Muxr/AttachTerminal',
          ($0.ClientFrame value) => value.writeToBuffer(),
          $0.ServerFrame.fromBuffer);
  static final _$listSessions = $grpc.ClientMethod<$0.Empty, $0.SessionList>(
      '/muxr.v1.Muxr/ListSessions',
      ($0.Empty value) => value.writeToBuffer(),
      $0.SessionList.fromBuffer);
  static final _$getLayout = $grpc.ClientMethod<$0.SessionRef, $0.Layout>(
      '/muxr.v1.Muxr/GetLayout',
      ($0.SessionRef value) => value.writeToBuffer(),
      $0.Layout.fromBuffer);
  static final _$writeToPane =
      $grpc.ClientMethod<$0.WriteToPaneReq, $0.ActionAck>(
          '/muxr.v1.Muxr/WriteToPane',
          ($0.WriteToPaneReq value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
  static final _$focusPane = $grpc.ClientMethod<$0.PaneTarget, $0.ActionAck>(
      '/muxr.v1.Muxr/FocusPane',
      ($0.PaneTarget value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$closePane = $grpc.ClientMethod<$0.PaneTarget, $0.ActionAck>(
      '/muxr.v1.Muxr/ClosePane',
      ($0.PaneTarget value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$newPane = $grpc.ClientMethod<$0.NewPaneReq, $0.ActionAck>(
      '/muxr.v1.Muxr/NewPane',
      ($0.NewPaneReq value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$renamePane =
      $grpc.ClientMethod<$0.RenamePaneReq, $0.ActionAck>(
          '/muxr.v1.Muxr/RenamePane',
          ($0.RenamePaneReq value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
  static final _$resizePane =
      $grpc.ClientMethod<$0.ResizePaneReq, $0.ActionAck>(
          '/muxr.v1.Muxr/ResizePane',
          ($0.ResizePaneReq value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
  static final _$togglePaneFloating =
      $grpc.ClientMethod<$0.PaneTarget, $0.ActionAck>(
          '/muxr.v1.Muxr/TogglePaneFloating',
          ($0.PaneTarget value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
  static final _$togglePaneFullscreen =
      $grpc.ClientMethod<$0.ToggleFullscreenReq, $0.ActionAck>(
          '/muxr.v1.Muxr/TogglePaneFullscreen',
          ($0.ToggleFullscreenReq value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
  static final _$newTab = $grpc.ClientMethod<$0.NewTabReq, $0.ActionAck>(
      '/muxr.v1.Muxr/NewTab',
      ($0.NewTabReq value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$closeTab = $grpc.ClientMethod<$0.TabTarget, $0.ActionAck>(
      '/muxr.v1.Muxr/CloseTab',
      ($0.TabTarget value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$goToTab = $grpc.ClientMethod<$0.TabTarget, $0.ActionAck>(
      '/muxr.v1.Muxr/GoToTab',
      ($0.TabTarget value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$renameTab = $grpc.ClientMethod<$0.RenameTabReq, $0.ActionAck>(
      '/muxr.v1.Muxr/RenameTab',
      ($0.RenameTabReq value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$scrollPane = $grpc.ClientMethod<$0.ScrollReq, $0.ActionAck>(
      '/muxr.v1.Muxr/ScrollPane',
      ($0.ScrollReq value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$renameSession =
      $grpc.ClientMethod<$0.RenameSessionReq, $0.ActionAck>(
          '/muxr.v1.Muxr/RenameSession',
          ($0.RenameSessionReq value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
  static final _$killSession = $grpc.ClientMethod<$0.SessionRef, $0.ActionAck>(
      '/muxr.v1.Muxr/KillSession',
      ($0.SessionRef value) => value.writeToBuffer(),
      $0.ActionAck.fromBuffer);
  static final _$createSession =
      $grpc.ClientMethod<$0.CreateSessionReq, $0.ActionAck>(
          '/muxr.v1.Muxr/CreateSession',
          ($0.CreateSessionReq value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
  static final _$createToken =
      $grpc.ClientMethod<$0.CreateTokenReq, $0.TokenInfo>(
          '/muxr.v1.Muxr/CreateToken',
          ($0.CreateTokenReq value) => value.writeToBuffer(),
          $0.TokenInfo.fromBuffer);
  static final _$listTokens = $grpc.ClientMethod<$0.Empty, $0.TokenList>(
      '/muxr.v1.Muxr/ListTokens',
      ($0.Empty value) => value.writeToBuffer(),
      $0.TokenList.fromBuffer);
  static final _$revokeToken =
      $grpc.ClientMethod<$0.RevokeTokenReq, $0.ActionAck>(
          '/muxr.v1.Muxr/RevokeToken',
          ($0.RevokeTokenReq value) => value.writeToBuffer(),
          $0.ActionAck.fromBuffer);
}

@$pb.GrpcServiceName('muxr.v1.Muxr')
abstract class MuxrServiceBase extends $grpc.Service {
  $core.String get $name => 'muxr.v1.Muxr';

  MuxrServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.VersionInfo>(
        'GetVersion',
        getVersion_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.VersionInfo value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.LoginRequest, $0.LoginResponse>(
        'Login',
        login_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.LoginRequest.fromBuffer(value),
        ($0.LoginResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ClientFrame, $0.ServerFrame>(
        'AttachTerminal',
        attachTerminal,
        true,
        true,
        ($core.List<$core.int> value) => $0.ClientFrame.fromBuffer(value),
        ($0.ServerFrame value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.SessionList>(
        'ListSessions',
        listSessions_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.SessionList value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SessionRef, $0.Layout>(
        'GetLayout',
        getLayout_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SessionRef.fromBuffer(value),
        ($0.Layout value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.WriteToPaneReq, $0.ActionAck>(
        'WriteToPane',
        writeToPane_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.WriteToPaneReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.PaneTarget, $0.ActionAck>(
        'FocusPane',
        focusPane_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PaneTarget.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.PaneTarget, $0.ActionAck>(
        'ClosePane',
        closePane_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PaneTarget.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.NewPaneReq, $0.ActionAck>(
        'NewPane',
        newPane_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.NewPaneReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RenamePaneReq, $0.ActionAck>(
        'RenamePane',
        renamePane_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RenamePaneReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ResizePaneReq, $0.ActionAck>(
        'ResizePane',
        resizePane_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ResizePaneReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.PaneTarget, $0.ActionAck>(
        'TogglePaneFloating',
        togglePaneFloating_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PaneTarget.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ToggleFullscreenReq, $0.ActionAck>(
        'TogglePaneFullscreen',
        togglePaneFullscreen_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ToggleFullscreenReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.NewTabReq, $0.ActionAck>(
        'NewTab',
        newTab_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.NewTabReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.TabTarget, $0.ActionAck>(
        'CloseTab',
        closeTab_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.TabTarget.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.TabTarget, $0.ActionAck>(
        'GoToTab',
        goToTab_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.TabTarget.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RenameTabReq, $0.ActionAck>(
        'RenameTab',
        renameTab_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RenameTabReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ScrollReq, $0.ActionAck>(
        'ScrollPane',
        scrollPane_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ScrollReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RenameSessionReq, $0.ActionAck>(
        'RenameSession',
        renameSession_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RenameSessionReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SessionRef, $0.ActionAck>(
        'KillSession',
        killSession_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SessionRef.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateSessionReq, $0.ActionAck>(
        'CreateSession',
        createSession_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateSessionReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateTokenReq, $0.TokenInfo>(
        'CreateToken',
        createToken_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateTokenReq.fromBuffer(value),
        ($0.TokenInfo value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.TokenList>(
        'ListTokens',
        listTokens_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.TokenList value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RevokeTokenReq, $0.ActionAck>(
        'RevokeToken',
        revokeToken_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RevokeTokenReq.fromBuffer(value),
        ($0.ActionAck value) => value.writeToBuffer()));
  }

  $async.Future<$0.VersionInfo> getVersion_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.Empty> $request) async {
    return getVersion($call, await $request);
  }

  $async.Future<$0.VersionInfo> getVersion(
      $grpc.ServiceCall call, $0.Empty request);

  $async.Future<$0.LoginResponse> login_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.LoginRequest> $request) async {
    return login($call, await $request);
  }

  $async.Future<$0.LoginResponse> login(
      $grpc.ServiceCall call, $0.LoginRequest request);

  $async.Stream<$0.ServerFrame> attachTerminal(
      $grpc.ServiceCall call, $async.Stream<$0.ClientFrame> request);

  $async.Future<$0.SessionList> listSessions_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.Empty> $request) async {
    return listSessions($call, await $request);
  }

  $async.Future<$0.SessionList> listSessions(
      $grpc.ServiceCall call, $0.Empty request);

  $async.Future<$0.Layout> getLayout_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.SessionRef> $request) async {
    return getLayout($call, await $request);
  }

  $async.Future<$0.Layout> getLayout(
      $grpc.ServiceCall call, $0.SessionRef request);

  $async.Future<$0.ActionAck> writeToPane_Pre($grpc.ServiceCall $call,
      $async.Future<$0.WriteToPaneReq> $request) async {
    return writeToPane($call, await $request);
  }

  $async.Future<$0.ActionAck> writeToPane(
      $grpc.ServiceCall call, $0.WriteToPaneReq request);

  $async.Future<$0.ActionAck> focusPane_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.PaneTarget> $request) async {
    return focusPane($call, await $request);
  }

  $async.Future<$0.ActionAck> focusPane(
      $grpc.ServiceCall call, $0.PaneTarget request);

  $async.Future<$0.ActionAck> closePane_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.PaneTarget> $request) async {
    return closePane($call, await $request);
  }

  $async.Future<$0.ActionAck> closePane(
      $grpc.ServiceCall call, $0.PaneTarget request);

  $async.Future<$0.ActionAck> newPane_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.NewPaneReq> $request) async {
    return newPane($call, await $request);
  }

  $async.Future<$0.ActionAck> newPane(
      $grpc.ServiceCall call, $0.NewPaneReq request);

  $async.Future<$0.ActionAck> renamePane_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.RenamePaneReq> $request) async {
    return renamePane($call, await $request);
  }

  $async.Future<$0.ActionAck> renamePane(
      $grpc.ServiceCall call, $0.RenamePaneReq request);

  $async.Future<$0.ActionAck> resizePane_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.ResizePaneReq> $request) async {
    return resizePane($call, await $request);
  }

  $async.Future<$0.ActionAck> resizePane(
      $grpc.ServiceCall call, $0.ResizePaneReq request);

  $async.Future<$0.ActionAck> togglePaneFloating_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.PaneTarget> $request) async {
    return togglePaneFloating($call, await $request);
  }

  $async.Future<$0.ActionAck> togglePaneFloating(
      $grpc.ServiceCall call, $0.PaneTarget request);

  $async.Future<$0.ActionAck> togglePaneFullscreen_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ToggleFullscreenReq> $request) async {
    return togglePaneFullscreen($call, await $request);
  }

  $async.Future<$0.ActionAck> togglePaneFullscreen(
      $grpc.ServiceCall call, $0.ToggleFullscreenReq request);

  $async.Future<$0.ActionAck> newTab_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.NewTabReq> $request) async {
    return newTab($call, await $request);
  }

  $async.Future<$0.ActionAck> newTab(
      $grpc.ServiceCall call, $0.NewTabReq request);

  $async.Future<$0.ActionAck> closeTab_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.TabTarget> $request) async {
    return closeTab($call, await $request);
  }

  $async.Future<$0.ActionAck> closeTab(
      $grpc.ServiceCall call, $0.TabTarget request);

  $async.Future<$0.ActionAck> goToTab_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.TabTarget> $request) async {
    return goToTab($call, await $request);
  }

  $async.Future<$0.ActionAck> goToTab(
      $grpc.ServiceCall call, $0.TabTarget request);

  $async.Future<$0.ActionAck> renameTab_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.RenameTabReq> $request) async {
    return renameTab($call, await $request);
  }

  $async.Future<$0.ActionAck> renameTab(
      $grpc.ServiceCall call, $0.RenameTabReq request);

  $async.Future<$0.ActionAck> scrollPane_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.ScrollReq> $request) async {
    return scrollPane($call, await $request);
  }

  $async.Future<$0.ActionAck> scrollPane(
      $grpc.ServiceCall call, $0.ScrollReq request);

  $async.Future<$0.ActionAck> renameSession_Pre($grpc.ServiceCall $call,
      $async.Future<$0.RenameSessionReq> $request) async {
    return renameSession($call, await $request);
  }

  $async.Future<$0.ActionAck> renameSession(
      $grpc.ServiceCall call, $0.RenameSessionReq request);

  $async.Future<$0.ActionAck> killSession_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.SessionRef> $request) async {
    return killSession($call, await $request);
  }

  $async.Future<$0.ActionAck> killSession(
      $grpc.ServiceCall call, $0.SessionRef request);

  $async.Future<$0.ActionAck> createSession_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateSessionReq> $request) async {
    return createSession($call, await $request);
  }

  $async.Future<$0.ActionAck> createSession(
      $grpc.ServiceCall call, $0.CreateSessionReq request);

  $async.Future<$0.TokenInfo> createToken_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateTokenReq> $request) async {
    return createToken($call, await $request);
  }

  $async.Future<$0.TokenInfo> createToken(
      $grpc.ServiceCall call, $0.CreateTokenReq request);

  $async.Future<$0.TokenList> listTokens_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.Empty> $request) async {
    return listTokens($call, await $request);
  }

  $async.Future<$0.TokenList> listTokens(
      $grpc.ServiceCall call, $0.Empty request);

  $async.Future<$0.ActionAck> revokeToken_Pre($grpc.ServiceCall $call,
      $async.Future<$0.RevokeTokenReq> $request) async {
    return revokeToken($call, await $request);
  }

  $async.Future<$0.ActionAck> revokeToken(
      $grpc.ServiceCall call, $0.RevokeTokenReq request);
}
