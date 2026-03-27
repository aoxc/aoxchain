use aoxclibs::{
    LibError,
    encoding::{
        decode_base64_standard_exact_len, decode_hex_exact_len, encode_base64_standard,
        encode_hex_upper,
    },
    time::{
        system_time_from_unix_timestamp, system_time_from_unix_timestamp_millis,
        unix_timestamp_from_system_time, unix_timestamp_millis_from_system_time,
    },
};

#[test]
fn public_encoding_contract_is_deterministic() {
    let hex = encode_hex_upper(b"aox");
    let hex_decoded = decode_hex_exact_len(&hex, 3).expect("hex exact-len decode failed");
    assert_eq!(hex_decoded, b"aox");

    let b64 = encode_base64_standard(b"chain");
    let b64_decoded =
        decode_base64_standard_exact_len(&b64, 5).expect("base64 exact-len decode failed");
    assert_eq!(b64_decoded, b"chain");
}

#[test]
fn public_time_contract_roundtrip() {
    let sys =
        system_time_from_unix_timestamp(1_700_000_123).expect("seconds -> system time failed");
    let sec = unix_timestamp_from_system_time(sys).expect("system time -> seconds failed");
    assert_eq!(sec, 1_700_000_123);

    let sys_ms = system_time_from_unix_timestamp_millis(1_700_000_123_456)
        .expect("millis -> system time failed");
    let ms = unix_timestamp_millis_from_system_time(sys_ms).expect("system time -> millis failed");
    assert_eq!(ms, 1_700_000_123_456);
}

#[test]
fn validation_errors_are_explicit() {
    let err = decode_hex_exact_len("AA", 2).expect_err("exact len mismatch should fail");
    assert!(matches!(err, LibError::ValidationError(_)));
}
