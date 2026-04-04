use serde::{Deserialize, Serialize};

use super::{
    AssetClass, AssetCode, AssetId, AssetRegistryError, AssetSymbol, DisplayName, IssuerId,
    MAX_DECIMALS, MintAuthority, NonZeroHash32, RegistryStatus, RiskGrade, SupplyModel,
};

/// Immutable, validated asset registry record.
///
/// Construction model:
/// This type is only created through validated constructors or validated
/// transitions. The goal is to keep invalid states unrepresentable wherever
/// reasonably possible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegistryEntry {
    pub asset_id: AssetId,
    pub asset_code: AssetCode,
    pub display_name: DisplayName,
    pub symbol: AssetSymbol,
    pub decimals: u8,
    pub asset_class: AssetClass,
    pub supply_model: SupplyModel,
    pub mint_authority: MintAuthority,
    pub issuer: IssuerId,
    pub max_supply: Option<u128>,
    pub registry_status: RegistryStatus,
    pub risk_grade: RiskGrade,
    pub metadata_hash: NonZeroHash32,
    pub policy_hash: NonZeroHash32,
    pub created_at_epoch: u64,
}

impl AssetRegistryEntry {
    /// Creates a fully validated asset registry record.
    ///
    /// Audit note:
    /// This constructor performs strict validation before admitting the record.
    /// The registry intentionally rejects ambiguous or partially consistent
    /// configurations rather than attempting downstream correction.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        asset_id: [u8; 32],
        asset_code: impl Into<String>,
        display_name: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
        asset_class: AssetClass,
        supply_model: SupplyModel,
        mint_authority: MintAuthority,
        issuer: [u8; 32],
        max_supply: Option<u128>,
        registry_status: RegistryStatus,
        risk_grade: RiskGrade,
        metadata_hash: [u8; 32],
        policy_hash: [u8; 32],
        created_at_epoch: u64,
    ) -> Result<Self, AssetRegistryError> {
        let asset_id = AssetId::new(asset_id)?;
        let issuer = IssuerId::new(issuer)?;
        let metadata_hash =
            NonZeroHash32::new(metadata_hash, AssetRegistryError::ZeroMetadataHash)?;
        let policy_hash = NonZeroHash32::new(policy_hash, AssetRegistryError::ZeroPolicyHash)?;
        let display_name = DisplayName::new(display_name)?;
        let symbol = AssetSymbol::new(symbol)?;
        let asset_code = AssetCode::new(asset_code, asset_class, supply_model)?;

        let entry = Self {
            asset_id,
            asset_code,
            display_name,
            symbol,
            decimals,
            asset_class,
            supply_model,
            mint_authority,
            issuer,
            max_supply,
            registry_status,
            risk_grade,
            metadata_hash,
            policy_hash,
            created_at_epoch,
        };

        entry.validate()?;
        Ok(entry)
    }

    /// Re-validates the record against all field and cross-field invariants.
    ///
    /// Audit note:
    /// Although the type is designed to be valid at construction time, this
    /// method remains useful for deserialization boundaries, migration checks,
    /// and defense-in-depth.
    pub fn validate(&self) -> Result<(), AssetRegistryError> {
        self.validate_numeric_policy()?;
        self.validate_supply_policy()?;
        self.validate_class_policy()?;
        self.validate_status_policy()?;
        Ok(())
    }

    /// Returns whether the current status may transition to `next`.
    ///
    /// State machine:
    /// - Proposed   -> Registered | Revoked
    /// - Registered -> Active | Revoked
    /// - Active     -> Frozen | Deprecated | Revoked
    /// - Frozen     -> Active | Deprecated | Revoked
    /// - Deprecated -> Revoked
    /// - Revoked    -> terminal
    pub fn can_transition_to(&self, next: RegistryStatus) -> bool {
        match self.registry_status {
            RegistryStatus::Proposed => {
                matches!(next, RegistryStatus::Registered | RegistryStatus::Revoked)
            }
            RegistryStatus::Registered => {
                matches!(next, RegistryStatus::Active | RegistryStatus::Revoked)
            }
            RegistryStatus::Active => matches!(
                next,
                RegistryStatus::Frozen | RegistryStatus::Deprecated | RegistryStatus::Revoked
            ),
            RegistryStatus::Frozen => matches!(
                next,
                RegistryStatus::Active | RegistryStatus::Deprecated | RegistryStatus::Revoked
            ),
            RegistryStatus::Deprecated => matches!(next, RegistryStatus::Revoked),
            RegistryStatus::Revoked => false,
        }
    }

    /// Produces a new validated record with an updated lifecycle status.
    ///
    /// Audit note:
    /// The original record remains unchanged. Transition validation is applied
    /// before the new state is admitted.
    pub fn transition_to(&self, next: RegistryStatus) -> Result<Self, AssetRegistryError> {
        if !self.can_transition_to(next) {
            return Err(AssetRegistryError::InvalidStatusTransition {
                from: self.registry_status,
                to: next,
            });
        }

        let updated = Self {
            registry_status: next,
            ..self.clone()
        };

        updated.validate_status_policy()?;
        Ok(updated)
    }

    /// Produces a new validated record with an updated risk grade.
    ///
    /// Audit note:
    /// Risk changes are allowed only if they remain consistent with class-level
    /// policy constraints.
    pub fn with_risk_grade(&self, risk_grade: RiskGrade) -> Result<Self, AssetRegistryError> {
        let updated = Self {
            risk_grade,
            ..self.clone()
        };

        updated.validate_class_policy()?;
        Ok(updated)
    }

    fn validate_numeric_policy(&self) -> Result<(), AssetRegistryError> {
        if self.decimals > MAX_DECIMALS {
            return Err(AssetRegistryError::InvalidDecimals {
                provided: self.decimals,
                maximum: MAX_DECIMALS,
            });
        }

        if self.created_at_epoch == 0 {
            return Err(AssetRegistryError::InvalidCreatedAtEpoch);
        }

        Ok(())
    }

    fn validate_supply_policy(&self) -> Result<(), AssetRegistryError> {
        match self.supply_model {
            SupplyModel::FixedGenesis => {
                let max_supply =
                    self.max_supply
                        .ok_or(AssetRegistryError::MissingMaxSupplyForSupplyModel {
                            supply_model: self.supply_model,
                        })?;

                if max_supply == 0 {
                    return Err(AssetRegistryError::ZeroMaxSupply);
                }

                if self.mint_authority != MintAuthority::ProtocolOnly {
                    return Err(AssetRegistryError::MintAuthorityMismatch {
                        supply_model: self.supply_model,
                        mint_authority: self.mint_authority,
                    });
                }
            }
            SupplyModel::TreasuryAuthorizedEmission => {
                if self.mint_authority != MintAuthority::Treasury {
                    return Err(AssetRegistryError::MintAuthorityMismatch {
                        supply_model: self.supply_model,
                        mint_authority: self.mint_authority,
                    });
                }

                if let Some(max_supply) = self.max_supply
                    && max_supply == 0
                {
                    return Err(AssetRegistryError::ZeroMaxSupply);
                }
            }
            SupplyModel::GovernanceAuthorizedEmission => {
                if self.mint_authority != MintAuthority::Governance {
                    return Err(AssetRegistryError::MintAuthorityMismatch {
                        supply_model: self.supply_model,
                        mint_authority: self.mint_authority,
                    });
                }

                if let Some(max_supply) = self.max_supply
                    && max_supply == 0
                {
                    return Err(AssetRegistryError::ZeroMaxSupply);
                }
            }
            SupplyModel::ProgrammaticEmission => {
                if self.mint_authority != MintAuthority::ProtocolOnly {
                    return Err(AssetRegistryError::MintAuthorityMismatch {
                        supply_model: self.supply_model,
                        mint_authority: self.mint_authority,
                    });
                }

                if self.max_supply.is_some() {
                    return Err(AssetRegistryError::UnexpectedMaxSupplyForSupplyModel {
                        supply_model: self.supply_model,
                    });
                }
            }
            SupplyModel::WrappedBacked => {
                if self.mint_authority != MintAuthority::Bridge {
                    return Err(AssetRegistryError::MintAuthorityMismatch {
                        supply_model: self.supply_model,
                        mint_authority: self.mint_authority,
                    });
                }

                if self.max_supply.is_some() {
                    return Err(AssetRegistryError::UnexpectedMaxSupplyForSupplyModel {
                        supply_model: self.supply_model,
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_class_policy(&self) -> Result<(), AssetRegistryError> {
        match self.asset_class {
            AssetClass::Native => {
                if self.supply_model != SupplyModel::FixedGenesis {
                    return Err(AssetRegistryError::InvalidSupplyModelForAssetClass {
                        asset_class: self.asset_class,
                        supply_model: self.supply_model,
                    });
                }

                if self.mint_authority != MintAuthority::ProtocolOnly {
                    return Err(AssetRegistryError::InvalidMintAuthorityForAssetClass {
                        asset_class: self.asset_class,
                        mint_authority: self.mint_authority,
                    });
                }

                if self.risk_grade != RiskGrade::Low {
                    return Err(AssetRegistryError::InvalidRiskGradeForAssetClass {
                        asset_class: self.asset_class,
                        risk_grade: self.risk_grade,
                    });
                }
            }
            AssetClass::Constitutional | AssetClass::System => {
                if self.mint_authority != MintAuthority::ProtocolOnly {
                    return Err(AssetRegistryError::InvalidMintAuthorityForAssetClass {
                        asset_class: self.asset_class,
                        mint_authority: self.mint_authority,
                    });
                }

                if self.risk_grade == RiskGrade::Critical {
                    return Err(AssetRegistryError::InvalidRiskGradeForAssetClass {
                        asset_class: self.asset_class,
                        risk_grade: self.risk_grade,
                    });
                }
            }
            AssetClass::Treasury => {
                if !matches!(
                    self.mint_authority,
                    MintAuthority::Treasury | MintAuthority::ProtocolOnly
                ) {
                    return Err(AssetRegistryError::InvalidMintAuthorityForAssetClass {
                        asset_class: self.asset_class,
                        mint_authority: self.mint_authority,
                    });
                }
            }
            AssetClass::Governance => {
                if !matches!(
                    self.mint_authority,
                    MintAuthority::Governance | MintAuthority::ProtocolOnly
                ) {
                    return Err(AssetRegistryError::InvalidMintAuthorityForAssetClass {
                        asset_class: self.asset_class,
                        mint_authority: self.mint_authority,
                    });
                }
            }
            AssetClass::Wrapped => {
                if self.supply_model != SupplyModel::WrappedBacked {
                    return Err(AssetRegistryError::InvalidSupplyModelForAssetClass {
                        asset_class: self.asset_class,
                        supply_model: self.supply_model,
                    });
                }

                if self.mint_authority != MintAuthority::Bridge {
                    return Err(AssetRegistryError::InvalidMintAuthorityForAssetClass {
                        asset_class: self.asset_class,
                        mint_authority: self.mint_authority,
                    });
                }
            }
            AssetClass::Experimental => {
                if self.risk_grade == RiskGrade::Low {
                    return Err(AssetRegistryError::InvalidRiskGradeForAssetClass {
                        asset_class: self.asset_class,
                        risk_grade: self.risk_grade,
                    });
                }
            }
            AssetClass::Utility | AssetClass::Synthetic => {}
        }

        Ok(())
    }

    fn validate_status_policy(&self) -> Result<(), AssetRegistryError> {
        if self.asset_class == AssetClass::Native
            && matches!(
                self.registry_status,
                RegistryStatus::Proposed | RegistryStatus::Registered
            )
        {
            return Err(AssetRegistryError::InvalidStatusForAssetClass {
                asset_class: self.asset_class,
                status: self.registry_status,
            });
        }

        Ok(())
    }
}
