//! pairing — QR-code pairing URI construction and reachable-address discovery.
//!
//! This module is purely functional (no rendering):
//! - [`net`] — enumerate non-loopback IPv4 candidates the phone could reach.
//! - [`payload`] — build the `zellimobile://pair?...` URI that goes into the QR.

pub mod net;
pub mod payload;
