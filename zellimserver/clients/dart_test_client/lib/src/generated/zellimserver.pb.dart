// This is a generated file - do not edit.
//
// Generated from zellimserver.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'zellimserver.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'zellimserver.pbenum.dart';

class Empty extends $pb.GeneratedMessage {
  factory Empty() => create();

  Empty._();

  factory Empty.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Empty.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Empty',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Empty clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Empty copyWith(void Function(Empty) updates) =>
      super.copyWith((message) => updates(message as Empty)) as Empty;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Empty create() => Empty._();
  @$core.override
  Empty createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Empty getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Empty>(create);
  static Empty? _defaultInstance;
}

class VersionInfo extends $pb.GeneratedMessage {
  factory VersionInfo({
    $core.String? serverVersion,
    $core.String? zellijVersion,
  }) {
    final result = create();
    if (serverVersion != null) result.serverVersion = serverVersion;
    if (zellijVersion != null) result.zellijVersion = zellijVersion;
    return result;
  }

  VersionInfo._();

  factory VersionInfo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VersionInfo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VersionInfo',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serverVersion')
    ..aOS(2, _omitFieldNames ? '' : 'zellijVersion')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VersionInfo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VersionInfo copyWith(void Function(VersionInfo) updates) =>
      super.copyWith((message) => updates(message as VersionInfo))
          as VersionInfo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VersionInfo create() => VersionInfo._();
  @$core.override
  VersionInfo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VersionInfo getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VersionInfo>(create);
  static VersionInfo? _defaultInstance;

  /// Semantic version of zellimserver itself.
  @$pb.TagNumber(1)
  $core.String get serverVersion => $_getSZ(0);
  @$pb.TagNumber(1)
  set serverVersion($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServerVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearServerVersion() => $_clearField(1);

  /// Zellij version this server was compiled against.
  @$pb.TagNumber(2)
  $core.String get zellijVersion => $_getSZ(1);
  @$pb.TagNumber(2)
  set zellijVersion($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasZellijVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearZellijVersion() => $_clearField(2);
}

class LoginRequest extends $pb.GeneratedMessage {
  factory LoginRequest({
    $core.String? authToken,
    $core.bool? rememberMe,
  }) {
    final result = create();
    if (authToken != null) result.authToken = authToken;
    if (rememberMe != null) result.rememberMe = rememberMe;
    return result;
  }

  LoginRequest._();

  factory LoginRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory LoginRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LoginRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'authToken')
    ..aOB(2, _omitFieldNames ? '' : 'rememberMe')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LoginRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LoginRequest copyWith(void Function(LoginRequest) updates) =>
      super.copyWith((message) => updates(message as LoginRequest))
          as LoginRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static LoginRequest create() => LoginRequest._();
  @$core.override
  LoginRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static LoginRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LoginRequest>(create);
  static LoginRequest? _defaultInstance;

  /// The persistent auth token created via `zellij-utils web_authentication_tokens`.
  @$pb.TagNumber(1)
  $core.String get authToken => $_getSZ(0);
  @$pb.TagNumber(1)
  set authToken($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAuthToken() => $_has(0);
  @$pb.TagNumber(1)
  void clearAuthToken() => $_clearField(1);

  /// If true, session token lives 28 days; otherwise ~5 minutes.
  @$pb.TagNumber(2)
  $core.bool get rememberMe => $_getBF(1);
  @$pb.TagNumber(2)
  set rememberMe($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRememberMe() => $_has(1);
  @$pb.TagNumber(2)
  void clearRememberMe() => $_clearField(2);
}

class LoginResponse extends $pb.GeneratedMessage {
  factory LoginResponse({
    $core.String? sessionToken,
    $core.bool? isReadOnly,
  }) {
    final result = create();
    if (sessionToken != null) result.sessionToken = sessionToken;
    if (isReadOnly != null) result.isReadOnly = isReadOnly;
    return result;
  }

  LoginResponse._();

  factory LoginResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory LoginResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LoginResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'sessionToken')
    ..aOB(2, _omitFieldNames ? '' : 'isReadOnly')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LoginResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LoginResponse copyWith(void Function(LoginResponse) updates) =>
      super.copyWith((message) => updates(message as LoginResponse))
          as LoginResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static LoginResponse create() => LoginResponse._();
  @$core.override
  LoginResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static LoginResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LoginResponse>(create);
  static LoginResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get sessionToken => $_getSZ(0);
  @$pb.TagNumber(1)
  set sessionToken($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSessionToken() => $_has(0);
  @$pb.TagNumber(1)
  void clearSessionToken() => $_clearField(1);

  /// True if the authenticating token is read-only (mutating RPCs are gated
  /// server-side; surfaced so the client can disable mutating controls). (F)
  @$pb.TagNumber(2)
  $core.bool get isReadOnly => $_getBF(1);
  @$pb.TagNumber(2)
  set isReadOnly($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIsReadOnly() => $_has(1);
  @$pb.TagNumber(2)
  void clearIsReadOnly() => $_clearField(2);
}

/// Metadata for one auth token. `token` is populated ONLY in a CreateToken
/// response (the secret is never returned by ListTokens).
class TokenInfo extends $pb.GeneratedMessage {
  factory TokenInfo({
    $core.String? name,
    $core.String? token,
    $core.bool? readOnly,
    $core.String? createdAt,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (token != null) result.token = token;
    if (readOnly != null) result.readOnly = readOnly;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  TokenInfo._();

  factory TokenInfo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TokenInfo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TokenInfo',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'token')
    ..aOB(3, _omitFieldNames ? '' : 'readOnly')
    ..aOS(4, _omitFieldNames ? '' : 'createdAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenInfo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenInfo copyWith(void Function(TokenInfo) updates) =>
      super.copyWith((message) => updates(message as TokenInfo)) as TokenInfo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TokenInfo create() => TokenInfo._();
  @$core.override
  TokenInfo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TokenInfo getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TokenInfo>(create);
  static TokenInfo? _defaultInstance;

  /// Human-readable token name.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// The secret token value — populated only on create, empty on list.
  @$pb.TagNumber(2)
  $core.String get token => $_getSZ(1);
  @$pb.TagNumber(2)
  set token($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasToken() => $_has(1);
  @$pb.TagNumber(2)
  void clearToken() => $_clearField(2);

  /// True if this is a read-only token.
  @$pb.TagNumber(3)
  $core.bool get readOnly => $_getBF(2);
  @$pb.TagNumber(3)
  set readOnly($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasReadOnly() => $_has(2);
  @$pb.TagNumber(3)
  void clearReadOnly() => $_clearField(3);

  /// Creation timestamp as stored by zellij (SQLite DATETIME string), "" if unknown.
  @$pb.TagNumber(4)
  $core.String get createdAt => $_getSZ(3);
  @$pb.TagNumber(4)
  set createdAt($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCreatedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreatedAt() => $_clearField(4);
}

/// List of tokens returned by ListTokens (metadata only).
class TokenList extends $pb.GeneratedMessage {
  factory TokenList({
    $core.Iterable<TokenInfo>? tokens,
  }) {
    final result = create();
    if (tokens != null) result.tokens.addAll(tokens);
    return result;
  }

  TokenList._();

  factory TokenList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TokenList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TokenList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..pPM<TokenInfo>(1, _omitFieldNames ? '' : 'tokens',
        subBuilder: TokenInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenList copyWith(void Function(TokenList) updates) =>
      super.copyWith((message) => updates(message as TokenList)) as TokenList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TokenList create() => TokenList._();
  @$core.override
  TokenList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TokenList getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TokenList>(create);
  static TokenList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<TokenInfo> get tokens => $_getList(0);
}

/// CreateToken request.
class CreateTokenReq extends $pb.GeneratedMessage {
  factory CreateTokenReq({
    $core.String? name,
    $core.bool? readOnly,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (readOnly != null) result.readOnly = readOnly;
    return result;
  }

  CreateTokenReq._();

  factory CreateTokenReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateTokenReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateTokenReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOB(2, _omitFieldNames ? '' : 'readOnly')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateTokenReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateTokenReq copyWith(void Function(CreateTokenReq) updates) =>
      super.copyWith((message) => updates(message as CreateTokenReq))
          as CreateTokenReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateTokenReq create() => CreateTokenReq._();
  @$core.override
  CreateTokenReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateTokenReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateTokenReq>(create);
  static CreateTokenReq? _defaultInstance;

  /// Name for the new token.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// If true, create a read-only token.
  @$pb.TagNumber(2)
  $core.bool get readOnly => $_getBF(1);
  @$pb.TagNumber(2)
  set readOnly($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasReadOnly() => $_has(1);
  @$pb.TagNumber(2)
  void clearReadOnly() => $_clearField(2);
}

/// RevokeToken request.
class RevokeTokenReq extends $pb.GeneratedMessage {
  factory RevokeTokenReq({
    $core.String? name,
  }) {
    final result = create();
    if (name != null) result.name = name;
    return result;
  }

  RevokeTokenReq._();

  factory RevokeTokenReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RevokeTokenReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RevokeTokenReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeTokenReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeTokenReq copyWith(void Function(RevokeTokenReq) updates) =>
      super.copyWith((message) => updates(message as RevokeTokenReq))
          as RevokeTokenReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RevokeTokenReq create() => RevokeTokenReq._();
  @$core.override
  RevokeTokenReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RevokeTokenReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RevokeTokenReq>(create);
  static RevokeTokenReq? _defaultInstance;

  /// Name of the token to revoke.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

enum ClientFrame_Kind { attach, input, resize, notSet }

/// Client → Server frame on the AttachTerminal stream.
class ClientFrame extends $pb.GeneratedMessage {
  factory ClientFrame({
    AttachReq? attach,
    $core.List<$core.int>? input,
    Resize? resize,
  }) {
    final result = create();
    if (attach != null) result.attach = attach;
    if (input != null) result.input = input;
    if (resize != null) result.resize = resize;
    return result;
  }

  ClientFrame._();

  factory ClientFrame.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientFrame.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ClientFrame_Kind> _ClientFrame_KindByTag = {
    1: ClientFrame_Kind.attach,
    2: ClientFrame_Kind.input,
    3: ClientFrame_Kind.resize,
    0: ClientFrame_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientFrame',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..aOM<AttachReq>(1, _omitFieldNames ? '' : 'attach',
        subBuilder: AttachReq.create)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'input', $pb.PbFieldType.OY)
    ..aOM<Resize>(3, _omitFieldNames ? '' : 'resize', subBuilder: Resize.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientFrame clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientFrame copyWith(void Function(ClientFrame) updates) =>
      super.copyWith((message) => updates(message as ClientFrame))
          as ClientFrame;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientFrame create() => ClientFrame._();
  @$core.override
  ClientFrame createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientFrame getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientFrame>(create);
  static ClientFrame? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  ClientFrame_Kind whichKind() => _ClientFrame_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearKind() => $_clearField($_whichOneof(0));

  /// First frame on any new stream: identify the session + initial dimensions.
  @$pb.TagNumber(1)
  AttachReq get attach => $_getN(0);
  @$pb.TagNumber(1)
  set attach(AttachReq value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAttach() => $_has(0);
  @$pb.TagNumber(1)
  void clearAttach() => $_clearField(1);
  @$pb.TagNumber(1)
  AttachReq ensureAttach() => $_ensure(0);

  /// Raw input bytes (key presses etc.) forwarded to the focused pane.
  @$pb.TagNumber(2)
  $core.List<$core.int> get input => $_getN(1);
  @$pb.TagNumber(2)
  set input($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInput() => $_has(1);
  @$pb.TagNumber(2)
  void clearInput() => $_clearField(2);

  /// Terminal resize event.
  @$pb.TagNumber(3)
  Resize get resize => $_getN(2);
  @$pb.TagNumber(3)
  set resize(Resize value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasResize() => $_has(2);
  @$pb.TagNumber(3)
  void clearResize() => $_clearField(3);
  @$pb.TagNumber(3)
  Resize ensureResize() => $_ensure(2);
}

/// Identifies the target session and initial terminal dimensions.
class AttachReq extends $pb.GeneratedMessage {
  factory AttachReq({
    $core.String? session,
    $core.int? rows,
    $core.int? cols,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (rows != null) result.rows = rows;
    if (cols != null) result.cols = cols;
    return result;
  }

  AttachReq._();

  factory AttachReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AttachReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AttachReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..aI(2, _omitFieldNames ? '' : 'rows', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'cols', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AttachReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AttachReq copyWith(void Function(AttachReq) updates) =>
      super.copyWith((message) => updates(message as AttachReq)) as AttachReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AttachReq create() => AttachReq._();
  @$core.override
  AttachReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AttachReq getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AttachReq>(create);
  static AttachReq? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get rows => $_getIZ(1);
  @$pb.TagNumber(2)
  set rows($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRows() => $_has(1);
  @$pb.TagNumber(2)
  void clearRows() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get cols => $_getIZ(2);
  @$pb.TagNumber(3)
  set cols($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCols() => $_has(2);
  @$pb.TagNumber(3)
  void clearCols() => $_clearField(3);
}

/// Terminal resize.
class Resize extends $pb.GeneratedMessage {
  factory Resize({
    $core.int? rows,
    $core.int? cols,
  }) {
    final result = create();
    if (rows != null) result.rows = rows;
    if (cols != null) result.cols = cols;
    return result;
  }

  Resize._();

  factory Resize.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Resize.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Resize',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'rows', fieldType: $pb.PbFieldType.OU3)
    ..aI(2, _omitFieldNames ? '' : 'cols', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Resize clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Resize copyWith(void Function(Resize) updates) =>
      super.copyWith((message) => updates(message as Resize)) as Resize;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Resize create() => Resize._();
  @$core.override
  Resize createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Resize getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Resize>(create);
  static Resize? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get rows => $_getIZ(0);
  @$pb.TagNumber(1)
  set rows($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRows() => $_has(0);
  @$pb.TagNumber(1)
  void clearRows() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get cols => $_getIZ(1);
  @$pb.TagNumber(2)
  set cols($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCols() => $_has(1);
  @$pb.TagNumber(2)
  void clearCols() => $_clearField(2);
}

enum ServerFrame_Kind { render, control, notSet }

/// Server → Client frame on the AttachTerminal stream.
class ServerFrame extends $pb.GeneratedMessage {
  factory ServerFrame({
    $core.List<$core.int>? render,
    ControlEvent? control,
  }) {
    final result = create();
    if (render != null) result.render = render;
    if (control != null) result.control = control;
    return result;
  }

  ServerFrame._();

  factory ServerFrame.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ServerFrame.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ServerFrame_Kind> _ServerFrame_KindByTag = {
    1: ServerFrame_Kind.render,
    2: ServerFrame_Kind.control,
    0: ServerFrame_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ServerFrame',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'render', $pb.PbFieldType.OY)
    ..aOM<ControlEvent>(2, _omitFieldNames ? '' : 'control',
        subBuilder: ControlEvent.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServerFrame clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServerFrame copyWith(void Function(ServerFrame) updates) =>
      super.copyWith((message) => updates(message as ServerFrame))
          as ServerFrame;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ServerFrame create() => ServerFrame._();
  @$core.override
  ServerFrame createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ServerFrame getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ServerFrame>(create);
  static ServerFrame? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  ServerFrame_Kind whichKind() => _ServerFrame_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearKind() => $_clearField($_whichOneof(0));

  /// Raw ANSI render bytes from the zellij session.
  @$pb.TagNumber(1)
  $core.List<$core.int> get render => $_getN(0);
  @$pb.TagNumber(1)
  set render($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRender() => $_has(0);
  @$pb.TagNumber(1)
  void clearRender() => $_clearField(1);

  /// Out-of-band control event (Phase C+).
  @$pb.TagNumber(2)
  ControlEvent get control => $_getN(1);
  @$pb.TagNumber(2)
  set control(ControlEvent value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasControl() => $_has(1);
  @$pb.TagNumber(2)
  void clearControl() => $_clearField(2);
  @$pb.TagNumber(2)
  ControlEvent ensureControl() => $_ensure(1);
}

/// Out-of-band control event sent from server to client (Phase C+).
class ControlEvent extends $pb.GeneratedMessage {
  factory ControlEvent({
    $core.String? kind,
    $core.String? payload,
  }) {
    final result = create();
    if (kind != null) result.kind = kind;
    if (payload != null) result.payload = payload;
    return result;
  }

  ControlEvent._();

  factory ControlEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ControlEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ControlEvent',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'kind')
    ..aOS(2, _omitFieldNames ? '' : 'payload')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ControlEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ControlEvent copyWith(void Function(ControlEvent) updates) =>
      super.copyWith((message) => updates(message as ControlEvent))
          as ControlEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ControlEvent create() => ControlEvent._();
  @$core.override
  ControlEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ControlEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ControlEvent>(create);
  static ControlEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get kind => $_getSZ(0);
  @$pb.TagNumber(1)
  set kind($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasKind() => $_has(0);
  @$pb.TagNumber(1)
  void clearKind() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get payload => $_getSZ(1);
  @$pb.TagNumber(2)
  set payload($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPayload() => $_has(1);
  @$pb.TagNumber(2)
  void clearPayload() => $_clearField(2);
}

/// Reference to a named zellij session (used in GetLayout).
class SessionRef extends $pb.GeneratedMessage {
  factory SessionRef({
    $core.String? session,
    $core.String? connectionId,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (connectionId != null) result.connectionId = connectionId;
    return result;
  }

  SessionRef._();

  factory SessionRef.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SessionRef.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SessionRef',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..aOS(2, _omitFieldNames ? '' : 'connectionId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionRef clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionRef copyWith(void Function(SessionRef) updates) =>
      super.copyWith((message) => updates(message as SessionRef)) as SessionRef;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SessionRef create() => SessionRef._();
  @$core.override
  SessionRef createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SessionRef getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SessionRef>(create);
  static SessionRef? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  /// optional; echoed from AttachTerminal's minted id to route per-connection (empty = legacy/fallback routing)
  @$pb.TagNumber(2)
  $core.String get connectionId => $_getSZ(1);
  @$pb.TagNumber(2)
  set connectionId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasConnectionId() => $_has(1);
  @$pb.TagNumber(2)
  void clearConnectionId() => $_clearField(2);
}

/// List of sessions returned by ListSessions.
class SessionList extends $pb.GeneratedMessage {
  factory SessionList({
    $core.Iterable<SessionInfo>? sessions,
  }) {
    final result = create();
    if (sessions != null) result.sessions.addAll(sessions);
    return result;
  }

  SessionList._();

  factory SessionList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SessionList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SessionList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..pPM<SessionInfo>(1, _omitFieldNames ? '' : 'sessions',
        subBuilder: SessionInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionList copyWith(void Function(SessionList) updates) =>
      super.copyWith((message) => updates(message as SessionList))
          as SessionList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SessionList create() => SessionList._();
  @$core.override
  SessionList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SessionList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SessionList>(create);
  static SessionList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SessionInfo> get sessions => $_getList(0);
}

/// Metadata for a single zellij session.
class SessionInfo extends $pb.GeneratedMessage {
  factory SessionInfo({
    $core.String? name,
    $fixnum.Int64? ageSecs,
    $core.bool? resurrectable,
    $core.int? tabCount,
    $core.int? paneCount,
    $core.bool? hasBell,
    $core.bool? isCurrent,
    $core.int? connectedClients,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (ageSecs != null) result.ageSecs = ageSecs;
    if (resurrectable != null) result.resurrectable = resurrectable;
    if (tabCount != null) result.tabCount = tabCount;
    if (paneCount != null) result.paneCount = paneCount;
    if (hasBell != null) result.hasBell = hasBell;
    if (isCurrent != null) result.isCurrent = isCurrent;
    if (connectedClients != null) result.connectedClients = connectedClients;
    return result;
  }

  SessionInfo._();

  factory SessionInfo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SessionInfo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SessionInfo',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'ageSecs', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(3, _omitFieldNames ? '' : 'resurrectable')
    ..aI(4, _omitFieldNames ? '' : 'tabCount', fieldType: $pb.PbFieldType.OU3)
    ..aI(5, _omitFieldNames ? '' : 'paneCount', fieldType: $pb.PbFieldType.OU3)
    ..aOB(6, _omitFieldNames ? '' : 'hasBell')
    ..aOB(7, _omitFieldNames ? '' : 'isCurrent')
    ..aI(8, _omitFieldNames ? '' : 'connectedClients',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionInfo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionInfo copyWith(void Function(SessionInfo) updates) =>
      super.copyWith((message) => updates(message as SessionInfo))
          as SessionInfo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SessionInfo create() => SessionInfo._();
  @$core.override
  SessionInfo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SessionInfo getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SessionInfo>(create);
  static SessionInfo? _defaultInstance;

  /// Session name.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// Approximate age of the session in seconds (from socket mtime).
  @$pb.TagNumber(2)
  $fixnum.Int64 get ageSecs => $_getI64(1);
  @$pb.TagNumber(2)
  set ageSecs($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAgeSecs() => $_has(1);
  @$pb.TagNumber(2)
  void clearAgeSecs() => $_clearField(2);

  /// True if this is a resurrectable (dead but recoverable) session.
  @$pb.TagNumber(3)
  $core.bool get resurrectable => $_getBF(2);
  @$pb.TagNumber(3)
  set resurrectable($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasResurrectable() => $_has(2);
  @$pb.TagNumber(3)
  void clearResurrectable() => $_clearField(3);

  /// ─── Phase F enrichment (live sessions only; 0/false for resurrectable) ────
  /// Number of tabs in the session (from a layout query).
  @$pb.TagNumber(4)
  $core.int get tabCount => $_getIZ(3);
  @$pb.TagNumber(4)
  set tabCount($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTabCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearTabCount() => $_clearField(4);

  /// Total number of panes across all tabs.
  @$pb.TagNumber(5)
  $core.int get paneCount => $_getIZ(4);
  @$pb.TagNumber(5)
  set paneCount($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasPaneCount() => $_has(4);
  @$pb.TagNumber(5)
  void clearPaneCount() => $_clearField(5);

  /// True if any tab has an active bell notification.
  @$pb.TagNumber(6)
  $core.bool get hasBell => $_getBF(5);
  @$pb.TagNumber(6)
  set hasBell($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasHasBell() => $_has(5);
  @$pb.TagNumber(6)
  void clearHasBell() => $_clearField(6);

  /// True if this is the session the requesting token is attached to.
  @$pb.TagNumber(7)
  $core.bool get isCurrent => $_getBF(6);
  @$pb.TagNumber(7)
  set isCurrent($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasIsCurrent() => $_has(6);
  @$pb.TagNumber(7)
  void clearIsCurrent() => $_clearField(7);

  /// Count of mobile clients currently attached to this session THROUGH
  /// zellimserver (active AttachTerminal relays), not zellij-level clients.
  @$pb.TagNumber(8)
  $core.int get connectedClients => $_getIZ(7);
  @$pb.TagNumber(8)
  set connectedClients($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasConnectedClients() => $_has(7);
  @$pb.TagNumber(8)
  void clearConnectedClients() => $_clearField(8);
}

/// Full tab/pane layout for a session (returned by GetLayout).
class Layout extends $pb.GeneratedMessage {
  factory Layout({
    $core.Iterable<TabMsg>? tabs,
  }) {
    final result = create();
    if (tabs != null) result.tabs.addAll(tabs);
    return result;
  }

  Layout._();

  factory Layout.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Layout.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Layout',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..pPM<TabMsg>(1, _omitFieldNames ? '' : 'tabs', subBuilder: TabMsg.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Layout clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Layout copyWith(void Function(Layout) updates) =>
      super.copyWith((message) => updates(message as Layout)) as Layout;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Layout create() => Layout._();
  @$core.override
  Layout createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Layout getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Layout>(create);
  static Layout? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<TabMsg> get tabs => $_getList(0);
}

/// One tab in the layout.
class TabMsg extends $pb.GeneratedMessage {
  factory TabMsg({
    $core.int? position,
    $core.String? name,
    $core.bool? active,
    $core.bool? hasBell,
    $core.int? panesToHide,
    $core.int? tabId,
    $core.Iterable<PaneMsg>? panes,
    $core.bool? fullscreenActive,
    $core.bool? floatingPanesVisible,
  }) {
    final result = create();
    if (position != null) result.position = position;
    if (name != null) result.name = name;
    if (active != null) result.active = active;
    if (hasBell != null) result.hasBell = hasBell;
    if (panesToHide != null) result.panesToHide = panesToHide;
    if (tabId != null) result.tabId = tabId;
    if (panes != null) result.panes.addAll(panes);
    if (fullscreenActive != null) result.fullscreenActive = fullscreenActive;
    if (floatingPanesVisible != null)
      result.floatingPanesVisible = floatingPanesVisible;
    return result;
  }

  TabMsg._();

  factory TabMsg.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TabMsg.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TabMsg',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'position', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOB(3, _omitFieldNames ? '' : 'active')
    ..aOB(4, _omitFieldNames ? '' : 'hasBell')
    ..aI(5, _omitFieldNames ? '' : 'panesToHide',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(6, _omitFieldNames ? '' : 'tabId', fieldType: $pb.PbFieldType.OU3)
    ..pPM<PaneMsg>(7, _omitFieldNames ? '' : 'panes',
        subBuilder: PaneMsg.create)
    ..aOB(8, _omitFieldNames ? '' : 'fullscreenActive')
    ..aOB(9, _omitFieldNames ? '' : 'floatingPanesVisible')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TabMsg clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TabMsg copyWith(void Function(TabMsg) updates) =>
      super.copyWith((message) => updates(message as TabMsg)) as TabMsg;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TabMsg create() => TabMsg._();
  @$core.override
  TabMsg createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TabMsg getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TabMsg>(create);
  static TabMsg? _defaultInstance;

  /// 0-indexed tab position.
  @$pb.TagNumber(1)
  $core.int get position => $_getIZ(0);
  @$pb.TagNumber(1)
  set position($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPosition() => $_has(0);
  @$pb.TagNumber(1)
  void clearPosition() => $_clearField(1);

  /// Display name of the tab.
  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  /// Whether this tab is currently active/focused.
  @$pb.TagNumber(3)
  $core.bool get active => $_getBF(2);
  @$pb.TagNumber(3)
  set active($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasActive() => $_has(2);
  @$pb.TagNumber(3)
  void clearActive() => $_clearField(3);

  /// Whether this tab has an active (persistent) bell notification.
  @$pb.TagNumber(4)
  $core.bool get hasBell => $_getBF(3);
  @$pb.TagNumber(4)
  set hasBell($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasHasBell() => $_has(3);
  @$pb.TagNumber(4)
  void clearHasBell() => $_clearField(4);

  /// Number of suppressed (hidden) panes.
  @$pb.TagNumber(5)
  $core.int get panesToHide => $_getIZ(4);
  @$pb.TagNumber(5)
  set panesToHide($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasPanesToHide() => $_has(4);
  @$pb.TagNumber(5)
  void clearPanesToHide() => $_clearField(5);

  /// Stable identifier for this tab (from TabInfo.tab_id).
  @$pb.TagNumber(6)
  $core.int get tabId => $_getIZ(5);
  @$pb.TagNumber(6)
  set tabId($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTabId() => $_has(5);
  @$pb.TagNumber(6)
  void clearTabId() => $_clearField(6);

  /// Panes in this tab.
  @$pb.TagNumber(7)
  $pb.PbList<PaneMsg> get panes => $_getList(6);

  /// Whether a pane in this tab is currently fullscreened (TabInfo.is_fullscreen_active).
  @$pb.TagNumber(8)
  $core.bool get fullscreenActive => $_getBF(7);
  @$pb.TagNumber(8)
  set fullscreenActive($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasFullscreenActive() => $_has(7);
  @$pb.TagNumber(8)
  void clearFullscreenActive() => $_clearField(8);

  /// Whether floating panes are visible in this tab (TabInfo.are_floating_panes_visible).
  @$pb.TagNumber(9)
  $core.bool get floatingPanesVisible => $_getBF(8);
  @$pb.TagNumber(9)
  set floatingPanesVisible($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasFloatingPanesVisible() => $_has(8);
  @$pb.TagNumber(9)
  void clearFloatingPanesVisible() => $_clearField(9);
}

/// One pane inside a tab.
class PaneMsg extends $pb.GeneratedMessage {
  factory PaneMsg({
    $core.int? id,
    $core.String? title,
    $core.bool? isFocused,
    $core.bool? isFloating,
    $core.bool? exited,
    $core.String? command,
    $core.String? cwd,
    $core.int? x,
    $core.int? y,
    $core.int? rows,
    $core.int? cols,
    $core.bool? isPlugin,
    $core.bool? isFullscreen,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (title != null) result.title = title;
    if (isFocused != null) result.isFocused = isFocused;
    if (isFloating != null) result.isFloating = isFloating;
    if (exited != null) result.exited = exited;
    if (command != null) result.command = command;
    if (cwd != null) result.cwd = cwd;
    if (x != null) result.x = x;
    if (y != null) result.y = y;
    if (rows != null) result.rows = rows;
    if (cols != null) result.cols = cols;
    if (isPlugin != null) result.isPlugin = isPlugin;
    if (isFullscreen != null) result.isFullscreen = isFullscreen;
    return result;
  }

  PaneMsg._();

  factory PaneMsg.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PaneMsg.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PaneMsg',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'title')
    ..aOB(3, _omitFieldNames ? '' : 'isFocused')
    ..aOB(4, _omitFieldNames ? '' : 'isFloating')
    ..aOB(5, _omitFieldNames ? '' : 'exited')
    ..aOS(6, _omitFieldNames ? '' : 'command')
    ..aOS(7, _omitFieldNames ? '' : 'cwd')
    ..aI(8, _omitFieldNames ? '' : 'x', fieldType: $pb.PbFieldType.OU3)
    ..aI(9, _omitFieldNames ? '' : 'y', fieldType: $pb.PbFieldType.OU3)
    ..aI(10, _omitFieldNames ? '' : 'rows', fieldType: $pb.PbFieldType.OU3)
    ..aI(11, _omitFieldNames ? '' : 'cols', fieldType: $pb.PbFieldType.OU3)
    ..aOB(12, _omitFieldNames ? '' : 'isPlugin')
    ..aOB(13, _omitFieldNames ? '' : 'isFullscreen')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaneMsg clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaneMsg copyWith(void Function(PaneMsg) updates) =>
      super.copyWith((message) => updates(message as PaneMsg)) as PaneMsg;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PaneMsg create() => PaneMsg._();
  @$core.override
  PaneMsg createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PaneMsg getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PaneMsg>(create);
  static PaneMsg? _defaultInstance;

  /// Pane id (unique within pane kind — terminal or plugin).
  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// Display title of the pane.
  @$pb.TagNumber(2)
  $core.String get title => $_getSZ(1);
  @$pb.TagNumber(2)
  set title($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTitle() => $_has(1);
  @$pb.TagNumber(2)
  void clearTitle() => $_clearField(2);

  /// Whether this pane is focused within its layer (tiled/floating).
  @$pb.TagNumber(3)
  $core.bool get isFocused => $_getBF(2);
  @$pb.TagNumber(3)
  set isFocused($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIsFocused() => $_has(2);
  @$pb.TagNumber(3)
  void clearIsFocused() => $_clearField(3);

  /// Whether the pane is floating (vs tiled).
  @$pb.TagNumber(4)
  $core.bool get isFloating => $_getBF(3);
  @$pb.TagNumber(4)
  set isFloating($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasIsFloating() => $_has(3);
  @$pb.TagNumber(4)
  void clearIsFloating() => $_clearField(4);

  /// Whether the pane has exited (command panes only).
  @$pb.TagNumber(5)
  $core.bool get exited => $_getBF(4);
  @$pb.TagNumber(5)
  set exited($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasExited() => $_has(4);
  @$pb.TagNumber(5)
  void clearExited() => $_clearField(5);

  /// Command running in the pane (if it's a command/terminal pane), else empty.
  @$pb.TagNumber(6)
  $core.String get command => $_getSZ(5);
  @$pb.TagNumber(6)
  set command($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCommand() => $_has(5);
  @$pb.TagNumber(6)
  void clearCommand() => $_clearField(6);

  /// Current working directory of the pane, if available.
  @$pb.TagNumber(7)
  $core.String get cwd => $_getSZ(6);
  @$pb.TagNumber(7)
  set cwd($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCwd() => $_has(6);
  @$pb.TagNumber(7)
  void clearCwd() => $_clearField(7);

  /// Geometry: x position (pane_x from PaneInfo).
  @$pb.TagNumber(8)
  $core.int get x => $_getIZ(7);
  @$pb.TagNumber(8)
  set x($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasX() => $_has(7);
  @$pb.TagNumber(8)
  void clearX() => $_clearField(8);

  /// Geometry: y position (pane_y from PaneInfo).
  @$pb.TagNumber(9)
  $core.int get y => $_getIZ(8);
  @$pb.TagNumber(9)
  set y($core.int value) => $_setUnsignedInt32(8, value);
  @$pb.TagNumber(9)
  $core.bool hasY() => $_has(8);
  @$pb.TagNumber(9)
  void clearY() => $_clearField(9);

  /// Geometry: rows.
  @$pb.TagNumber(10)
  $core.int get rows => $_getIZ(9);
  @$pb.TagNumber(10)
  set rows($core.int value) => $_setUnsignedInt32(9, value);
  @$pb.TagNumber(10)
  $core.bool hasRows() => $_has(9);
  @$pb.TagNumber(10)
  void clearRows() => $_clearField(10);

  /// Geometry: columns.
  @$pb.TagNumber(11)
  $core.int get cols => $_getIZ(10);
  @$pb.TagNumber(11)
  set cols($core.int value) => $_setUnsignedInt32(10, value);
  @$pb.TagNumber(11)
  $core.bool hasCols() => $_has(10);
  @$pb.TagNumber(11)
  void clearCols() => $_clearField(11);

  /// Whether this is a plugin pane (vs terminal).
  @$pb.TagNumber(12)
  $core.bool get isPlugin => $_getBF(11);
  @$pb.TagNumber(12)
  set isPlugin($core.bool value) => $_setBool(11, value);
  @$pb.TagNumber(12)
  $core.bool hasIsPlugin() => $_has(11);
  @$pb.TagNumber(12)
  void clearIsPlugin() => $_clearField(12);

  /// Whether this pane is currently fullscreened (PaneInfo.is_fullscreen).
  @$pb.TagNumber(13)
  $core.bool get isFullscreen => $_getBF(12);
  @$pb.TagNumber(13)
  set isFullscreen($core.bool value) => $_setBool(12, value);
  @$pb.TagNumber(13)
  $core.bool hasIsFullscreen() => $_has(12);
  @$pb.TagNumber(13)
  void clearIsFullscreen() => $_clearField(13);
}

/// Identifies a specific pane to target with an action.
class PaneTarget extends $pb.GeneratedMessage {
  factory PaneTarget({
    $core.String? session,
    $core.int? paneId,
    $core.bool? isPlugin,
    $core.String? connectionId,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (paneId != null) result.paneId = paneId;
    if (isPlugin != null) result.isPlugin = isPlugin;
    if (connectionId != null) result.connectionId = connectionId;
    return result;
  }

  PaneTarget._();

  factory PaneTarget.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PaneTarget.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PaneTarget',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..aI(2, _omitFieldNames ? '' : 'paneId', fieldType: $pb.PbFieldType.OU3)
    ..aOB(3, _omitFieldNames ? '' : 'isPlugin')
    ..aOS(4, _omitFieldNames ? '' : 'connectionId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaneTarget clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaneTarget copyWith(void Function(PaneTarget) updates) =>
      super.copyWith((message) => updates(message as PaneTarget)) as PaneTarget;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PaneTarget create() => PaneTarget._();
  @$core.override
  PaneTarget createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PaneTarget getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PaneTarget>(create);
  static PaneTarget? _defaultInstance;

  /// Session the pane lives in.
  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  /// Pane id (unique within its kind — terminal or plugin).
  @$pb.TagNumber(2)
  $core.int get paneId => $_getIZ(1);
  @$pb.TagNumber(2)
  set paneId($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPaneId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPaneId() => $_clearField(2);

  /// True if this is a plugin pane → PaneId::Plugin; else PaneId::Terminal.
  @$pb.TagNumber(3)
  $core.bool get isPlugin => $_getBF(2);
  @$pb.TagNumber(3)
  set isPlugin($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIsPlugin() => $_has(2);
  @$pb.TagNumber(3)
  void clearIsPlugin() => $_clearField(3);

  /// optional; echoed from AttachTerminal's minted id to route per-connection (empty = legacy/fallback routing)
  @$pb.TagNumber(4)
  $core.String get connectionId => $_getSZ(3);
  @$pb.TagNumber(4)
  set connectionId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasConnectionId() => $_has(3);
  @$pb.TagNumber(4)
  void clearConnectionId() => $_clearField(4);
}

/// Toggle-fullscreen request with an optional floating-pane hint (Bug 2c).
///
/// The relay's fullscreen decision needs to know, for the target pane: whether
/// it is floating, whether floating panes are visible in its tab, and whether it
/// is the focused floating pane. The mobile client already polls all three
/// (Pane.is_floating, Tab.floating_panes_visible, Pane.is_focused), so it sends
/// them here to spare the relay a synchronous IPC query on the hot path.
///
/// has_floating_hint is the explicit presence flag: because proto3 bools default
/// to false, an all-false hint is indistinguishable from "not provided". A caller
/// that knows the floating state MUST set has_floating_hint=true; the relay then
/// trusts the three bools. When has_floating_hint=false (e.g. a target-only
/// request) the relay ignores them and runs a live IPC query — so a hint-less
/// caller can never silently mis-route a floating pane onto the tiled path.
class ToggleFullscreenReq extends $pb.GeneratedMessage {
  factory ToggleFullscreenReq({
    PaneTarget? target,
    $core.bool? targetIsFloating,
    $core.bool? floatingVisible,
    $core.bool? targetIsFocusedFloating,
    $core.bool? hasFloatingHint,
  }) {
    final result = create();
    if (target != null) result.target = target;
    if (targetIsFloating != null) result.targetIsFloating = targetIsFloating;
    if (floatingVisible != null) result.floatingVisible = floatingVisible;
    if (targetIsFocusedFloating != null)
      result.targetIsFocusedFloating = targetIsFocusedFloating;
    if (hasFloatingHint != null) result.hasFloatingHint = hasFloatingHint;
    return result;
  }

  ToggleFullscreenReq._();

  factory ToggleFullscreenReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ToggleFullscreenReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ToggleFullscreenReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOM<PaneTarget>(1, _omitFieldNames ? '' : 'target',
        subBuilder: PaneTarget.create)
    ..aOB(2, _omitFieldNames ? '' : 'targetIsFloating')
    ..aOB(3, _omitFieldNames ? '' : 'floatingVisible')
    ..aOB(4, _omitFieldNames ? '' : 'targetIsFocusedFloating')
    ..aOB(5, _omitFieldNames ? '' : 'hasFloatingHint')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ToggleFullscreenReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ToggleFullscreenReq copyWith(void Function(ToggleFullscreenReq) updates) =>
      super.copyWith((message) => updates(message as ToggleFullscreenReq))
          as ToggleFullscreenReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ToggleFullscreenReq create() => ToggleFullscreenReq._();
  @$core.override
  ToggleFullscreenReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ToggleFullscreenReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ToggleFullscreenReq>(create);
  static ToggleFullscreenReq? _defaultInstance;

  @$pb.TagNumber(1)
  PaneTarget get target => $_getN(0);
  @$pb.TagNumber(1)
  set target(PaneTarget value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearTarget() => $_clearField(1);
  @$pb.TagNumber(1)
  PaneTarget ensureTarget() => $_ensure(0);

  /// Pane.is_floating for the target pane.
  @$pb.TagNumber(2)
  $core.bool get targetIsFloating => $_getBF(1);
  @$pb.TagNumber(2)
  set targetIsFloating($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTargetIsFloating() => $_has(1);
  @$pb.TagNumber(2)
  void clearTargetIsFloating() => $_clearField(2);

  /// Tab.floating_panes_visible for the target pane's tab.
  @$pb.TagNumber(3)
  $core.bool get floatingVisible => $_getBF(2);
  @$pb.TagNumber(3)
  set floatingVisible($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFloatingVisible() => $_has(2);
  @$pb.TagNumber(3)
  void clearFloatingVisible() => $_clearField(3);

  /// True iff the target is the currently-focused, visible floating pane
  /// (target_is_floating && floating_visible && pane.is_focused).
  @$pb.TagNumber(4)
  $core.bool get targetIsFocusedFloating => $_getBF(3);
  @$pb.TagNumber(4)
  set targetIsFocusedFloating($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTargetIsFocusedFloating() => $_has(3);
  @$pb.TagNumber(4)
  void clearTargetIsFocusedFloating() => $_clearField(4);

  /// True iff the three fields above are populated and should be trusted. When
  /// false the relay falls back to a live floating-state query.
  @$pb.TagNumber(5)
  $core.bool get hasFloatingHint => $_getBF(4);
  @$pb.TagNumber(5)
  set hasFloatingHint($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasHasFloatingHint() => $_has(4);
  @$pb.TagNumber(5)
  void clearHasFloatingHint() => $_clearField(5);
}

/// Identifies a specific tab to target with an action (Phase D2).
class TabTarget extends $pb.GeneratedMessage {
  factory TabTarget({
    $core.String? session,
    $fixnum.Int64? tabId,
    $core.String? connectionId,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (tabId != null) result.tabId = tabId;
    if (connectionId != null) result.connectionId = connectionId;
    return result;
  }

  TabTarget._();

  factory TabTarget.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TabTarget.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TabTarget',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'tabId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'connectionId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TabTarget clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TabTarget copyWith(void Function(TabTarget) updates) =>
      super.copyWith((message) => updates(message as TabTarget)) as TabTarget;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TabTarget create() => TabTarget._();
  @$core.override
  TabTarget createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TabTarget getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TabTarget>(create);
  static TabTarget? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get tabId => $_getI64(1);
  @$pb.TagNumber(2)
  set tabId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTabId() => $_has(1);
  @$pb.TagNumber(2)
  void clearTabId() => $_clearField(2);

  /// optional; echoed from AttachTerminal's minted id to route per-connection (empty = legacy/fallback routing)
  @$pb.TagNumber(3)
  $core.String get connectionId => $_getSZ(2);
  @$pb.TagNumber(3)
  set connectionId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasConnectionId() => $_has(2);
  @$pb.TagNumber(3)
  void clearConnectionId() => $_clearField(3);
}

/// Generic acknowledgement returned by action RPCs.
class ActionAck extends $pb.GeneratedMessage {
  factory ActionAck({
    $core.bool? ok,
    $core.String? error,
    $core.String? info,
  }) {
    final result = create();
    if (ok != null) result.ok = ok;
    if (error != null) result.error = error;
    if (info != null) result.info = info;
    return result;
  }

  ActionAck._();

  factory ActionAck.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ActionAck.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ActionAck',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'ok')
    ..aOS(2, _omitFieldNames ? '' : 'error')
    ..aOS(3, _omitFieldNames ? '' : 'info')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ActionAck clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ActionAck copyWith(void Function(ActionAck) updates) =>
      super.copyWith((message) => updates(message as ActionAck)) as ActionAck;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ActionAck create() => ActionAck._();
  @$core.override
  ActionAck createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ActionAck getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ActionAck>(create);
  static ActionAck? _defaultInstance;

  /// True if the action completed without a surfaced error.
  @$pb.TagNumber(1)
  $core.bool get ok => $_getBF(0);
  @$pb.TagNumber(1)
  set ok($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOk() => $_has(0);
  @$pb.TagNumber(1)
  void clearOk() => $_clearField(1);

  /// Surfaced error message (LogError), empty if none.
  @$pb.TagNumber(2)
  $core.String get error => $_getSZ(1);
  @$pb.TagNumber(2)
  set error($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);

  /// Info payload (Log) — e.g. a newly-created pane id "terminal_<n>", empty if none.
  @$pb.TagNumber(3)
  $core.String get info => $_getSZ(2);
  @$pb.TagNumber(3)
  set info($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInfo() => $_has(2);
  @$pb.TagNumber(3)
  void clearInfo() => $_clearField(3);
}

/// WriteToPane request.
class WriteToPaneReq extends $pb.GeneratedMessage {
  factory WriteToPaneReq({
    PaneTarget? target,
    $core.List<$core.int>? data,
  }) {
    final result = create();
    if (target != null) result.target = target;
    if (data != null) result.data = data;
    return result;
  }

  WriteToPaneReq._();

  factory WriteToPaneReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WriteToPaneReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WriteToPaneReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOM<PaneTarget>(1, _omitFieldNames ? '' : 'target',
        subBuilder: PaneTarget.create)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'data', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WriteToPaneReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WriteToPaneReq copyWith(void Function(WriteToPaneReq) updates) =>
      super.copyWith((message) => updates(message as WriteToPaneReq))
          as WriteToPaneReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WriteToPaneReq create() => WriteToPaneReq._();
  @$core.override
  WriteToPaneReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WriteToPaneReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WriteToPaneReq>(create);
  static WriteToPaneReq? _defaultInstance;

  @$pb.TagNumber(1)
  PaneTarget get target => $_getN(0);
  @$pb.TagNumber(1)
  set target(PaneTarget value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearTarget() => $_clearField(1);
  @$pb.TagNumber(1)
  PaneTarget ensureTarget() => $_ensure(0);

  /// Raw bytes to write to the pane (e.g. keystrokes, including newlines).
  @$pb.TagNumber(2)
  $core.List<$core.int> get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => $_clearField(2);
}

/// NewPane request.
class NewPaneReq extends $pb.GeneratedMessage {
  factory NewPaneReq({
    $core.String? session,
    $core.bool? floating,
    $core.String? paneName,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (floating != null) result.floating = floating;
    if (paneName != null) result.paneName = paneName;
    return result;
  }

  NewPaneReq._();

  factory NewPaneReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory NewPaneReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'NewPaneReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..aOB(2, _omitFieldNames ? '' : 'floating')
    ..aOS(3, _omitFieldNames ? '' : 'paneName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NewPaneReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NewPaneReq copyWith(void Function(NewPaneReq) updates) =>
      super.copyWith((message) => updates(message as NewPaneReq)) as NewPaneReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static NewPaneReq create() => NewPaneReq._();
  @$core.override
  NewPaneReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static NewPaneReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<NewPaneReq>(create);
  static NewPaneReq? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  /// If true, open a floating pane; else a focused tiled pane.
  @$pb.TagNumber(2)
  $core.bool get floating => $_getBF(1);
  @$pb.TagNumber(2)
  set floating($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFloating() => $_has(1);
  @$pb.TagNumber(2)
  void clearFloating() => $_clearField(2);

  /// Optional name for the new pane.
  @$pb.TagNumber(3)
  $core.String get paneName => $_getSZ(2);
  @$pb.TagNumber(3)
  set paneName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPaneName() => $_has(2);
  @$pb.TagNumber(3)
  void clearPaneName() => $_clearField(3);
}

/// RenamePane request.
class RenamePaneReq extends $pb.GeneratedMessage {
  factory RenamePaneReq({
    PaneTarget? target,
    $core.String? name,
  }) {
    final result = create();
    if (target != null) result.target = target;
    if (name != null) result.name = name;
    return result;
  }

  RenamePaneReq._();

  factory RenamePaneReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RenamePaneReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RenamePaneReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOM<PaneTarget>(1, _omitFieldNames ? '' : 'target',
        subBuilder: PaneTarget.create)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenamePaneReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenamePaneReq copyWith(void Function(RenamePaneReq) updates) =>
      super.copyWith((message) => updates(message as RenamePaneReq))
          as RenamePaneReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RenamePaneReq create() => RenamePaneReq._();
  @$core.override
  RenamePaneReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RenamePaneReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenamePaneReq>(create);
  static RenamePaneReq? _defaultInstance;

  @$pb.TagNumber(1)
  PaneTarget get target => $_getN(0);
  @$pb.TagNumber(1)
  set target(PaneTarget value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearTarget() => $_clearField(1);
  @$pb.TagNumber(1)
  PaneTarget ensureTarget() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

/// ResizePane request.
class ResizePaneReq extends $pb.GeneratedMessage {
  factory ResizePaneReq({
    PaneTarget? target,
    ResizeKind? resize,
    ResizeDirection? direction,
  }) {
    final result = create();
    if (target != null) result.target = target;
    if (resize != null) result.resize = resize;
    if (direction != null) result.direction = direction;
    return result;
  }

  ResizePaneReq._();

  factory ResizePaneReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ResizePaneReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResizePaneReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOM<PaneTarget>(1, _omitFieldNames ? '' : 'target',
        subBuilder: PaneTarget.create)
    ..aE<ResizeKind>(2, _omitFieldNames ? '' : 'resize',
        enumValues: ResizeKind.values)
    ..aE<ResizeDirection>(3, _omitFieldNames ? '' : 'direction',
        enumValues: ResizeDirection.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResizePaneReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResizePaneReq copyWith(void Function(ResizePaneReq) updates) =>
      super.copyWith((message) => updates(message as ResizePaneReq))
          as ResizePaneReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ResizePaneReq create() => ResizePaneReq._();
  @$core.override
  ResizePaneReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ResizePaneReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ResizePaneReq>(create);
  static ResizePaneReq? _defaultInstance;

  @$pb.TagNumber(1)
  PaneTarget get target => $_getN(0);
  @$pb.TagNumber(1)
  set target(PaneTarget value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearTarget() => $_clearField(1);
  @$pb.TagNumber(1)
  PaneTarget ensureTarget() => $_ensure(0);

  @$pb.TagNumber(2)
  ResizeKind get resize => $_getN(1);
  @$pb.TagNumber(2)
  set resize(ResizeKind value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasResize() => $_has(1);
  @$pb.TagNumber(2)
  void clearResize() => $_clearField(2);

  /// Direction is optional; UNSPECIFIED resizes uniformly.
  @$pb.TagNumber(3)
  ResizeDirection get direction => $_getN(2);
  @$pb.TagNumber(3)
  set direction(ResizeDirection value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasDirection() => $_has(2);
  @$pb.TagNumber(3)
  void clearDirection() => $_clearField(3);
}

/// NewTab request.
class NewTabReq extends $pb.GeneratedMessage {
  factory NewTabReq({
    $core.String? session,
    $core.String? tabName,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (tabName != null) result.tabName = tabName;
    return result;
  }

  NewTabReq._();

  factory NewTabReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory NewTabReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'NewTabReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..aOS(2, _omitFieldNames ? '' : 'tabName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NewTabReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NewTabReq copyWith(void Function(NewTabReq) updates) =>
      super.copyWith((message) => updates(message as NewTabReq)) as NewTabReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static NewTabReq create() => NewTabReq._();
  @$core.override
  NewTabReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static NewTabReq getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<NewTabReq>(create);
  static NewTabReq? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  /// Optional name for the new tab.
  @$pb.TagNumber(2)
  $core.String get tabName => $_getSZ(1);
  @$pb.TagNumber(2)
  set tabName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTabName() => $_has(1);
  @$pb.TagNumber(2)
  void clearTabName() => $_clearField(2);
}

/// RenameTab request.
class RenameTabReq extends $pb.GeneratedMessage {
  factory RenameTabReq({
    $core.String? session,
    $fixnum.Int64? tabId,
    $core.String? name,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (tabId != null) result.tabId = tabId;
    if (name != null) result.name = name;
    return result;
  }

  RenameTabReq._();

  factory RenameTabReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RenameTabReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RenameTabReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'tabId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameTabReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameTabReq copyWith(void Function(RenameTabReq) updates) =>
      super.copyWith((message) => updates(message as RenameTabReq))
          as RenameTabReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RenameTabReq create() => RenameTabReq._();
  @$core.override
  RenameTabReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RenameTabReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenameTabReq>(create);
  static RenameTabReq? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get tabId => $_getI64(1);
  @$pb.TagNumber(2)
  set tabId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTabId() => $_has(1);
  @$pb.TagNumber(2)
  void clearTabId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get name => $_getSZ(2);
  @$pb.TagNumber(3)
  set name($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasName() => $_has(2);
  @$pb.TagNumber(3)
  void clearName() => $_clearField(3);
}

/// ScrollPane request.
class ScrollReq extends $pb.GeneratedMessage {
  factory ScrollReq({
    PaneTarget? target,
    ScrollDirection? direction,
  }) {
    final result = create();
    if (target != null) result.target = target;
    if (direction != null) result.direction = direction;
    return result;
  }

  ScrollReq._();

  factory ScrollReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ScrollReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ScrollReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOM<PaneTarget>(1, _omitFieldNames ? '' : 'target',
        subBuilder: PaneTarget.create)
    ..aE<ScrollDirection>(2, _omitFieldNames ? '' : 'direction',
        enumValues: ScrollDirection.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScrollReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScrollReq copyWith(void Function(ScrollReq) updates) =>
      super.copyWith((message) => updates(message as ScrollReq)) as ScrollReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ScrollReq create() => ScrollReq._();
  @$core.override
  ScrollReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ScrollReq getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ScrollReq>(create);
  static ScrollReq? _defaultInstance;

  @$pb.TagNumber(1)
  PaneTarget get target => $_getN(0);
  @$pb.TagNumber(1)
  set target(PaneTarget value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearTarget() => $_clearField(1);
  @$pb.TagNumber(1)
  PaneTarget ensureTarget() => $_ensure(0);

  @$pb.TagNumber(2)
  ScrollDirection get direction => $_getN(1);
  @$pb.TagNumber(2)
  set direction(ScrollDirection value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasDirection() => $_has(1);
  @$pb.TagNumber(2)
  void clearDirection() => $_clearField(2);
}

/// RenameSession request.
class RenameSessionReq extends $pb.GeneratedMessage {
  factory RenameSessionReq({
    $core.String? session,
    $core.String? name,
  }) {
    final result = create();
    if (session != null) result.session = session;
    if (name != null) result.name = name;
    return result;
  }

  RenameSessionReq._();

  factory RenameSessionReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RenameSessionReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RenameSessionReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'session')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameSessionReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameSessionReq copyWith(void Function(RenameSessionReq) updates) =>
      super.copyWith((message) => updates(message as RenameSessionReq))
          as RenameSessionReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RenameSessionReq create() => RenameSessionReq._();
  @$core.override
  RenameSessionReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RenameSessionReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenameSessionReq>(create);
  static RenameSessionReq? _defaultInstance;

  /// Session to rename (must match the bearer token's session).
  @$pb.TagNumber(1)
  $core.String get session => $_getSZ(0);
  @$pb.TagNumber(1)
  set session($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSession() => $_has(0);
  @$pb.TagNumber(1)
  void clearSession() => $_clearField(1);

  /// New name for the session.
  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

/// CreateSession request.
class CreateSessionReq extends $pb.GeneratedMessage {
  factory CreateSessionReq({
    $core.String? name,
    $core.String? layout,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (layout != null) result.layout = layout;
    return result;
  }

  CreateSessionReq._();

  factory CreateSessionReq.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateSessionReq.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateSessionReq',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'zellimserver.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'layout')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateSessionReq clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateSessionReq copyWith(void Function(CreateSessionReq) updates) =>
      super.copyWith((message) => updates(message as CreateSessionReq))
          as CreateSessionReq;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateSessionReq create() => CreateSessionReq._();
  @$core.override
  CreateSessionReq createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateSessionReq getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateSessionReq>(create);
  static CreateSessionReq? _defaultInstance;

  /// Name for the new session.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// Optional layout file path (passed as --layout to zellij; unused if empty).
  @$pb.TagNumber(2)
  $core.String get layout => $_getSZ(1);
  @$pb.TagNumber(2)
  set layout($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLayout() => $_has(1);
  @$pb.TagNumber(2)
  void clearLayout() => $_clearField(2);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
