use super::*;
use core::str::FromStr;

fn bytes(v: u8) -> [u8; 32] {
    [v; 32]
}

fn valid_utility_entry() -> AssetRegistryEntry {
    AssetRegistryEntry::new(
        bytes(1),
        "AOXC.UTIL.TREASURY.0001",
        "AOXC Utility Credit",
        "AUX1",
        8,
        AssetClass::Utility,
        SupplyModel::TreasuryAuthorizedEmission,
        MintAuthority::Treasury,
        bytes(2),
        Some(1_000_000),
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    )
    .expect("valid utility entry must construct successfully")
}

#[test]
fn constructs_valid_entry() {
    let entry = valid_utility_entry();
    assert_eq!(entry.validate(), Ok(()));
}

#[test]
fn rejects_zero_asset_id() {
    let result = AssetRegistryEntry::new(
        [0u8; 32],
        "AOXC.UTIL.TREASURY.0001",
        "AOXC Utility Credit",
        "AUX1",
        8,
        AssetClass::Utility,
        SupplyModel::TreasuryAuthorizedEmission,
        MintAuthority::Treasury,
        bytes(2),
        Some(1_000_000),
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(result, Err(AssetRegistryError::ZeroAssetId));
}

#[test]
fn rejects_empty_display_name() {
    let result = DisplayName::new("   ");
    assert_eq!(result, Err(AssetRegistryError::EmptyDisplayName));
}

#[test]
fn rejects_invalid_symbol_length() {
    let result = AssetSymbol::new("A");
    assert_eq!(result, Err(AssetRegistryError::InvalidSymbolLength));
}

#[test]
fn rejects_invalid_symbol_format() {
    let result = AssetSymbol::new("aoxc");
    assert_eq!(result, Err(AssetRegistryError::InvalidSymbolFormat));
}

#[test]
fn rejects_symbol_starting_with_digit() {
    let result = AssetSymbol::new("1AOX");
    assert_eq!(result, Err(AssetRegistryError::InvalidSymbolFormat));
}

#[test]
fn parses_structurally_valid_asset_code_from_str() {
    let parsed = AssetCode::from_str("AOXC.UTIL.TREASURY.0001");
    assert!(parsed.is_ok());
}

#[test]
fn rejects_asset_code_with_invalid_namespace() {
    let result = AssetCode::from_str("TEST.UTIL.TREASURY.0001");
    assert_eq!(result, Err(AssetRegistryError::InvalidAssetCodeNamespace));
}

#[test]
fn rejects_asset_code_with_invalid_sequence() {
    let result = AssetCode::from_str("AOXC.UTIL.TREASURY.00A1");
    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidAssetCodeSequenceSegment {
            found: "00A1".to_owned()
        })
    );
}

#[test]
fn rejects_asset_code_class_mismatch() {
    let result = AssetCode::new(
        "AOXC.GOV.TREASURY.0001",
        AssetClass::Utility,
        SupplyModel::TreasuryAuthorizedEmission,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidAssetCodeClassSegment {
            expected: "UTIL",
            found: "GOV".to_owned()
        })
    );
}

#[test]
fn rejects_asset_code_policy_mismatch() {
    let result = AssetCode::new(
        "AOXC.UTIL.GOV.0001",
        AssetClass::Utility,
        SupplyModel::TreasuryAuthorizedEmission,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidAssetCodePolicySegment {
            expected: "TREASURY",
            found: "GOV".to_owned()
        })
    );
}

#[test]
fn rejects_decimals_above_limit() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.UTIL.TREASURY.0001",
        "AOXC Utility Credit",
        "AUX1",
        19,
        AssetClass::Utility,
        SupplyModel::TreasuryAuthorizedEmission,
        MintAuthority::Treasury,
        bytes(2),
        Some(1_000_000),
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidDecimals {
            provided: 19,
            maximum: MAX_DECIMALS
        })
    );
}

#[test]
fn rejects_zero_created_at_epoch() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.UTIL.TREASURY.0001",
        "AOXC Utility Credit",
        "AUX1",
        8,
        AssetClass::Utility,
        SupplyModel::TreasuryAuthorizedEmission,
        MintAuthority::Treasury,
        bytes(2),
        Some(1_000_000),
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        0,
    );

    assert_eq!(result, Err(AssetRegistryError::InvalidCreatedAtEpoch));
}

#[test]
fn rejects_missing_max_supply_for_fixed_genesis() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.UTIL.FIXED.0001",
        "AOXC Fixed Utility",
        "AUXF",
        8,
        AssetClass::Utility,
        SupplyModel::FixedGenesis,
        MintAuthority::ProtocolOnly,
        bytes(2),
        None,
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::MissingMaxSupplyForSupplyModel {
            supply_model: SupplyModel::FixedGenesis
        })
    );
}

#[test]
fn rejects_zero_max_supply() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.UTIL.FIXED.0001",
        "AOXC Fixed Utility",
        "AUXF",
        8,
        AssetClass::Utility,
        SupplyModel::FixedGenesis,
        MintAuthority::ProtocolOnly,
        bytes(2),
        Some(0),
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(result, Err(AssetRegistryError::ZeroMaxSupply));
}

#[test]
fn rejects_unexpected_max_supply_for_programmatic_emission() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.UTIL.PROGRAM.0001",
        "AOXC Programmatic Utility",
        "AUXP",
        8,
        AssetClass::Utility,
        SupplyModel::ProgrammaticEmission,
        MintAuthority::ProtocolOnly,
        bytes(2),
        Some(100),
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::UnexpectedMaxSupplyForSupplyModel {
            supply_model: SupplyModel::ProgrammaticEmission
        })
    );
}

#[test]
fn rejects_wrong_mint_authority_for_wrapped_supply() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.WRAPPED.WRAP.0001",
        "AOXC Wrapped Asset",
        "AWR1",
        8,
        AssetClass::Wrapped,
        SupplyModel::WrappedBacked,
        MintAuthority::ProtocolOnly,
        bytes(2),
        None,
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::MintAuthorityMismatch {
            supply_model: SupplyModel::WrappedBacked,
            mint_authority: MintAuthority::ProtocolOnly
        })
    );
}

#[test]
fn rejects_invalid_supply_model_for_wrapped_class() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.WRAPPED.TREASURY.0001",
        "AOXC Wrapped Asset",
        "AWR1",
        8,
        AssetClass::Wrapped,
        SupplyModel::TreasuryAuthorizedEmission,
        MintAuthority::Treasury,
        bytes(2),
        Some(100),
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidSupplyModelForAssetClass {
            asset_class: AssetClass::Wrapped,
            supply_model: SupplyModel::TreasuryAuthorizedEmission
        })
    );
}

#[test]
fn rejects_invalid_risk_grade_for_native_class() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.NATIVE.FIXED.0001",
        "AOXC Native",
        "AOXC",
        18,
        AssetClass::Native,
        SupplyModel::FixedGenesis,
        MintAuthority::ProtocolOnly,
        bytes(2),
        Some(100_000_000),
        RegistryStatus::Active,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidRiskGradeForAssetClass {
            asset_class: AssetClass::Native,
            risk_grade: RiskGrade::Medium
        })
    );
}

#[test]
fn rejects_non_finalized_status_for_native_class() {
    let result = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.NATIVE.FIXED.0001",
        "AOXC Native",
        "AOXC",
        18,
        AssetClass::Native,
        SupplyModel::FixedGenesis,
        MintAuthority::ProtocolOnly,
        bytes(2),
        Some(100_000_000),
        RegistryStatus::Registered,
        RiskGrade::Low,
        bytes(3),
        bytes(4),
        1,
    );

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidStatusForAssetClass {
            asset_class: AssetClass::Native,
            status: RegistryStatus::Registered
        })
    );
}

#[test]
fn allows_valid_status_transition() {
    let entry = valid_utility_entry();
    let updated = entry
        .transition_to(RegistryStatus::Active)
        .expect("registered -> active transition must succeed");

    assert_eq!(updated.registry_status, RegistryStatus::Active);
    assert_eq!(updated.validate(), Ok(()));
}

#[test]
fn rejects_invalid_status_transition() {
    let entry = valid_utility_entry();
    let result = entry.transition_to(RegistryStatus::Proposed);

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidStatusTransition {
            from: RegistryStatus::Registered,
            to: RegistryStatus::Proposed
        })
    );
}

#[test]
fn treats_revoked_as_terminal_state() {
    let revoked = valid_utility_entry()
        .transition_to(RegistryStatus::Active)
        .expect("registered -> active transition must succeed")
        .transition_to(RegistryStatus::Revoked)
        .expect("active -> revoked transition must succeed");

    let result = revoked.transition_to(RegistryStatus::Active);

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidStatusTransition {
            from: RegistryStatus::Revoked,
            to: RegistryStatus::Active
        })
    );
}

#[test]
fn allows_risk_grade_update_when_class_policy_remains_valid() {
    let entry = valid_utility_entry();
    let updated = entry
        .with_risk_grade(RiskGrade::High)
        .expect("utility asset should allow risk upgrade");

    assert_eq!(updated.risk_grade, RiskGrade::High);
}

#[test]
fn rejects_risk_grade_update_when_class_policy_becomes_invalid() {
    let entry = AssetRegistryEntry::new(
        bytes(1),
        "AOXC.EXPERIMENTAL.PROGRAM.0001",
        "AOXC Experimental Asset",
        "AEXP",
        8,
        AssetClass::Experimental,
        SupplyModel::ProgrammaticEmission,
        MintAuthority::ProtocolOnly,
        bytes(2),
        None,
        RegistryStatus::Registered,
        RiskGrade::Medium,
        bytes(3),
        bytes(4),
        1,
    )
    .expect("valid experimental entry must construct successfully");

    let result = entry.with_risk_grade(RiskGrade::Low);

    assert_eq!(
        result,
        Err(AssetRegistryError::InvalidRiskGradeForAssetClass {
            asset_class: AssetClass::Experimental,
            risk_grade: RiskGrade::Low
        })
    );
}
