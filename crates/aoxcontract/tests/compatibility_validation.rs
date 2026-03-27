use aoxcontract::{
    Compatibility, CompatibilityError, ContractError, NetworkClass, RuntimeFamily, VmTarget,
};

#[test]
fn minimum_schema_must_be_supported() {
    let err = Compatibility::new(
        2,
        vec![1],
        vec![RuntimeFamily::Wasm],
        vec![NetworkClass::Mainnet],
        vec![VmTarget::Evm],
        false,
    )
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Compatibility(CompatibilityError::MinimumSchemaVersionNotSupported)
    ));
}

#[test]
fn duplicate_supported_schema_versions_are_rejected() {
    let err = Compatibility::new(
        1,
        vec![1, 1],
        vec![RuntimeFamily::Wasm],
        vec![NetworkClass::Mainnet],
        vec![VmTarget::Evm],
        false,
    )
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Compatibility(CompatibilityError::DuplicateSupportedSchemaVersions)
    ));
}

#[test]
fn runtime_families_cannot_be_empty() {
    let err = Compatibility::new(
        1,
        vec![1],
        vec![],
        vec![NetworkClass::Mainnet],
        vec![],
        false,
    )
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Compatibility(CompatibilityError::EmptySupportedRuntimeFamilies)
    ));
}
