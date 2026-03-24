use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass {
    Constitutional,
    System,
    Registered,
    Restricted,
    Experimental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplyModel {
    FixedGenesis,
    GovernedEmission,
    ProgrammaticEmission,
    TreasuryAuthorizedEmission,
    MintDisabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintAuthority {
    ProtocolOnly,
    Governance,
    Issuer,
    Treasury,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryStatus {
    Proposed,
    Registered,
    Active,
    Frozen,
    Deprecated,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskGrade {
    Core,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegistryEntry {
    pub asset_id: [u8; 32],
    pub asset_code: String,
    pub asset_class: AssetClass,
    pub issuer_id: [u8; 32],
    pub display_name: String,
    pub symbol: String,
    pub decimals: u8,
    pub supply_model: SupplyModel,
    pub max_supply: Option<u128>,
    pub mint_authority: MintAuthority,
    pub metadata_hash: [u8; 32],
    pub policy_hash: [u8; 32],
    pub risk_grade: RiskGrade,
    pub status: RegistryStatus,
    pub created_at_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssetRegistryError {
    #[error("asset code must follow AOXC-[CLASS]-[POLICY]-[SERIES] format")]
    InvalidAssetCode,
    #[error("asset symbol must be uppercase ASCII and 2..=12 chars")]
    InvalidSymbol,
    #[error("asset decimals must be <= 18")]
    InvalidDecimals,
    #[error("issuer id must not be zero")]
    ZeroIssuer,
    #[error("metadata hash must not be zero")]
    ZeroMetadataHash,
    #[error("policy hash must not be zero")]
    ZeroPolicyHash,
    #[error("max supply must be present for fixed genesis model")]
    MissingFixedSupply,
    #[error("mint disabled model must use protocol-only authority")]
    InvalidMintAuthorityForMintDisabled,
}

impl AssetRegistryEntry {
    pub fn validate(&self) -> Result<(), AssetRegistryError> {
        validate_asset_code(&self.asset_code)?;
        validate_symbol(&self.symbol)?;

        if self.decimals > 18 {
            return Err(AssetRegistryError::InvalidDecimals);
        }
        if self.issuer_id == [0u8; 32] {
            return Err(AssetRegistryError::ZeroIssuer);
        }
        if self.metadata_hash == [0u8; 32] {
            return Err(AssetRegistryError::ZeroMetadataHash);
        }
        if self.policy_hash == [0u8; 32] {
            return Err(AssetRegistryError::ZeroPolicyHash);
        }

        if self.supply_model == SupplyModel::FixedGenesis && self.max_supply.is_none() {
            return Err(AssetRegistryError::MissingFixedSupply);
        }

        if self.supply_model == SupplyModel::MintDisabled
            && self.mint_authority != MintAuthority::ProtocolOnly
        {
            return Err(AssetRegistryError::InvalidMintAuthorityForMintDisabled);
        }

        Ok(())
    }
}

fn validate_asset_code(value: &str) -> Result<(), AssetRegistryError> {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 4 || parts[0] != "AOXC" {
        return Err(AssetRegistryError::InvalidAssetCode);
    }
    if parts[1].is_empty() || parts[2].is_empty() || parts[3].is_empty() {
        return Err(AssetRegistryError::InvalidAssetCode);
    }
    Ok(())
}

fn validate_symbol(value: &str) -> Result<(), AssetRegistryError> {
    if !(2..=12).contains(&value.len()) {
        return Err(AssetRegistryError::InvalidSymbol);
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err(AssetRegistryError::InvalidSymbol);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AssetRegistryEntry {
        AssetRegistryEntry {
            asset_id: [1u8; 32],
            asset_code: "AOXC-REG-UTIL-0101".to_string(),
            asset_class: AssetClass::Registered,
            issuer_id: [2u8; 32],
            display_name: "AOXC Utility".to_string(),
            symbol: "AUX".to_string(),
            decimals: 18,
            supply_model: SupplyModel::GovernedEmission,
            max_supply: None,
            mint_authority: MintAuthority::Governance,
            metadata_hash: [3u8; 32],
            policy_hash: [4u8; 32],
            risk_grade: RiskGrade::Medium,
            status: RegistryStatus::Registered,
            created_at_epoch: 1,
        }
    }

    #[test]
    fn valid_registry_entry_passes() {
        let entry = sample();
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn invalid_asset_code_is_rejected() {
        let mut entry = sample();
        entry.asset_code = "INVALID".to_string();
        assert_eq!(
            entry.validate().unwrap_err(),
            AssetRegistryError::InvalidAssetCode
        );
    }

    #[test]
    fn fixed_genesis_requires_max_supply() {
        let mut entry = sample();
        entry.supply_model = SupplyModel::FixedGenesis;
        entry.max_supply = None;
        assert_eq!(
            entry.validate().unwrap_err(),
            AssetRegistryError::MissingFixedSupply
        );
    }
}
