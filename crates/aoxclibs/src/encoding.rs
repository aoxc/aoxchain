use base64::{Engine as _, engine::general_purpose};

use crate::types::LibError;

#[must_use]
pub fn encode_hex_upper(data: &[u8]) -> String {
    hex::encode_upper(data)
}

#[must_use]
pub fn encode_hex_lower(data: &[u8]) -> String {
    hex::encode(data)
}

pub fn decode_hex(data: &str) -> Result<Vec<u8>, LibError> {
    let normalized = normalize_hex_input(data)?;
    hex::decode(normalized).map_err(|e| LibError::EncodingError(e.to_string()))
}

/// Decode hexadecimal text and enforce a maximum output byte length.
pub fn decode_hex_with_max_len(data: &str, max_len: usize) -> Result<Vec<u8>, LibError> {
    let bytes = decode_hex(data)?;
    ensure_max_len(bytes.len(), max_len)?;
    Ok(bytes)
}

#[must_use]
pub fn encode_base64_standard(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

#[must_use]
pub fn encode_base64_urlsafe_no_pad(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn decode_base64_standard(data: &str) -> Result<Vec<u8>, LibError> {
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| LibError::EncodingError(e.to_string()))
}

pub fn decode_base64_standard_with_max_len(
    data: &str,
    max_len: usize,
) -> Result<Vec<u8>, LibError> {
    let bytes = decode_base64_standard(data)?;
    ensure_max_len(bytes.len(), max_len)?;
    Ok(bytes)
}

pub fn decode_base64_urlsafe_no_pad(data: &str) -> Result<Vec<u8>, LibError> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|e| LibError::EncodingError(e.to_string()))
}

pub fn decode_base64_urlsafe_no_pad_with_max_len(
    data: &str,
    max_len: usize,
) -> Result<Vec<u8>, LibError> {
    let bytes = decode_base64_urlsafe_no_pad(data)?;
    ensure_max_len(bytes.len(), max_len)?;
    Ok(bytes)
}

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
            "hex input cannot be only prefix".to_owned(),
        ));
    }

    if normalized.len() % 2 != 0 {
        return Err(LibError::ValidationError(
            "hex input must have even length".to_owned(),
        ));
    }

    Ok(normalized)
}

fn ensure_max_len(actual_len: usize, max_len: usize) -> Result<(), LibError> {
    if actual_len > max_len {
        return Err(LibError::ValidationError(format!(
            "decoded length {actual_len} exceeds maximum {max_len}",
        )));
    }
    Ok(())
}

#[must_use]
pub fn encode_base64_standard(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

#[must_use]
pub fn encode_base64_urlsafe_no_pad(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn decode_base64_standard(data: &str) -> Result<Vec<u8>, LibError> {
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| LibError::EncodingError(e.to_string()))
}

pub fn decode_base64_urlsafe_no_pad(data: &str) -> Result<Vec<u8>, LibError> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|e| LibError::EncodingError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_roundtrip_upper_and_lower() {
        let original = vec![0xDE, 0xAD, 0xBE, 0xEF];

        let encoded_upper = encode_hex_upper(&original);
        assert_eq!(encoded_upper, "DEADBEEF");
        let decoded_upper = decode_hex(&encoded_upper).expect("Decode upper failed");
        assert_eq!(decoded_upper, original);

        let encoded_lower = encode_hex_lower(&original);
        assert_eq!(encoded_lower, "deadbeef");
        let decoded_lower = decode_hex(&encoded_lower).expect("Decode lower failed");
        assert_eq!(decoded_lower, original);
    }

    #[test]
    fn test_base64_standard_roundtrip() {
        let original = b"aoxchain";
        let encoded = encode_base64_standard(original);
        let decoded = decode_base64_standard(&encoded).expect("Decode base64 standard failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_base64_urlsafe_no_pad_roundtrip() {
        let original = b"aoxchain/network";
        let encoded = encode_base64_urlsafe_no_pad(original);
        let decoded =
            decode_base64_urlsafe_no_pad(&encoded).expect("Decode base64 urlsafe no pad failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_invalid_data_returns_error() {
        let err = decode_hex("XYZ").expect_err("Invalid hex should fail");
        assert!(matches!(err, LibError::EncodingError(_)));

        let err = decode_base64_standard("@#%$").expect_err("Invalid base64 should fail");
        assert!(matches!(err, LibError::EncodingError(_)));
    }
}
