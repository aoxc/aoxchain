use super::*;

fn sample_pubkey() -> Vec<u8> {
    vec![0x11; 32]
}

#[test]
fn actor_id_generation_is_deterministic() {
    let pk = sample_pubkey();

    let a =
        generate_actor_id(&pk, "validator", "europe").expect("actor id generation must succeed");
    let b = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

    assert_eq!(a, b);
}

#[test]
fn actor_id_contains_expected_structure() {
    let pk = sample_pubkey();
    let actor_id =
        generate_actor_id(&pk, "validator", "europe").expect("actor id generation must succeed");

    let parts: Vec<&str> = actor_id.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0], AOXC_PREFIX);
    assert_eq!(parts[1].len(), ROLE_LEN);
    assert_eq!(parts[2].len(), ZONE_LEN);
    assert_eq!(parts[3].len(), SERIAL_HEX_LEN);
    assert_eq!(parts[4].len(), CHECKSUM_LEN);
}

#[test]
fn actor_id_validation_accepts_valid_identifier() {
    let pk = sample_pubkey();
    let actor_id = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

    assert_eq!(validate_actor_id(&actor_id), Ok(()));
}

#[test]
fn actor_id_validation_rejects_checksum_mismatch() {
    let pk = sample_pubkey();
    let valid = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");
    let mut parts: Vec<&str> = valid.split('-').collect();

    parts[4] = "ZZ";
    let tampered = parts.join("-");

    assert_eq!(
        validate_actor_id(&tampered),
        Err(ActorIdError::ChecksumMismatch)
    );
}

#[test]
fn actor_id_binding_verification_accepts_matching_inputs() {
    let pk = sample_pubkey();
    let actor_id = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

    assert_eq!(verify_actor_id_binding(&actor_id, &pk, "VAL", "EU"), Ok(()));
}

#[test]
fn actor_id_binding_verification_rejects_wrong_zone() {
    let pk = sample_pubkey();
    let actor_id = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

    assert_eq!(
        verify_actor_id_binding(&actor_id, &pk, "VAL", "NA"),
        Err(ActorIdError::DerivationMismatch)
    );
}

#[test]
fn empty_public_key_is_rejected() {
    assert_eq!(
        generate_actor_id(&[], "VAL", "EU"),
        Err(ActorIdError::EmptyPublicKey)
    );
}

#[test]
fn too_short_public_key_is_rejected() {
    assert_eq!(
        generate_actor_id(&[0x11; 8], "VAL", "EU"),
        Err(ActorIdError::InvalidPublicKeyLength)
    );
}

#[test]
fn parse_actor_id_extracts_components() {
    let pk = sample_pubkey();
    let actor_id = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

    let parsed = parse_actor_id(&actor_id).expect("actor id parsing must succeed");
    assert_eq!(parsed.prefix, "AOXC");
    assert_eq!(parsed.role, "VAL");
    assert_eq!(parsed.zone, "EU");
    assert_eq!(parsed.serial.len(), SERIAL_HEX_LEN);
    assert_eq!(parsed.checksum.len(), CHECKSUM_LEN);
}

#[test]
fn validation_rejects_lowercase_actor_id() {
    let pk = sample_pubkey();
    let actor_id = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

    assert_eq!(
        validate_actor_id(&actor_id.to_ascii_lowercase()),
        Err(ActorIdError::InvalidPrefix)
    );
}

#[test]
fn validation_rejects_surrounding_whitespace() {
    let pk = sample_pubkey();
    let actor_id = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

    let candidate = format!(" {} ", actor_id);
    assert_eq!(
        validate_actor_id(&candidate),
        Err(ActorIdError::InvalidFormat)
    );
}

#[test]
fn generation_rejects_unknown_descriptive_role() {
    let pk = sample_pubkey();
    assert_eq!(
        generate_actor_id(&pk, "super-validator", "EU"),
        Err(ActorIdError::InvalidRole)
    );
}

#[test]
fn generation_rejects_unknown_descriptive_zone() {
    let pk = sample_pubkey();
    assert_eq!(
        generate_actor_id(&pk, "VAL", "antarctica"),
        Err(ActorIdError::InvalidZone)
    );
}

#[test]
fn canonical_aliases_are_supported_explicitly() {
    let pk = sample_pubkey();

    let a = generate_actor_id(&pk, "observer", "north-america")
        .expect("actor id generation must succeed");
    let b = generate_actor_id(&pk, "OBS", "NA").expect("actor id generation must succeed");

    assert_eq!(a, b);
}

#[test]
fn parse_rejects_whitespace_only_identifier() {
    assert_eq!(parse_actor_id("   "), Err(ActorIdError::EmptyActorId));
}

#[test]
fn parse_rejects_structural_component_count_mismatch() {
    assert_eq!(
        parse_actor_id("AOXC-VAL-EU-ABCD"),
        Err(ActorIdError::InvalidFormat)
    );
}

#[test]
fn validation_rejects_invalid_serial_lengths_and_characters() {
    let pk = sample_pubkey();
    let valid = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
    let mut parts: Vec<&str> = valid.split('-').collect();

    parts[3] = "ABC";
    let bad_len = parts.join("-");
    assert_eq!(
        validate_actor_id(&bad_len),
        Err(ActorIdError::InvalidSerial)
    );

    parts[3] = "ZZZZZZZZZZZZZZZZZZZZZZZZ";
    let bad_chars = parts.join("-");
    assert_eq!(
        validate_actor_id(&bad_chars),
        Err(ActorIdError::InvalidSerial)
    );
}

#[test]
fn validation_rejects_invalid_checksum_symbols() {
    let pk = sample_pubkey();
    let valid = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
    let mut parts: Vec<&str> = valid.split('-').collect();

    parts[4] = "IO";
    let actor_id = parts.join("-");

    assert_eq!(
        validate_actor_id(&actor_id),
        Err(ActorIdError::InvalidChecksum)
    );
}

#[test]
fn generate_and_validate_actor_id_matches_validation_path() {
    let pk = sample_pubkey();
    let actor_id =
        generate_and_validate_actor_id(&pk, "validator", "europe").expect("must succeed");

    assert_eq!(validate_actor_id(&actor_id), Ok(()));
}
