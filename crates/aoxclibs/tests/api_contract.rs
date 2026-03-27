use aoxclibs::{
    LibError,
    encoding::{
        decode_base64_standard_with_max_len, decode_hex_with_max_len, encode_base64_standard,
        encode_hex_upper,
    },
    time::{unix_timestamp_from_system_time, unix_timestamp_millis_from_system_time},
};
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn public_encoding_contract_is_deterministic() {
    let hex = encode_hex_upper(b"aox");
    let hex_decoded = decode_hex_with_max_len(&hex, 3).expect("hex max-len decode failed");
    assert_eq!(hex_decoded, b"aox");

    let b64 = encode_base64_standard(b"chain");
    let b64_decoded =
        decode_base64_standard_with_max_len(&b64, 5).expect("base64 max-len decode failed");
    assert_eq!(b64_decoded, b"chain");
}

#[test]
fn public_time_contract_roundtrip() {
    let sys = UNIX_EPOCH + Duration::from_secs(1_700_000_123);
    let sec = unix_timestamp_from_system_time(sys).expect("system time -> seconds failed");
    assert_eq!(sec, 1_700_000_123);

    let sys_ms = UNIX_EPOCH + Duration::from_millis(1_700_000_123_456);
    let ms = unix_timestamp_millis_from_system_time(sys_ms).expect("system time -> millis failed");
    assert_eq!(ms, 1_700_000_123_456);
}

#[test]
fn validation_errors_are_explicit() {
    let err = decode_hex_with_max_len("AA", 0).expect_err("max len mismatch should fail");
    assert!(matches!(err, LibError::ValidationError(_)));
}
