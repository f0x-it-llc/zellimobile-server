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

import 'package:protobuf/protobuf.dart' as $pb;

/// Resize direction (which border / direction to grow or shrink).
class ResizeDirection extends $pb.ProtobufEnum {
  static const ResizeDirection RESIZE_DIRECTION_UNSPECIFIED = ResizeDirection._(
      0, _omitEnumNames ? '' : 'RESIZE_DIRECTION_UNSPECIFIED');
  static const ResizeDirection RESIZE_DIRECTION_LEFT =
      ResizeDirection._(1, _omitEnumNames ? '' : 'RESIZE_DIRECTION_LEFT');
  static const ResizeDirection RESIZE_DIRECTION_RIGHT =
      ResizeDirection._(2, _omitEnumNames ? '' : 'RESIZE_DIRECTION_RIGHT');
  static const ResizeDirection RESIZE_DIRECTION_UP =
      ResizeDirection._(3, _omitEnumNames ? '' : 'RESIZE_DIRECTION_UP');
  static const ResizeDirection RESIZE_DIRECTION_DOWN =
      ResizeDirection._(4, _omitEnumNames ? '' : 'RESIZE_DIRECTION_DOWN');

  static const $core.List<ResizeDirection> values = <ResizeDirection>[
    RESIZE_DIRECTION_UNSPECIFIED,
    RESIZE_DIRECTION_LEFT,
    RESIZE_DIRECTION_RIGHT,
    RESIZE_DIRECTION_UP,
    RESIZE_DIRECTION_DOWN,
  ];

  static final $core.List<ResizeDirection?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static ResizeDirection? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ResizeDirection._(super.value, super.name);
}

/// Whether to increase or decrease the pane size.
class ResizeKind extends $pb.ProtobufEnum {
  static const ResizeKind RESIZE_KIND_INCREASE =
      ResizeKind._(0, _omitEnumNames ? '' : 'RESIZE_KIND_INCREASE');
  static const ResizeKind RESIZE_KIND_DECREASE =
      ResizeKind._(1, _omitEnumNames ? '' : 'RESIZE_KIND_DECREASE');

  static const $core.List<ResizeKind> values = <ResizeKind>[
    RESIZE_KIND_INCREASE,
    RESIZE_KIND_DECREASE,
  ];

  static final $core.List<ResizeKind?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 1);
  static ResizeKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ResizeKind._(super.value, super.name);
}

/// Scroll direction enum.
class ScrollDirection extends $pb.ProtobufEnum {
  static const ScrollDirection SCROLL_DIRECTION_UP =
      ScrollDirection._(0, _omitEnumNames ? '' : 'SCROLL_DIRECTION_UP');
  static const ScrollDirection SCROLL_DIRECTION_DOWN =
      ScrollDirection._(1, _omitEnumNames ? '' : 'SCROLL_DIRECTION_DOWN');
  static const ScrollDirection SCROLL_DIRECTION_TO_TOP =
      ScrollDirection._(2, _omitEnumNames ? '' : 'SCROLL_DIRECTION_TO_TOP');
  static const ScrollDirection SCROLL_DIRECTION_TO_BOTTOM =
      ScrollDirection._(3, _omitEnumNames ? '' : 'SCROLL_DIRECTION_TO_BOTTOM');
  static const ScrollDirection SCROLL_DIRECTION_PAGE_UP =
      ScrollDirection._(4, _omitEnumNames ? '' : 'SCROLL_DIRECTION_PAGE_UP');
  static const ScrollDirection SCROLL_DIRECTION_PAGE_DOWN =
      ScrollDirection._(5, _omitEnumNames ? '' : 'SCROLL_DIRECTION_PAGE_DOWN');
  static const ScrollDirection SCROLL_DIRECTION_HALF_PAGE_UP =
      ScrollDirection._(
          6, _omitEnumNames ? '' : 'SCROLL_DIRECTION_HALF_PAGE_UP');
  static const ScrollDirection SCROLL_DIRECTION_HALF_PAGE_DOWN =
      ScrollDirection._(
          7, _omitEnumNames ? '' : 'SCROLL_DIRECTION_HALF_PAGE_DOWN');

  static const $core.List<ScrollDirection> values = <ScrollDirection>[
    SCROLL_DIRECTION_UP,
    SCROLL_DIRECTION_DOWN,
    SCROLL_DIRECTION_TO_TOP,
    SCROLL_DIRECTION_TO_BOTTOM,
    SCROLL_DIRECTION_PAGE_UP,
    SCROLL_DIRECTION_PAGE_DOWN,
    SCROLL_DIRECTION_HALF_PAGE_UP,
    SCROLL_DIRECTION_HALF_PAGE_DOWN,
  ];

  static final $core.List<ScrollDirection?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 7);
  static ScrollDirection? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ScrollDirection._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
