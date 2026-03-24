use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateKind {
    Admission,
    Activation,
    Freeze,
    Deprecation,
    Revocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryCertificate {
    pub kind: CertificateKind,
    pub epoch: u64,
    pub policy_hash: [u8; 32],
    pub signer_set_root: [u8; 32],
    pub signature_commitment: [u8; 32],
    pub quorum_weight_bps: u16,
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
    #[error("asset id already exists in registry")]
    DuplicateAssetId,
    #[error("asset symbol already exists in registry")]
    SymbolCollision,
    #[error("asset not found")]
    NotFound,
    #[error("invalid registry state transition")]
    InvalidRegistryTransition,
    #[error("registry certificate policy hash does not match asset policy hash")]
    CertificatePolicyMismatch,
    #[error("registry certificate signer set root is missing")]
    MissingCertificateSignerSet,
    #[error("registry certificate signature commitment is missing")]
    MissingCertificateSignature,
    #[error("registry certificate quorum is below required threshold")]
    InsufficientCertificateQuorum,
    #[error("registry certificate kind does not match operation")]
    CertificateKindMismatch,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetRegistry {
    entries: HashMap<[u8; 32], AssetRegistryEntry>,
    symbol_index: HashMap<String, [u8; 32]>,
}

impl AssetRegistry {
    pub fn propose(&mut self, mut entry: AssetRegistryEntry) -> Result<(), AssetRegistryError> {
        entry.validate()?;
        entry.status = RegistryStatus::Proposed;

        if self.entries.contains_key(&entry.asset_id) {
            return Err(AssetRegistryError::DuplicateAssetId);
        }
        if self.symbol_index.contains_key(&entry.symbol) {
            return Err(AssetRegistryError::SymbolCollision);
        }

        self.symbol_index
            .insert(entry.symbol.clone(), entry.asset_id);
        self.entries.insert(entry.asset_id, entry);
        Ok(())
    }

    pub fn register(
        &mut self,
        asset_id: [u8; 32],
        cert: &RegistryCertificate,
    ) -> Result<(), AssetRegistryError> {
        self.transition(asset_id, RegistryStatus::Registered, cert)
    }

    pub fn activate(
        &mut self,
        asset_id: [u8; 32],
        cert: &RegistryCertificate,
    ) -> Result<(), AssetRegistryError> {
        self.transition(asset_id, RegistryStatus::Active, cert)
    }

    pub fn freeze(
        &mut self,
        asset_id: [u8; 32],
        cert: &RegistryCertificate,
    ) -> Result<(), AssetRegistryError> {
        self.transition(asset_id, RegistryStatus::Frozen, cert)
    }

    pub fn deprecate(
        &mut self,
        asset_id: [u8; 32],
        cert: &RegistryCertificate,
    ) -> Result<(), AssetRegistryError> {
        self.transition(asset_id, RegistryStatus::Deprecated, cert)
    }

    pub fn revoke(
        &mut self,
        asset_id: [u8; 32],
        cert: &RegistryCertificate,
    ) -> Result<(), AssetRegistryError> {
        self.transition(asset_id, RegistryStatus::Revoked, cert)
    }

    #[must_use]
    pub fn get(&self, asset_id: &[u8; 32]) -> Option<&AssetRegistryEntry> {
        self.entries.get(asset_id)
    }

    fn transition(
        &mut self,
        asset_id: [u8; 32],
        next: RegistryStatus,
        cert: &RegistryCertificate,
    ) -> Result<(), AssetRegistryError> {
        let entry = self
            .entries
            .get_mut(&asset_id)
            .ok_or(AssetRegistryError::NotFound)?;

        validate_certificate(entry, next, cert)?;

        if !is_valid_transition(entry.status, next) {
            return Err(AssetRegistryError::InvalidRegistryTransition);
        }

        entry.status = next;
        Ok(())
    }
}

fn is_valid_transition(current: RegistryStatus, next: RegistryStatus) -> bool {
    match (current, next) {
        (RegistryStatus::Proposed, RegistryStatus::Registered)
        | (RegistryStatus::Registered, RegistryStatus::Active)
        | (RegistryStatus::Active, RegistryStatus::Frozen)
        | (RegistryStatus::Active, RegistryStatus::Deprecated)
        | (RegistryStatus::Frozen, RegistryStatus::Active)
        | (RegistryStatus::Frozen, RegistryStatus::Deprecated)
        | (RegistryStatus::Deprecated, RegistryStatus::Revoked)
        | (RegistryStatus::Active, RegistryStatus::Revoked)
        | (RegistryStatus::Frozen, RegistryStatus::Revoked) => true,
        _ if current == next => true,
        _ => false,
    }
}

fn validate_certificate(
    entry: &AssetRegistryEntry,
    next: RegistryStatus,
    cert: &RegistryCertificate,
) -> Result<(), AssetRegistryError> {
    let expected_kind = match next {
        RegistryStatus::Registered => CertificateKind::Admission,
        RegistryStatus::Active => CertificateKind::Activation,
        RegistryStatus::Frozen => CertificateKind::Freeze,
        RegistryStatus::Deprecated => CertificateKind::Deprecation,
        RegistryStatus::Revoked => CertificateKind::Revocation,
        RegistryStatus::Proposed => return Err(AssetRegistryError::CertificateKindMismatch),
    };

    if cert.kind != expected_kind {
        return Err(AssetRegistryError::CertificateKindMismatch);
    }
    if cert.policy_hash != entry.policy_hash {
        return Err(AssetRegistryError::CertificatePolicyMismatch);
    }
    if cert.signer_set_root == [0u8; 32] {
        return Err(AssetRegistryError::MissingCertificateSignerSet);
    }
    if cert.signature_commitment == [0u8; 32] {
        return Err(AssetRegistryError::MissingCertificateSignature);
    }
    if cert.quorum_weight_bps < 6700 {
        return Err(AssetRegistryError::InsufficientCertificateQuorum);
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

    fn cert(kind: CertificateKind, policy_hash: [u8; 32]) -> RegistryCertificate {
        RegistryCertificate {
            kind,
            epoch: 1,
            policy_hash,
            signer_set_root: [8u8; 32],
            signature_commitment: [9u8; 32],
            quorum_weight_bps: 7000,
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

    #[test]
    fn registry_enforces_symbol_uniqueness_and_lifecycle() {
        let mut registry = AssetRegistry::default();
        let entry = sample();
        let id = entry.asset_id;
        let policy_hash = entry.policy_hash;

        registry.propose(entry.clone()).unwrap();
        registry
            .register(id, &cert(CertificateKind::Admission, policy_hash))
            .unwrap();
        registry
            .activate(id, &cert(CertificateKind::Activation, policy_hash))
            .unwrap();
        registry
            .freeze(id, &cert(CertificateKind::Freeze, policy_hash))
            .unwrap();
        registry
            .activate(id, &cert(CertificateKind::Activation, policy_hash))
            .unwrap();
        registry
            .deprecate(id, &cert(CertificateKind::Deprecation, policy_hash))
            .unwrap();
        registry
            .revoke(id, &cert(CertificateKind::Revocation, policy_hash))
            .unwrap();

        let mut collision = entry;
        collision.asset_id = [9u8; 32];
        let err = registry.propose(collision).unwrap_err();
        assert_eq!(err, AssetRegistryError::SymbolCollision);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut registry = AssetRegistry::default();
        let entry = sample();
        let id = entry.asset_id;
        let policy_hash = entry.policy_hash;

        registry.propose(entry).unwrap();
        let err = registry
            .activate(id, &cert(CertificateKind::Activation, policy_hash))
            .unwrap_err();
        assert_eq!(err, AssetRegistryError::InvalidRegistryTransition);
    }

    #[test]
    fn invalid_certificate_quorum_is_rejected() {
        let mut registry = AssetRegistry::default();
        let entry = sample();
        let id = entry.asset_id;
        let policy_hash = entry.policy_hash;
        registry.propose(entry).unwrap();

        let mut weak = cert(CertificateKind::Admission, policy_hash);
        weak.quorum_weight_bps = 5100;
        let err = registry.register(id, &weak).unwrap_err();
        assert_eq!(err, AssetRegistryError::InsufficientCertificateQuorum);
    }
}
