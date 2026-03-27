use base64::{Engine as _, engine::general_purpose};

use crate::types::LibError;

/// Encodes the provided byte slice into an uppercase hexadecimal string.
///
/// This function performs a deterministic byte-to-text transformation and does not allocate
/// intermediate buffers beyond the output `String`.
#[must_use]
pub fn encode_hex_upper(data: &[u8]) -> String {
    hex::encode_upper(data)
}

/// Encodes the provided byte slice into a lowercase hexadecimal string.
///
/// This function is suitable for canonical machine-oriented serialization where lowercase
/// hexadecimal output is preferred.
#[must_use]
pub fn encode_hex_lower(data: &[u8]) -> String {
    hex::encode(data)
}

/// Decodes a hexadecimal string into raw bytes.
///
/// Accepted input rules:
/// - Leading and trailing ASCII/Unicode whitespace is ignored.
/// - Optional `0x` or `0X` prefix is accepted.
/// - The normalized hexadecimal payload must not be empty.
/// - The normalized hexadecimal payload must have an even number of characters.
///
/// Returns:
/// - `Ok(Vec<u8>)` when decoding succeeds.
/// - `Err(LibError)` when validation or decoding fails.
pub fn decode_hex(data: &str) -> Result<Vec<u8>, LibError> {
    let normalized = normalize_hex_input(data)?;

    hex::decode(normalized).map_err(|e| LibError::EncodingError(e.to_string()))
}

/// Decodes a hexadecimal string into raw bytes and enforces an upper bound on output length.
///
/// This function should be preferred when decoding untrusted input in order to reduce memory
/// amplification risk and to establish deterministic validation boundaries.
pub fn decode_hex_with_max_len(data: &str, max_len: usize) -> Result<Vec<u8>, LibError> {
    let bytes = decode_hex(data)?;
    ensure_max_len(bytes.len(), max_len)?;
    Ok(bytes)
}

/// Decodes a hexadecimal string into raw bytes and enforces an exact output length.
///
/// This is useful for fixed-width identity material (e.g., 32-byte hashes or keys).
pub fn decode_hex_exact_len(data: &str, exact_len: usize) -> Result<Vec<u8>, LibError> {
    let bytes = decode_hex(data)?;
    ensure_exact_len(bytes.len(), exact_len)?;
    Ok(bytes)
}

/// Encodes the provided byte slice using standard Base64 as defined by RFC 4648.
///
/// Padding is preserved.
#[must_use]
pub fn encode_base64_standard(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Encodes the provided byte slice using URL-safe Base64 without padding.
///
/// This representation is suitable for identifiers, URLs, and transport contexts where
/// `+`, `/`, and trailing `=` padding are undesirable.
#[must_use]
pub fn encode_base64_urlsafe_no_pad(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Decodes a standard Base64 string into raw bytes.
///
/// Returns a domain-specific `LibError::EncodingError` on malformed input.
pub fn decode_base64_standard(data: &str) -> Result<Vec<u8>, LibError> {
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| LibError::EncodingError(e.to_string()))
}

/// Decodes a standard Base64 string into raw bytes and enforces an upper bound on output length.
///
/// This function is appropriate for defensive decoding of externally supplied payloads.
pub fn decode_base64_standard_with_max_len(
    data: &str,
    max_len: usize,
) -> Result<Vec<u8>, LibError> {
    let bytes = decode_base64_standard(data)?;
    ensure_max_len(bytes.len(), max_len)?;
    Ok(bytes)
}

/// Decodes a standard Base64 string into raw bytes and enforces an exact output length.
pub fn decode_base64_standard_exact_len(data: &str, exact_len: usize) -> Result<Vec<u8>, LibError> {
    let bytes = decode_base64_standard(data)?;
    ensure_exact_len(bytes.len(), exact_len)?;
    Ok(bytes)
}

/// Decodes a URL-safe Base64 string without padding into raw bytes.
///
/// Returns a domain-specific `LibError::EncodingError` on malformed input.
pub fn decode_base64_urlsafe_no_pad(data: &str) -> Result<Vec<u8>, LibError> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|e| LibError::EncodingError(e.to_string()))
}

/// Decodes a URL-safe Base64 string without padding into raw bytes and enforces an upper bound
/// on output length.
///
/// This function is recommended for all decoding paths that process untrusted input.
pub fn decode_base64_urlsafe_no_pad_with_max_len(
    data: &str,
    max_len: usize,
) -> Result<Vec<u8>, LibError> {
    let bytes = decode_base64_urlsafe_no_pad(data)?;
    ensure_max_len(bytes.len(), max_len)?;
    Ok(bytes)
}

/// Normalizes hexadecimal input before decoding.
///
/// Normalization rules:
/// - Trims surrounding whitespace.
/// - Removes an optional `0x` or `0X` prefix.
/// - Rejects empty payloads.
/// - Rejects odd-length payloads.
///
/// The returned slice always borrows from the original input and performs no heap allocation.
fn normalize_hex_input(input: &str) -> Result<&str, LibError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(LibError::ValidationError(
            "hex input cannot be empty".to_owned(),
        ));
    }

    let normalized = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    if normalized.is_empty() {
        return Err(LibError::ValidationError(
            "hex input cannot contain only a prefix".to_owned(),
        ));
    }

    if !normalized.len().is_multiple_of(2) {
        return Err(LibError::ValidationError(
            "hex input must have an even number of characters".to_owned(),
        ));
    }

    Ok(normalized)
}

/// Enforces a maximum decoded output length.
///
/// This function centralizes output-size validation to keep all decoding call sites consistent.
fn ensure_max_len(actual_len: usize, max_len: usize) -> Result<(), LibError> {
    if actual_len > max_len {
        return Err(LibError::ValidationError(format!(
            "decoded length {actual_len} exceeds maximum allowed length {max_len}"
        )));
    }

    Ok(())
}

/// Enforces an exact decoded output length.
fn ensure_exact_len(actual_len: usize, exact_len: usize) -> Result<(), LibError> {
    if actual_len != exact_len {
        return Err(LibError::ValidationError(format!(
            "decoded length {actual_len} does not match required length {exact_len}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_roundtrip_upper_and_lower() {
        let original = vec![0xDE, 0xAD, 0xBE, 0xEF];

        let encoded_upper = encode_hex_upper(&original);
        assert_eq!(encoded_upper, "DEADBEEF");

        let decoded_upper = decode_hex(&encoded_upper).expect("uppercase hex decode must succeed");
        assert_eq!(decoded_upper, original);

        let encoded_lower = encode_hex_lower(&original);
        assert_eq!(encoded_lower, "deadbeef");

        let decoded_lower = decode_hex(&encoded_lower).expect("lowercase hex decode must succeed");
        assert_eq!(decoded_lower, original);
    }

    #[test]
    fn test_hex_roundtrip_with_prefix_and_whitespace() {
        let decoded = decode_hex("  0xdeadbeef  ").expect("prefixed hex decode must succeed");
        assert_eq!(decoded, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_decode_hex_rejects_empty_input() {
        let err = decode_hex("   ").expect_err("empty hex input must fail");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_decode_hex_rejects_prefix_only() {
        let err = decode_hex("0x").expect_err("prefix-only hex input must fail");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_decode_hex_rejects_odd_length() {
        let err = decode_hex("ABC").expect_err("odd-length hex input must fail");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_decode_hex_rejects_invalid_characters() {
        let err = decode_hex("ZZ").expect_err("invalid hex characters must fail");
        assert!(matches!(err, LibError::EncodingError(_)));
    }

    #[test]
    fn test_decode_hex_with_max_len_enforces_limit() {
        let err =
            decode_hex_with_max_len("DEADBEEF", 3).expect_err("length limit must be enforced");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_decode_hex_exact_len_enforces_exact_length() {
        let decoded = decode_hex_exact_len("DEADBEEF", 4).expect("exact len decode must succeed");
        assert_eq!(decoded, vec![0xDE, 0xAD, 0xBE, 0xEF]);

        let err = decode_hex_exact_len("DEADBEEF", 3).expect_err("exact len mismatch must fail");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_base64_standard_roundtrip() {
        let original = b"aoxchain";

        let encoded = encode_base64_standard(original);
        let decoded =
            decode_base64_standard(&encoded).expect("standard base64 decode must succeed");

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_base64_urlsafe_no_pad_roundtrip() {
        let original = b"aoxchain/network";

        let encoded = encode_base64_urlsafe_no_pad(original);
        let decoded = decode_base64_urlsafe_no_pad(&encoded)
            .expect("URL-safe no-pad base64 decode must succeed");

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_base64_standard_with_max_len_enforces_limit() {
        let encoded = encode_base64_standard(b"abcd");
        let err = decode_base64_standard_with_max_len(&encoded, 3)
            .expect_err("length limit must be enforced");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_decode_base64_standard_exact_len_enforces_exact_length() {
        let encoded = encode_base64_standard(b"abcd");
        let decoded =
            decode_base64_standard_exact_len(&encoded, 4).expect("exact len decode must succeed");
        assert_eq!(decoded, b"abcd");

        let err = decode_base64_standard_exact_len(&encoded, 5)
            .expect_err("exact len mismatch must fail");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_decode_base64_urlsafe_no_pad_with_max_len_enforces_limit() {
        let encoded = encode_base64_urlsafe_no_pad(b"abcd");
        let err = decode_base64_urlsafe_no_pad_with_max_len(&encoded, 3)
            .expect_err("length limit must be enforced");
        assert!(matches!(err, LibError::ValidationError(_)));
    }

    #[test]
    fn test_decode_base64_standard_rejects_invalid_data() {
        let err = decode_base64_standard("@#%$").expect_err("invalid base64 input must fail");
        assert!(matches!(err, LibError::EncodingError(_)));
    }

    #[test]
    fn test_decode_base64_urlsafe_no_pad_rejects_invalid_data() {
        let err = decode_base64_urlsafe_no_pad("@#%$")
            .expect_err("invalid URL-safe base64 input must fail");
        assert!(matches!(err, LibError::EncodingError(_)));
    }
}
