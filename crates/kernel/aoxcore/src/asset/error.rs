use core::fmt;

use super::{AssetClass, MintAuthority, RegistryStatus, RiskGrade, SupplyModel};

/// Detailed error surface for registry validation.
///
/// Audit note:
/// Errors intentionally carry structured context. This improves operator
/// diagnosis, test fidelity, and forensic clarity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetRegistryError {
    EmptyDisplayName,
    InvalidAssetCodeFormat,
    InvalidAssetCodeNamespace,
    InvalidAssetCodeClassSegment {
        expected: &'static str,
        found: String,
    },
    InvalidAssetCodePolicySegment {
        expected: &'static str,
        found: String,
    },
    InvalidAssetCodeSequenceSegment {
        found: String,
    },
    InvalidSymbolLength,
    InvalidSymbolFormat,
    ZeroAssetId,
    ZeroIssuer,
    ZeroMetadataHash,
    ZeroPolicyHash,
    InvalidDecimals {
        provided: u8,
        maximum: u8,
    },
    MissingMaxSupplyForSupplyModel {
        supply_model: SupplyModel,
    },
    ZeroMaxSupply,
    UnexpectedMaxSupplyForSupplyModel {
        supply_model: SupplyModel,
    },
    MintAuthorityMismatch {
        supply_model: SupplyModel,
        mint_authority: MintAuthority,
    },
    InvalidSupplyModelForAssetClass {
        asset_class: AssetClass,
        supply_model: SupplyModel,
    },
    InvalidMintAuthorityForAssetClass {
        asset_class: AssetClass,
        mint_authority: MintAuthority,
    },
    InvalidRiskGradeForAssetClass {
        asset_class: AssetClass,
        risk_grade: RiskGrade,
    },
    InvalidStatusForAssetClass {
        asset_class: AssetClass,
        status: RegistryStatus,
    },
    InvalidCreatedAtEpoch,
    InvalidStatusTransition {
        from: RegistryStatus,
        to: RegistryStatus,
    },
}

impl fmt::Display for AssetRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDisplayName => write!(f, "display name must not be empty"),
            Self::InvalidAssetCodeFormat => write!(f, "asset code format is invalid"),
            Self::InvalidAssetCodeNamespace => write!(f, "asset code namespace is invalid"),
            Self::InvalidAssetCodeClassSegment { expected, found } => {
                write!(
                    f,
                    "asset code class segment is invalid: expected {}, found {}",
                    expected, found
                )
            }
            Self::InvalidAssetCodePolicySegment { expected, found } => {
                write!(
                    f,
                    "asset code policy segment is invalid: expected {}, found {}",
                    expected, found
                )
            }
            Self::InvalidAssetCodeSequenceSegment { found } => {
                write!(f, "asset code sequence segment is invalid: {}", found)
            }
            Self::InvalidSymbolLength => write!(f, "symbol length is invalid"),
            Self::InvalidSymbolFormat => write!(f, "symbol format is invalid"),
            Self::ZeroAssetId => write!(f, "asset id must not be zero"),
            Self::ZeroIssuer => write!(f, "issuer must not be zero"),
            Self::ZeroMetadataHash => write!(f, "metadata hash must not be zero"),
            Self::ZeroPolicyHash => write!(f, "policy hash must not be zero"),
            Self::InvalidDecimals { provided, maximum } => write!(
                f,
                "decimals exceeds permitted range: provided {}, maximum {}",
                provided, maximum
            ),
            Self::MissingMaxSupplyForSupplyModel { supply_model } => {
                write!(
                    f,
                    "max supply is required for supply model {:?}",
                    supply_model
                )
            }
            Self::ZeroMaxSupply => write!(f, "max supply must be greater than zero"),
            Self::UnexpectedMaxSupplyForSupplyModel { supply_model } => write!(
                f,
                "max supply must not be configured for supply model {:?}",
                supply_model
            ),
            Self::MintAuthorityMismatch {
                supply_model,
                mint_authority,
            } => write!(
                f,
                "mint authority {:?} is incompatible with supply model {:?}",
                mint_authority, supply_model
            ),
            Self::InvalidSupplyModelForAssetClass {
                asset_class,
                supply_model,
            } => write!(
                f,
                "supply model {:?} is incompatible with asset class {:?}",
                supply_model, asset_class
            ),
            Self::InvalidMintAuthorityForAssetClass {
                asset_class,
                mint_authority,
            } => write!(
                f,
                "mint authority {:?} is incompatible with asset class {:?}",
                mint_authority, asset_class
            ),
            Self::InvalidRiskGradeForAssetClass {
                asset_class,
                risk_grade,
            } => write!(
                f,
                "risk grade {:?} is incompatible with asset class {:?}",
                risk_grade, asset_class
            ),
            Self::InvalidStatusForAssetClass {
                asset_class,
                status,
            } => write!(
                f,
                "registry status {:?} is incompatible with asset class {:?}",
                status, asset_class
            ),
            Self::InvalidCreatedAtEpoch => write!(f, "created_at_epoch is invalid"),
            Self::InvalidStatusTransition { from, to } => {
                write!(
                    f,
                    "registry status transition is invalid: {:?} -> {:?}",
                    from, to
                )
            }
        }
    }
}

impl std::error::Error for AssetRegistryError {}
