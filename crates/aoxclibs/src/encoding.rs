use crate::types::LibError;

#[must_use]
pub fn encode_hex_upper(data: &[u8]) -> String {
    hex::encode_upper(data)
}

pub fn decode_hex(data: &str) -> Result<Vec<u8>, LibError> {
    hex::decode(data).map_err(|e| LibError::EncodingError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_roundtrip() {
        let original = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let encoded = encode_hex_upper(&original);
        assert_eq!(encoded, "DEADBEEF");
        let decoded = decode_hex(&encoded).expect("Decode failed");
        assert_eq!(decoded, original);
    }
}
