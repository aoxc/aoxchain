use core::str::FromStr;

use super::{
    ASSET_CODE_NAMESPACE, ASSET_CODE_SEQUENCE_LEN, AssetClass, AssetCode, AssetRegistryError,
    SupplyModel,
};

impl FromStr for AssetCode {
    type Err = AssetRegistryError;

    /// Parses an asset code without semantic class/model context.
    ///
    /// Audit note:
    /// This parser validates structural correctness only. For full semantic
    /// validation, `AssetCode::new` should be preferred because it cross-checks
    /// the code against the declared class and supply model.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_asset_code_structure(s)?;
        Ok(Self(s.to_owned()))
    }
}

pub(super) fn validate_asset_code(
    value: &str,
    asset_class: AssetClass,
    supply_model: SupplyModel,
) -> Result<(), AssetRegistryError> {
    validate_asset_code_structure(value)?;

    let parts = split_asset_code(value)?;
    let expected_class = class_segment(asset_class);
    let expected_policy = policy_segment(supply_model);

    if parts[1] != expected_class {
        return Err(AssetRegistryError::InvalidAssetCodeClassSegment {
            expected: expected_class,
            found: parts[1].to_owned(),
        });
    }

    if parts[2] != expected_policy {
        return Err(AssetRegistryError::InvalidAssetCodePolicySegment {
            expected: expected_policy,
            found: parts[2].to_owned(),
        });
    }

    Ok(())
}

fn validate_asset_code_structure(value: &str) -> Result<(), AssetRegistryError> {
    let parts = split_asset_code(value)?;

    if parts[0] != ASSET_CODE_NAMESPACE {
        return Err(AssetRegistryError::InvalidAssetCodeNamespace);
    }

    if !is_known_class_segment(parts[1]) {
        return Err(AssetRegistryError::InvalidAssetCodeClassSegment {
            expected: "known class segment",
            found: parts[1].to_owned(),
        });
    }

    if !is_known_policy_segment(parts[2]) {
        return Err(AssetRegistryError::InvalidAssetCodePolicySegment {
            expected: "known policy segment",
            found: parts[2].to_owned(),
        });
    }

    if parts[3].len() != ASSET_CODE_SEQUENCE_LEN || !parts[3].chars().all(|c| c.is_ascii_digit()) {
        return Err(AssetRegistryError::InvalidAssetCodeSequenceSegment {
            found: parts[3].to_owned(),
        });
    }

    Ok(())
}

fn split_asset_code(value: &str) -> Result<[&str; 4], AssetRegistryError> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 4 || parts.iter().any(|p| p.is_empty()) {
        return Err(AssetRegistryError::InvalidAssetCodeFormat);
    }

    let array: [&str; 4] = parts
        .try_into()
        .map_err(|_| AssetRegistryError::InvalidAssetCodeFormat)?;
    Ok(array)
}

fn class_segment(class: AssetClass) -> &'static str {
    match class {
        AssetClass::Native => "NATIVE",
        AssetClass::Constitutional => "CONST",
        AssetClass::System => "SYSTEM",
        AssetClass::Treasury => "TREASURY",
        AssetClass::Governance => "GOV",
        AssetClass::Utility => "UTIL",
        AssetClass::Synthetic => "SYNTH",
        AssetClass::Wrapped => "WRAPPED",
        AssetClass::Experimental => "EXPERIMENTAL",
    }
}

fn policy_segment(model: SupplyModel) -> &'static str {
    match model {
        SupplyModel::FixedGenesis => "FIXED",
        SupplyModel::TreasuryAuthorizedEmission => "TREASURY",
        SupplyModel::GovernanceAuthorizedEmission => "GOV",
        SupplyModel::ProgrammaticEmission => "PROGRAM",
        SupplyModel::WrappedBacked => "WRAP",
    }
}

fn is_known_class_segment(value: &str) -> bool {
    matches!(
        value,
        "NATIVE"
            | "CONST"
            | "SYSTEM"
            | "TREASURY"
            | "GOV"
            | "UTIL"
            | "SYNTH"
            | "WRAPPED"
            | "EXPERIMENTAL"
    )
}

fn is_known_policy_segment(value: &str) -> bool {
    matches!(value, "FIXED" | "TREASURY" | "GOV" | "PROGRAM" | "WRAP")
}
