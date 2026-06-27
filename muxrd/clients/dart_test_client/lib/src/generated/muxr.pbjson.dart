// This is a generated file - do not edit.
//
// Generated from muxr.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use resizeDirectionDescriptor instead')
const ResizeDirection$json = {
  '1': 'ResizeDirection',
  '2': [
    {'1': 'RESIZE_DIRECTION_UNSPECIFIED', '2': 0},
    {'1': 'RESIZE_DIRECTION_LEFT', '2': 1},
    {'1': 'RESIZE_DIRECTION_RIGHT', '2': 2},
    {'1': 'RESIZE_DIRECTION_UP', '2': 3},
    {'1': 'RESIZE_DIRECTION_DOWN', '2': 4},
  ],
};

/// Descriptor for `ResizeDirection`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List resizeDirectionDescriptor = $convert.base64Decode(
    'Cg9SZXNpemVEaXJlY3Rpb24SIAocUkVTSVpFX0RJUkVDVElPTl9VTlNQRUNJRklFRBAAEhkKFV'
    'JFU0laRV9ESVJFQ1RJT05fTEVGVBABEhoKFlJFU0laRV9ESVJFQ1RJT05fUklHSFQQAhIXChNS'
    'RVNJWkVfRElSRUNUSU9OX1VQEAMSGQoVUkVTSVpFX0RJUkVDVElPTl9ET1dOEAQ=');

@$core.Deprecated('Use resizeKindDescriptor instead')
const ResizeKind$json = {
  '1': 'ResizeKind',
  '2': [
    {'1': 'RESIZE_KIND_INCREASE', '2': 0},
    {'1': 'RESIZE_KIND_DECREASE', '2': 1},
  ],
};

/// Descriptor for `ResizeKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List resizeKindDescriptor = $convert.base64Decode(
    'CgpSZXNpemVLaW5kEhgKFFJFU0laRV9LSU5EX0lOQ1JFQVNFEAASGAoUUkVTSVpFX0tJTkRfRE'
    'VDUkVBU0UQAQ==');

@$core.Deprecated('Use scrollDirectionDescriptor instead')
const ScrollDirection$json = {
  '1': 'ScrollDirection',
  '2': [
    {'1': 'SCROLL_DIRECTION_UP', '2': 0},
    {'1': 'SCROLL_DIRECTION_DOWN', '2': 1},
    {'1': 'SCROLL_DIRECTION_TO_TOP', '2': 2},
    {'1': 'SCROLL_DIRECTION_TO_BOTTOM', '2': 3},
    {'1': 'SCROLL_DIRECTION_PAGE_UP', '2': 4},
    {'1': 'SCROLL_DIRECTION_PAGE_DOWN', '2': 5},
    {'1': 'SCROLL_DIRECTION_HALF_PAGE_UP', '2': 6},
    {'1': 'SCROLL_DIRECTION_HALF_PAGE_DOWN', '2': 7},
  ],
};

/// Descriptor for `ScrollDirection`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List scrollDirectionDescriptor = $convert.base64Decode(
    'Cg9TY3JvbGxEaXJlY3Rpb24SFwoTU0NST0xMX0RJUkVDVElPTl9VUBAAEhkKFVNDUk9MTF9ESV'
    'JFQ1RJT05fRE9XThABEhsKF1NDUk9MTF9ESVJFQ1RJT05fVE9fVE9QEAISHgoaU0NST0xMX0RJ'
    'UkVDVElPTl9UT19CT1RUT00QAxIcChhTQ1JPTExfRElSRUNUSU9OX1BBR0VfVVAQBBIeChpTQ1'
    'JPTExfRElSRUNUSU9OX1BBR0VfRE9XThAFEiEKHVNDUk9MTF9ESVJFQ1RJT05fSEFMRl9QQUdF'
    'X1VQEAYSIwofU0NST0xMX0RJUkVDVElPTl9IQUxGX1BBR0VfRE9XThAH');

@$core.Deprecated('Use emptyDescriptor instead')
const Empty$json = {
  '1': 'Empty',
};

/// Descriptor for `Empty`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List emptyDescriptor =
    $convert.base64Decode('CgVFbXB0eQ==');

@$core.Deprecated('Use versionInfoDescriptor instead')
const VersionInfo$json = {
  '1': 'VersionInfo',
  '2': [
    {'1': 'server_version', '3': 1, '4': 1, '5': 9, '10': 'serverVersion'},
    {'1': 'zellij_version', '3': 2, '4': 1, '5': 9, '10': 'zellijVersion'},
  ],
};

/// Descriptor for `VersionInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List versionInfoDescriptor = $convert.base64Decode(
    'CgtWZXJzaW9uSW5mbxIlCg5zZXJ2ZXJfdmVyc2lvbhgBIAEoCVINc2VydmVyVmVyc2lvbhIlCg'
    '56ZWxsaWpfdmVyc2lvbhgCIAEoCVINemVsbGlqVmVyc2lvbg==');

@$core.Deprecated('Use loginRequestDescriptor instead')
const LoginRequest$json = {
  '1': 'LoginRequest',
  '2': [
    {'1': 'auth_token', '3': 1, '4': 1, '5': 9, '10': 'authToken'},
    {'1': 'remember_me', '3': 2, '4': 1, '5': 8, '10': 'rememberMe'},
  ],
};

/// Descriptor for `LoginRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List loginRequestDescriptor = $convert.base64Decode(
    'CgxMb2dpblJlcXVlc3QSHQoKYXV0aF90b2tlbhgBIAEoCVIJYXV0aFRva2VuEh8KC3JlbWVtYm'
    'VyX21lGAIgASgIUgpyZW1lbWJlck1l');

@$core.Deprecated('Use loginResponseDescriptor instead')
const LoginResponse$json = {
  '1': 'LoginResponse',
  '2': [
    {'1': 'session_token', '3': 1, '4': 1, '5': 9, '10': 'sessionToken'},
    {'1': 'is_read_only', '3': 2, '4': 1, '5': 8, '10': 'isReadOnly'},
  ],
};

/// Descriptor for `LoginResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List loginResponseDescriptor = $convert.base64Decode(
    'Cg1Mb2dpblJlc3BvbnNlEiMKDXNlc3Npb25fdG9rZW4YASABKAlSDHNlc3Npb25Ub2tlbhIgCg'
    'xpc19yZWFkX29ubHkYAiABKAhSCmlzUmVhZE9ubHk=');

@$core.Deprecated('Use tokenInfoDescriptor instead')
const TokenInfo$json = {
  '1': 'TokenInfo',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'token', '3': 2, '4': 1, '5': 9, '10': 'token'},
    {'1': 'read_only', '3': 3, '4': 1, '5': 8, '10': 'readOnly'},
    {'1': 'created_at', '3': 4, '4': 1, '5': 9, '10': 'createdAt'},
  ],
};

/// Descriptor for `TokenInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenInfoDescriptor = $convert.base64Decode(
    'CglUb2tlbkluZm8SEgoEbmFtZRgBIAEoCVIEbmFtZRIUCgV0b2tlbhgCIAEoCVIFdG9rZW4SGw'
    'oJcmVhZF9vbmx5GAMgASgIUghyZWFkT25seRIdCgpjcmVhdGVkX2F0GAQgASgJUgljcmVhdGVk'
    'QXQ=');

@$core.Deprecated('Use tokenListDescriptor instead')
const TokenList$json = {
  '1': 'TokenList',
  '2': [
    {
      '1': 'tokens',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.muxr.v1.TokenInfo',
      '10': 'tokens'
    },
  ],
};

/// Descriptor for `TokenList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenListDescriptor = $convert.base64Decode(
    'CglUb2tlbkxpc3QSKgoGdG9rZW5zGAEgAygLMhIubXV4ci52MS5Ub2tlbkluZm9SBnRva2Vucw'
    '==');

@$core.Deprecated('Use createTokenReqDescriptor instead')
const CreateTokenReq$json = {
  '1': 'CreateTokenReq',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'read_only', '3': 2, '4': 1, '5': 8, '10': 'readOnly'},
  ],
};

/// Descriptor for `CreateTokenReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createTokenReqDescriptor = $convert.base64Decode(
    'Cg5DcmVhdGVUb2tlblJlcRISCgRuYW1lGAEgASgJUgRuYW1lEhsKCXJlYWRfb25seRgCIAEoCF'
    'IIcmVhZE9ubHk=');

@$core.Deprecated('Use revokeTokenReqDescriptor instead')
const RevokeTokenReq$json = {
  '1': 'RevokeTokenReq',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RevokeTokenReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeTokenReqDescriptor =
    $convert.base64Decode('Cg5SZXZva2VUb2tlblJlcRISCgRuYW1lGAEgASgJUgRuYW1l');

@$core.Deprecated('Use clientFrameDescriptor instead')
const ClientFrame$json = {
  '1': 'ClientFrame',
  '2': [
    {
      '1': 'attach',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.AttachReq',
      '9': 0,
      '10': 'attach'
    },
    {'1': 'input', '3': 2, '4': 1, '5': 12, '9': 0, '10': 'input'},
    {
      '1': 'resize',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.Resize',
      '9': 0,
      '10': 'resize'
    },
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `ClientFrame`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientFrameDescriptor = $convert.base64Decode(
    'CgtDbGllbnRGcmFtZRIsCgZhdHRhY2gYASABKAsyEi5tdXhyLnYxLkF0dGFjaFJlcUgAUgZhdH'
    'RhY2gSFgoFaW5wdXQYAiABKAxIAFIFaW5wdXQSKQoGcmVzaXplGAMgASgLMg8ubXV4ci52MS5S'
    'ZXNpemVIAFIGcmVzaXplQgYKBGtpbmQ=');

@$core.Deprecated('Use attachReqDescriptor instead')
const AttachReq$json = {
  '1': 'AttachReq',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'rows', '3': 2, '4': 1, '5': 13, '10': 'rows'},
    {'1': 'cols', '3': 3, '4': 1, '5': 13, '10': 'cols'},
  ],
};

/// Descriptor for `AttachReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List attachReqDescriptor = $convert.base64Decode(
    'CglBdHRhY2hSZXESGAoHc2Vzc2lvbhgBIAEoCVIHc2Vzc2lvbhISCgRyb3dzGAIgASgNUgRyb3'
    'dzEhIKBGNvbHMYAyABKA1SBGNvbHM=');

@$core.Deprecated('Use resizeDescriptor instead')
const Resize$json = {
  '1': 'Resize',
  '2': [
    {'1': 'rows', '3': 1, '4': 1, '5': 13, '10': 'rows'},
    {'1': 'cols', '3': 2, '4': 1, '5': 13, '10': 'cols'},
  ],
};

/// Descriptor for `Resize`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resizeDescriptor = $convert.base64Decode(
    'CgZSZXNpemUSEgoEcm93cxgBIAEoDVIEcm93cxISCgRjb2xzGAIgASgNUgRjb2xz');

@$core.Deprecated('Use serverFrameDescriptor instead')
const ServerFrame$json = {
  '1': 'ServerFrame',
  '2': [
    {'1': 'render', '3': 1, '4': 1, '5': 12, '9': 0, '10': 'render'},
    {
      '1': 'control',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.ControlEvent',
      '9': 0,
      '10': 'control'
    },
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `ServerFrame`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List serverFrameDescriptor = $convert.base64Decode(
    'CgtTZXJ2ZXJGcmFtZRIYCgZyZW5kZXIYASABKAxIAFIGcmVuZGVyEjEKB2NvbnRyb2wYAiABKA'
    'syFS5tdXhyLnYxLkNvbnRyb2xFdmVudEgAUgdjb250cm9sQgYKBGtpbmQ=');

@$core.Deprecated('Use controlEventDescriptor instead')
const ControlEvent$json = {
  '1': 'ControlEvent',
  '2': [
    {'1': 'kind', '3': 1, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'payload', '3': 2, '4': 1, '5': 9, '10': 'payload'},
  ],
};

/// Descriptor for `ControlEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List controlEventDescriptor = $convert.base64Decode(
    'CgxDb250cm9sRXZlbnQSEgoEa2luZBgBIAEoCVIEa2luZBIYCgdwYXlsb2FkGAIgASgJUgdwYX'
    'lsb2Fk');

@$core.Deprecated('Use sessionRefDescriptor instead')
const SessionRef$json = {
  '1': 'SessionRef',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'connection_id', '3': 2, '4': 1, '5': 9, '10': 'connectionId'},
  ],
};

/// Descriptor for `SessionRef`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sessionRefDescriptor = $convert.base64Decode(
    'CgpTZXNzaW9uUmVmEhgKB3Nlc3Npb24YASABKAlSB3Nlc3Npb24SIwoNY29ubmVjdGlvbl9pZB'
    'gCIAEoCVIMY29ubmVjdGlvbklk');

@$core.Deprecated('Use sessionListDescriptor instead')
const SessionList$json = {
  '1': 'SessionList',
  '2': [
    {
      '1': 'sessions',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.muxr.v1.SessionInfo',
      '10': 'sessions'
    },
  ],
};

/// Descriptor for `SessionList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sessionListDescriptor = $convert.base64Decode(
    'CgtTZXNzaW9uTGlzdBIwCghzZXNzaW9ucxgBIAMoCzIULm11eHIudjEuU2Vzc2lvbkluZm9SCH'
    'Nlc3Npb25z');

@$core.Deprecated('Use sessionInfoDescriptor instead')
const SessionInfo$json = {
  '1': 'SessionInfo',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'age_secs', '3': 2, '4': 1, '5': 4, '10': 'ageSecs'},
    {'1': 'resurrectable', '3': 3, '4': 1, '5': 8, '10': 'resurrectable'},
    {'1': 'tab_count', '3': 4, '4': 1, '5': 13, '10': 'tabCount'},
    {'1': 'pane_count', '3': 5, '4': 1, '5': 13, '10': 'paneCount'},
    {'1': 'has_bell', '3': 6, '4': 1, '5': 8, '10': 'hasBell'},
    {'1': 'is_current', '3': 7, '4': 1, '5': 8, '10': 'isCurrent'},
    {
      '1': 'connected_clients',
      '3': 8,
      '4': 1,
      '5': 13,
      '10': 'connectedClients'
    },
  ],
};

/// Descriptor for `SessionInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sessionInfoDescriptor = $convert.base64Decode(
    'CgtTZXNzaW9uSW5mbxISCgRuYW1lGAEgASgJUgRuYW1lEhkKCGFnZV9zZWNzGAIgASgEUgdhZ2'
    'VTZWNzEiQKDXJlc3VycmVjdGFibGUYAyABKAhSDXJlc3VycmVjdGFibGUSGwoJdGFiX2NvdW50'
    'GAQgASgNUgh0YWJDb3VudBIdCgpwYW5lX2NvdW50GAUgASgNUglwYW5lQ291bnQSGQoIaGFzX2'
    'JlbGwYBiABKAhSB2hhc0JlbGwSHQoKaXNfY3VycmVudBgHIAEoCFIJaXNDdXJyZW50EisKEWNv'
    'bm5lY3RlZF9jbGllbnRzGAggASgNUhBjb25uZWN0ZWRDbGllbnRz');

@$core.Deprecated('Use layoutDescriptor instead')
const Layout$json = {
  '1': 'Layout',
  '2': [
    {
      '1': 'tabs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.muxr.v1.TabMsg',
      '10': 'tabs'
    },
  ],
};

/// Descriptor for `Layout`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List layoutDescriptor = $convert.base64Decode(
    'CgZMYXlvdXQSIwoEdGFicxgBIAMoCzIPLm11eHIudjEuVGFiTXNnUgR0YWJz');

@$core.Deprecated('Use tabMsgDescriptor instead')
const TabMsg$json = {
  '1': 'TabMsg',
  '2': [
    {'1': 'position', '3': 1, '4': 1, '5': 13, '10': 'position'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'active', '3': 3, '4': 1, '5': 8, '10': 'active'},
    {'1': 'has_bell', '3': 4, '4': 1, '5': 8, '10': 'hasBell'},
    {'1': 'panes_to_hide', '3': 5, '4': 1, '5': 13, '10': 'panesToHide'},
    {'1': 'tab_id', '3': 6, '4': 1, '5': 13, '10': 'tabId'},
    {
      '1': 'panes',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.muxr.v1.PaneMsg',
      '10': 'panes'
    },
    {
      '1': 'fullscreen_active',
      '3': 8,
      '4': 1,
      '5': 8,
      '10': 'fullscreenActive'
    },
    {
      '1': 'floating_panes_visible',
      '3': 9,
      '4': 1,
      '5': 8,
      '10': 'floatingPanesVisible'
    },
  ],
};

/// Descriptor for `TabMsg`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tabMsgDescriptor = $convert.base64Decode(
    'CgZUYWJNc2cSGgoIcG9zaXRpb24YASABKA1SCHBvc2l0aW9uEhIKBG5hbWUYAiABKAlSBG5hbW'
    'USFgoGYWN0aXZlGAMgASgIUgZhY3RpdmUSGQoIaGFzX2JlbGwYBCABKAhSB2hhc0JlbGwSIgoN'
    'cGFuZXNfdG9faGlkZRgFIAEoDVILcGFuZXNUb0hpZGUSFQoGdGFiX2lkGAYgASgNUgV0YWJJZB'
    'ImCgVwYW5lcxgHIAMoCzIQLm11eHIudjEuUGFuZU1zZ1IFcGFuZXMSKwoRZnVsbHNjcmVlbl9h'
    'Y3RpdmUYCCABKAhSEGZ1bGxzY3JlZW5BY3RpdmUSNAoWZmxvYXRpbmdfcGFuZXNfdmlzaWJsZR'
    'gJIAEoCFIUZmxvYXRpbmdQYW5lc1Zpc2libGU=');

@$core.Deprecated('Use paneMsgDescriptor instead')
const PaneMsg$json = {
  '1': 'PaneMsg',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    {'1': 'title', '3': 2, '4': 1, '5': 9, '10': 'title'},
    {'1': 'is_focused', '3': 3, '4': 1, '5': 8, '10': 'isFocused'},
    {'1': 'is_floating', '3': 4, '4': 1, '5': 8, '10': 'isFloating'},
    {'1': 'exited', '3': 5, '4': 1, '5': 8, '10': 'exited'},
    {'1': 'command', '3': 6, '4': 1, '5': 9, '10': 'command'},
    {'1': 'cwd', '3': 7, '4': 1, '5': 9, '10': 'cwd'},
    {'1': 'x', '3': 8, '4': 1, '5': 13, '10': 'x'},
    {'1': 'y', '3': 9, '4': 1, '5': 13, '10': 'y'},
    {'1': 'rows', '3': 10, '4': 1, '5': 13, '10': 'rows'},
    {'1': 'cols', '3': 11, '4': 1, '5': 13, '10': 'cols'},
    {'1': 'is_plugin', '3': 12, '4': 1, '5': 8, '10': 'isPlugin'},
    {'1': 'is_fullscreen', '3': 13, '4': 1, '5': 8, '10': 'isFullscreen'},
  ],
};

/// Descriptor for `PaneMsg`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paneMsgDescriptor = $convert.base64Decode(
    'CgdQYW5lTXNnEg4KAmlkGAEgASgNUgJpZBIUCgV0aXRsZRgCIAEoCVIFdGl0bGUSHQoKaXNfZm'
    '9jdXNlZBgDIAEoCFIJaXNGb2N1c2VkEh8KC2lzX2Zsb2F0aW5nGAQgASgIUgppc0Zsb2F0aW5n'
    'EhYKBmV4aXRlZBgFIAEoCFIGZXhpdGVkEhgKB2NvbW1hbmQYBiABKAlSB2NvbW1hbmQSEAoDY3'
    'dkGAcgASgJUgNjd2QSDAoBeBgIIAEoDVIBeBIMCgF5GAkgASgNUgF5EhIKBHJvd3MYCiABKA1S'
    'BHJvd3MSEgoEY29scxgLIAEoDVIEY29scxIbCglpc19wbHVnaW4YDCABKAhSCGlzUGx1Z2luEi'
    'MKDWlzX2Z1bGxzY3JlZW4YDSABKAhSDGlzRnVsbHNjcmVlbg==');

@$core.Deprecated('Use paneTargetDescriptor instead')
const PaneTarget$json = {
  '1': 'PaneTarget',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'pane_id', '3': 2, '4': 1, '5': 13, '10': 'paneId'},
    {'1': 'is_plugin', '3': 3, '4': 1, '5': 8, '10': 'isPlugin'},
    {'1': 'connection_id', '3': 4, '4': 1, '5': 9, '10': 'connectionId'},
  ],
};

/// Descriptor for `PaneTarget`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paneTargetDescriptor = $convert.base64Decode(
    'CgpQYW5lVGFyZ2V0EhgKB3Nlc3Npb24YASABKAlSB3Nlc3Npb24SFwoHcGFuZV9pZBgCIAEoDV'
    'IGcGFuZUlkEhsKCWlzX3BsdWdpbhgDIAEoCFIIaXNQbHVnaW4SIwoNY29ubmVjdGlvbl9pZBgE'
    'IAEoCVIMY29ubmVjdGlvbklk');

@$core.Deprecated('Use toggleFullscreenReqDescriptor instead')
const ToggleFullscreenReq$json = {
  '1': 'ToggleFullscreenReq',
  '2': [
    {
      '1': 'target',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.PaneTarget',
      '10': 'target'
    },
    {
      '1': 'target_is_floating',
      '3': 2,
      '4': 1,
      '5': 8,
      '10': 'targetIsFloating'
    },
    {'1': 'floating_visible', '3': 3, '4': 1, '5': 8, '10': 'floatingVisible'},
    {
      '1': 'target_is_focused_floating',
      '3': 4,
      '4': 1,
      '5': 8,
      '10': 'targetIsFocusedFloating'
    },
    {'1': 'has_floating_hint', '3': 5, '4': 1, '5': 8, '10': 'hasFloatingHint'},
  ],
};

/// Descriptor for `ToggleFullscreenReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List toggleFullscreenReqDescriptor = $convert.base64Decode(
    'ChNUb2dnbGVGdWxsc2NyZWVuUmVxEisKBnRhcmdldBgBIAEoCzITLm11eHIudjEuUGFuZVRhcm'
    'dldFIGdGFyZ2V0EiwKEnRhcmdldF9pc19mbG9hdGluZxgCIAEoCFIQdGFyZ2V0SXNGbG9hdGlu'
    'ZxIpChBmbG9hdGluZ192aXNpYmxlGAMgASgIUg9mbG9hdGluZ1Zpc2libGUSOwoadGFyZ2V0X2'
    'lzX2ZvY3VzZWRfZmxvYXRpbmcYBCABKAhSF3RhcmdldElzRm9jdXNlZEZsb2F0aW5nEioKEWhh'
    'c19mbG9hdGluZ19oaW50GAUgASgIUg9oYXNGbG9hdGluZ0hpbnQ=');

@$core.Deprecated('Use tabTargetDescriptor instead')
const TabTarget$json = {
  '1': 'TabTarget',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'tab_id', '3': 2, '4': 1, '5': 4, '10': 'tabId'},
    {'1': 'connection_id', '3': 3, '4': 1, '5': 9, '10': 'connectionId'},
  ],
};

/// Descriptor for `TabTarget`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tabTargetDescriptor = $convert.base64Decode(
    'CglUYWJUYXJnZXQSGAoHc2Vzc2lvbhgBIAEoCVIHc2Vzc2lvbhIVCgZ0YWJfaWQYAiABKARSBX'
    'RhYklkEiMKDWNvbm5lY3Rpb25faWQYAyABKAlSDGNvbm5lY3Rpb25JZA==');

@$core.Deprecated('Use actionAckDescriptor instead')
const ActionAck$json = {
  '1': 'ActionAck',
  '2': [
    {'1': 'ok', '3': 1, '4': 1, '5': 8, '10': 'ok'},
    {'1': 'error', '3': 2, '4': 1, '5': 9, '10': 'error'},
    {'1': 'info', '3': 3, '4': 1, '5': 9, '10': 'info'},
  ],
};

/// Descriptor for `ActionAck`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List actionAckDescriptor = $convert.base64Decode(
    'CglBY3Rpb25BY2sSDgoCb2sYASABKAhSAm9rEhQKBWVycm9yGAIgASgJUgVlcnJvchISCgRpbm'
    'ZvGAMgASgJUgRpbmZv');

@$core.Deprecated('Use writeToPaneReqDescriptor instead')
const WriteToPaneReq$json = {
  '1': 'WriteToPaneReq',
  '2': [
    {
      '1': 'target',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.PaneTarget',
      '10': 'target'
    },
    {'1': 'data', '3': 2, '4': 1, '5': 12, '10': 'data'},
  ],
};

/// Descriptor for `WriteToPaneReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List writeToPaneReqDescriptor = $convert.base64Decode(
    'Cg5Xcml0ZVRvUGFuZVJlcRIrCgZ0YXJnZXQYASABKAsyEy5tdXhyLnYxLlBhbmVUYXJnZXRSBn'
    'RhcmdldBISCgRkYXRhGAIgASgMUgRkYXRh');

@$core.Deprecated('Use newPaneReqDescriptor instead')
const NewPaneReq$json = {
  '1': 'NewPaneReq',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'floating', '3': 2, '4': 1, '5': 8, '10': 'floating'},
    {'1': 'pane_name', '3': 3, '4': 1, '5': 9, '10': 'paneName'},
  ],
};

/// Descriptor for `NewPaneReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List newPaneReqDescriptor = $convert.base64Decode(
    'CgpOZXdQYW5lUmVxEhgKB3Nlc3Npb24YASABKAlSB3Nlc3Npb24SGgoIZmxvYXRpbmcYAiABKA'
    'hSCGZsb2F0aW5nEhsKCXBhbmVfbmFtZRgDIAEoCVIIcGFuZU5hbWU=');

@$core.Deprecated('Use renamePaneReqDescriptor instead')
const RenamePaneReq$json = {
  '1': 'RenamePaneReq',
  '2': [
    {
      '1': 'target',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.PaneTarget',
      '10': 'target'
    },
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenamePaneReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renamePaneReqDescriptor = $convert.base64Decode(
    'Cg1SZW5hbWVQYW5lUmVxEisKBnRhcmdldBgBIAEoCzITLm11eHIudjEuUGFuZVRhcmdldFIGdG'
    'FyZ2V0EhIKBG5hbWUYAiABKAlSBG5hbWU=');

@$core.Deprecated('Use resizePaneReqDescriptor instead')
const ResizePaneReq$json = {
  '1': 'ResizePaneReq',
  '2': [
    {
      '1': 'target',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.PaneTarget',
      '10': 'target'
    },
    {
      '1': 'resize',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.muxr.v1.ResizeKind',
      '10': 'resize'
    },
    {
      '1': 'direction',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.muxr.v1.ResizeDirection',
      '10': 'direction'
    },
  ],
};

/// Descriptor for `ResizePaneReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resizePaneReqDescriptor = $convert.base64Decode(
    'Cg1SZXNpemVQYW5lUmVxEisKBnRhcmdldBgBIAEoCzITLm11eHIudjEuUGFuZVRhcmdldFIGdG'
    'FyZ2V0EisKBnJlc2l6ZRgCIAEoDjITLm11eHIudjEuUmVzaXplS2luZFIGcmVzaXplEjYKCWRp'
    'cmVjdGlvbhgDIAEoDjIYLm11eHIudjEuUmVzaXplRGlyZWN0aW9uUglkaXJlY3Rpb24=');

@$core.Deprecated('Use newTabReqDescriptor instead')
const NewTabReq$json = {
  '1': 'NewTabReq',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'tab_name', '3': 2, '4': 1, '5': 9, '10': 'tabName'},
  ],
};

/// Descriptor for `NewTabReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List newTabReqDescriptor = $convert.base64Decode(
    'CglOZXdUYWJSZXESGAoHc2Vzc2lvbhgBIAEoCVIHc2Vzc2lvbhIZCgh0YWJfbmFtZRgCIAEoCV'
    'IHdGFiTmFtZQ==');

@$core.Deprecated('Use renameTabReqDescriptor instead')
const RenameTabReq$json = {
  '1': 'RenameTabReq',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'tab_id', '3': 2, '4': 1, '5': 4, '10': 'tabId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameTabReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameTabReqDescriptor = $convert.base64Decode(
    'CgxSZW5hbWVUYWJSZXESGAoHc2Vzc2lvbhgBIAEoCVIHc2Vzc2lvbhIVCgZ0YWJfaWQYAiABKA'
    'RSBXRhYklkEhIKBG5hbWUYAyABKAlSBG5hbWU=');

@$core.Deprecated('Use scrollReqDescriptor instead')
const ScrollReq$json = {
  '1': 'ScrollReq',
  '2': [
    {
      '1': 'target',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.muxr.v1.PaneTarget',
      '10': 'target'
    },
    {
      '1': 'direction',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.muxr.v1.ScrollDirection',
      '10': 'direction'
    },
  ],
};

/// Descriptor for `ScrollReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scrollReqDescriptor = $convert.base64Decode(
    'CglTY3JvbGxSZXESKwoGdGFyZ2V0GAEgASgLMhMubXV4ci52MS5QYW5lVGFyZ2V0UgZ0YXJnZX'
    'QSNgoJZGlyZWN0aW9uGAIgASgOMhgubXV4ci52MS5TY3JvbGxEaXJlY3Rpb25SCWRpcmVjdGlv'
    'bg==');

@$core.Deprecated('Use renameSessionReqDescriptor instead')
const RenameSessionReq$json = {
  '1': 'RenameSessionReq',
  '2': [
    {'1': 'session', '3': 1, '4': 1, '5': 9, '10': 'session'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameSessionReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameSessionReqDescriptor = $convert.base64Decode(
    'ChBSZW5hbWVTZXNzaW9uUmVxEhgKB3Nlc3Npb24YASABKAlSB3Nlc3Npb24SEgoEbmFtZRgCIA'
    'EoCVIEbmFtZQ==');

@$core.Deprecated('Use createSessionReqDescriptor instead')
const CreateSessionReq$json = {
  '1': 'CreateSessionReq',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'layout', '3': 2, '4': 1, '5': 9, '10': 'layout'},
  ],
};

/// Descriptor for `CreateSessionReq`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createSessionReqDescriptor = $convert.base64Decode(
    'ChBDcmVhdGVTZXNzaW9uUmVxEhIKBG5hbWUYASABKAlSBG5hbWUSFgoGbGF5b3V0GAIgASgJUg'
    'ZsYXlvdXQ=');
