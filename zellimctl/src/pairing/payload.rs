//! payload — QR pairing URI construction.
//!
//! Builds the `zellimobile://pair?...` URI that is encoded into the pairing QR
//! code.  The mobile client parses this URI to extract the server address, TLS
//! fingerprint, auth token, and label.
//!
//! ## URI format
//!
//! ```text
//! zellimobile://pair?v=1&h=<host>&p=<port>&fp=<hex-fp>&t=<base64url>&ro=<0|1>&n=<pct-encoded>
//! ```
//!
//! - `v=1`     — URI version (allows future breaking changes).
//! - `h`       — server host or IP.
//! - `p`       — server port.
//! - `fp`      — lowercase hex SHA-256 fingerprint of the server TLS cert DER.
//! - `t`       — base64url (no padding) of the plaintext auth token bytes.
//! - `ro`      — `1` for read-only tokens, `0` for read-write.
//! - `n`       — percent-encoded human label for the server (shown in the app).

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Parameters needed to generate a pairing QR code URI.
#[allow(dead_code)]
pub struct PairingParams {
    /// Server hostname or IP address.
    pub host: String,
    /// Server gRPC port.
    pub port: u16,
    /// Lowercase hex SHA-256 fingerprint of the server TLS certificate DER bytes.
    pub cert_fp_hex: String,
    /// Freshly-minted plaintext auth token (will be base64url-encoded in the URI).
    pub token: String,
    /// Whether this token grants read-only access.
    pub read_only: bool,
    /// Human-readable label for the server (e.g. "Home desktop").
    pub label: String,
}

impl PairingParams {
    /// Build the `zellimobile://pair?...` URI.
    ///
    /// The token is base64url-encoded (no padding) so it is safe to embed in a
    /// URL without further escaping.  The label is percent-encoded.
    #[allow(dead_code)]
    pub fn to_uri(&self) -> String {
        let token_b64 = URL_SAFE_NO_PAD.encode(self.token.as_bytes());
        let label_pct = percent_encode(&self.label);
        let ro = if self.read_only { "1" } else { "0" };

        format!(
            "zellimobile://pair?v=1&h={h}&p={p}&fp={fp}&t={t}&ro={ro}&n={n}",
            h = self.host,
            p = self.port,
            fp = self.cert_fp_hex,
            t = token_b64,
            ro = ro,
            n = label_pct,
        )
    }
}

/// Percent-encode a string for use in a URI query-parameter value.
///
/// Encodes all bytes that are not unreserved URI characters (ALPHA / DIGIT /
/// `-` / `_` / `.` / `~`) as defined in RFC 3986 §2.3.  This is intentionally
/// conservative so the result is parseable by a simple `split('&')` on the
/// mobile side.
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn example_params(read_only: bool) -> PairingParams {
        PairingParams {
            host: "192.168.1.10".to_string(),
            port: 50051,
            cert_fp_hex: "a1b2c3d4e5f6".repeat(5) + "a1b2c3d4", // 64-char fake hex
            token: "mysecrettoken".to_string(),
            read_only,
            label: "Home desktop".to_string(),
        }
    }

    #[test]
    fn uri_has_correct_scheme_and_host() {
        let uri = example_params(false).to_uri();
        assert!(
            uri.starts_with("zellimobile://pair?"),
            "URI should start with zellimobile://pair? but got: {uri}"
        );
    }

    #[test]
    fn uri_contains_version() {
        let uri = example_params(false).to_uri();
        assert!(uri.contains("v=1"), "URI should contain v=1: {uri}");
    }

    #[test]
    fn uri_contains_host_and_port() {
        let uri = example_params(false).to_uri();
        assert!(
            uri.contains("h=192.168.1.10"),
            "URI should contain host: {uri}"
        );
        assert!(uri.contains("p=50051"), "URI should contain port: {uri}");
    }

    #[test]
    fn uri_contains_fingerprint() {
        let params = example_params(false);
        let uri = params.to_uri();
        assert!(
            uri.contains(&format!("fp={}", params.cert_fp_hex)),
            "URI should contain fingerprint: {uri}"
        );
    }

    #[test]
    fn uri_token_is_base64url_encoded() {
        let params = example_params(false);
        let uri = params.to_uri();

        // Extract the `t=` value.
        let t_val = uri
            .split('&')
            .find(|s| s.starts_with("t="))
            .and_then(|s| s.strip_prefix("t="))
            .expect("URI should contain t= parameter");

        // Decode it and verify it round-trips to the original token.
        let decoded = URL_SAFE_NO_PAD
            .decode(t_val)
            .expect("t= value should be valid base64url");
        assert_eq!(
            std::str::from_utf8(&decoded).unwrap(),
            params.token,
            "base64url-decoded token should match original"
        );
    }

    #[test]
    fn uri_ro_flag_read_only() {
        let uri = example_params(true).to_uri();
        assert!(
            uri.contains("ro=1"),
            "read-only URI should have ro=1: {uri}"
        );
    }

    #[test]
    fn uri_ro_flag_read_write() {
        let uri = example_params(false).to_uri();
        assert!(
            uri.contains("ro=0"),
            "read-write URI should have ro=0: {uri}"
        );
    }

    #[test]
    fn uri_label_is_percent_encoded() {
        let params = example_params(false);
        let uri = params.to_uri();
        // "Home desktop" — the space should be percent-encoded as %20.
        assert!(
            uri.contains("n=Home%20desktop"),
            "label should be percent-encoded in URI: {uri}"
        );
    }

    #[test]
    fn percent_encode_unreserved_chars_unchanged() {
        let s = "abcABC012-_.~";
        assert_eq!(
            percent_encode(s),
            s,
            "unreserved chars should not be encoded"
        );
    }

    #[test]
    fn percent_encode_spaces_and_special() {
        assert_eq!(percent_encode(" "), "%20");
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode("a/b"), "a%2Fb");
        assert_eq!(percent_encode("a+b"), "a%2Bb");
    }

    #[test]
    fn uri_all_required_params_present() {
        let uri = example_params(false).to_uri();
        for param in &["v=", "h=", "p=", "fp=", "t=", "ro=", "n="] {
            assert!(
                uri.contains(param),
                "URI should contain param {param}: {uri}"
            );
        }
    }
}
