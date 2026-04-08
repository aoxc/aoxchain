use super::*;
use crate::identity::hd_path::HdPath;

fn sample_seed() -> [u8; MASTER_SEED_LEN] {
    [0x11; MASTER_SEED_LEN]
}

fn sample_path() -> HdPath {
    HdPath {
        chain: 1,
        role: 1,
        zone: 2,
        index: 0,
    }
}

#[test]
fn deterministic_derivation_for_same_seed_and_path() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let a = engine
        .derive_entropy(&path)
        .expect("entropy derivation must succeed");
    let b = engine
        .derive_entropy(&path)
        .expect("entropy derivation must succeed");

    assert_eq!(a, b);
}

#[test]
fn derivation_changes_when_path_changes() {
    let engine = KeyEngine::from_seed(sample_seed());

    let a = engine
        .derive_entropy(&HdPath {
            chain: 1,
            role: 1,
            zone: 2,
            index: 0,
        })
        .expect("entropy derivation must succeed");

    let b = engine
        .derive_entropy(&HdPath {
            chain: 1,
            role: 1,
            zone: 2,
            index: 1,
        })
        .expect("entropy derivation must succeed");

    assert_ne!(a, b);
}

#[test]
fn derivation_changes_when_seed_changes() {
    let path = sample_path();

    let a = KeyEngine::from_seed([0x11; MASTER_SEED_LEN])
        .derive_entropy(&path)
        .expect("entropy derivation must succeed");
    let b = KeyEngine::from_seed([0x22; MASTER_SEED_LEN])
        .derive_entropy(&path)
        .expect("entropy derivation must succeed");

    assert_ne!(a, b);
}

#[test]
fn fingerprint_is_stable() {
    let engine = KeyEngine::from_seed(sample_seed());

    let a = engine.fingerprint();
    let b = engine.fingerprint();

    assert_eq!(a, b);
    assert_eq!(a.len(), 32);
}

#[test]
fn invalid_zero_path_is_rejected_in_strict_mode() {
    let engine = KeyEngine::from_seed(sample_seed());

    let path = HdPath {
        chain: 0,
        role: 0,
        zone: 0,
        index: 0,
    };

    assert_eq!(
        engine.try_derive_entropy(&path),
        Err(KeyEngineError::InvalidPath)
    );
}

#[test]
fn out_of_range_component_is_rejected() {
    let engine = KeyEngine::from_seed(sample_seed());

    let path = HdPath {
        chain: MAX_CANONICAL_HD_COMPONENT + 1,
        role: 1,
        zone: 1,
        index: 1,
    };

    assert_eq!(
        engine.try_derive_entropy(&path),
        Err(KeyEngineError::InvalidPath)
    );
}

#[test]
fn entropy_hex_has_expected_length() {
    let engine = KeyEngine::from_seed(sample_seed());

    let hex = engine
        .derive_entropy_hex(&sample_path())
        .expect("hex derivation must succeed");

    assert_eq!(hex.len(), DERIVED_ENTROPY_LEN * 2);
}

#[test]
fn derive_key_material_matches_entropy() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let a = engine
        .derive_key_material(&path)
        .expect("key material derivation must succeed");
    let b = engine
        .try_derive_entropy(&path)
        .expect("entropy derivation must succeed");

    assert_eq!(a, b);
}

#[test]
fn role_seed_derivation_is_deterministic() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let a = engine
        .derive_role_seed(&path, "consensus")
        .expect("role seed derivation must succeed");
    let b = engine
        .derive_role_seed(&path, "consensus")
        .expect("role seed derivation must succeed");

    assert_eq!(a, b);
}

#[test]
fn role_seed_derivation_changes_by_label() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let a = engine
        .derive_role_seed(&path, "consensus")
        .expect("role seed derivation must succeed");
    let b = engine
        .derive_role_seed(&path, "transport")
        .expect("role seed derivation must succeed");

    assert_ne!(a, b);
}

#[test]
fn empty_role_label_is_rejected() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let result = engine.derive_role_seed(&path, "");
    assert_eq!(result, Err(KeyEngineError::EmptyRoleLabel));
}

#[test]
fn whitespace_only_role_label_is_rejected() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let result = engine.derive_role_seed(&path, "   ");
    assert_eq!(result, Err(KeyEngineError::EmptyRoleLabel));
}

#[test]
fn surrounding_whitespace_in_role_label_is_rejected() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let result = engine.derive_role_seed(&path, " consensus ");
    assert_eq!(result, Err(KeyEngineError::InvalidRoleLabel));
}

#[test]
fn internal_whitespace_in_role_label_is_rejected() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let result = engine.derive_role_seed(&path, "consensus role");
    assert_eq!(result, Err(KeyEngineError::InvalidRoleLabel));
}

#[test]
fn invalid_characters_in_role_label_are_rejected() {
    let engine = KeyEngine::from_seed(sample_seed());
    let path = sample_path();

    let result = engine.derive_role_seed(&path, "consensus!");
    assert_eq!(result, Err(KeyEngineError::InvalidRoleLabel));
}

#[test]
fn error_codes_are_stable() {
    assert_eq!(
        KeyEngineError::InvalidPath.code(),
        "KEY_ENGINE_INVALID_PATH"
    );
    assert_eq!(
        KeyEngineError::InvalidEntropyLength.code(),
        "KEY_ENGINE_INVALID_ENTROPY_LENGTH"
    );
    assert_eq!(
        KeyEngineError::EmptyRoleLabel.code(),
        "KEY_ENGINE_EMPTY_ROLE_LABEL"
    );
    assert_eq!(
        KeyEngineError::InvalidRoleLabel.code(),
        "KEY_ENGINE_INVALID_ROLE_LABEL"
    );
}
