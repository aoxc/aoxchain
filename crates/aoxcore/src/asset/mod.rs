use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

/// Maximum supported decimal precision for registry assets.
///
/// Rationale:
/// The registry deliberately caps decimal precision to reduce downstream
/// formatting ambiguity and avoid excessive precision assumptions across
/// execution, accounting, and client surfaces.
pub const MAX_DECIMALS: u8 = 18;

/// Required namespace prefix for canonical AOXC asset codes.
pub const ASSET_CODE_NAMESPACE: &str = "AOXC";

/// Canonical length of the sequence segment in the asset code.
pub const ASSET_CODE_SEQUENCE_LEN: usize = 4;

/// Strongly typed identifier for an asset registry record.
///
/// Security rationale:
/// A dedicated wrapper prevents accidental use of arbitrary byte arrays in
/// places that require a validated non-zero asset identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId([u8; 32]);

impl AssetId {
    /// Creates a validated asset identifier.
    pub fn new(bytes: [u8; 32]) -> Result<Self, AssetRegistryError> {
        if bytes == [0u8; 32] {
            return Err(AssetRegistryError::ZeroAssetId);
        }
        Ok(Self(bytes))
    }

    /// Returns the underlying canonical byte representation.
    pub fn into_inner(self) -> [u8; 32] {
        self.0
    }

    /// Returns the underlying bytes by reference.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Strongly typed issuer identifier.
///
/// Security rationale:
/// This wrapper makes the authorization-bearing issuer field explicit and
/// guarantees that the record cannot be constructed with an all-zero issuer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssuerId([u8; 32]);

impl IssuerId {
    /// Creates a validated issuer identifier.
    pub fn new(bytes: [u8; 32]) -> Result<Self, AssetRegistryError> {
        if bytes == [0u8; 32] {
            return Err(AssetRegistryError::ZeroIssuer);
        }
        Ok(Self(bytes))
    }

    /// Returns the underlying canonical byte representation.
    pub fn into_inner(self) -> [u8; 32] {
        self.0
    }

    /// Returns the underlying bytes by reference.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Strongly typed non-zero 32-byte hash.
///
/// Security rationale:
/// Metadata and policy commitments are control-bearing fields. A typed hash
/// wrapper ensures that an all-zero placeholder cannot silently pass into the
/// registry as if it were a legitimate commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NonZeroHash32([u8; 32]);

impl NonZeroHash32 {
    /// Creates a validated non-zero hash.
    pub fn new(
        bytes: [u8; 32],
        zero_error: AssetRegistryError,
    ) -> Result<Self, AssetRegistryError> {
        if bytes == [0u8; 32] {
            return Err(zero_error);
        }
        Ok(Self(bytes))
    }

    /// Returns the underlying canonical byte representation.
    pub fn into_inner(self) -> [u8; 32] {
        self.0
    }

    /// Returns the underlying bytes by reference.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Canonical registry display name.
///
/// Security rationale:
/// Display names are consumed by operator interfaces and downstream
/// integrations. Whitespace-only or empty names create ambiguity and should
/// be rejected at construction time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayName(String);

impl DisplayName {
    /// Creates a validated display name.
    pub fn new(value: impl Into<String>) -> Result<Self, AssetRegistryError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AssetRegistryError::EmptyDisplayName);
        }
        Ok(Self(value))
    }

    /// Returns the canonical string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Canonical registry symbol.
///
/// Security rationale:
/// Symbols appear across accounting, UX, and integration layers. The registry
/// enforces a narrow allowed alphabet to prevent ambiguity, spoofability, and
/// mixed-case inconsistencies.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetSymbol(String);

impl AssetSymbol {
    /// Creates a validated symbol.
    pub fn new(value: impl Into<String>) -> Result<Self, AssetRegistryError> {
        let value = value.into();

        if !(2..=12).contains(&value.len()) {
            return Err(AssetRegistryError::InvalidSymbolLength);
        }

        if !value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Err(AssetRegistryError::InvalidSymbolFormat);
        }

        if !value.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            return Err(AssetRegistryError::InvalidSymbolFormat);
        }

        Ok(Self(value))
    }

    /// Returns the canonical symbol string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the underlying symbol string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Canonical AOXC asset code.
///
/// Format:
/// `AOXC.<CLASS>.<POLICY>.<NNNN>`
///
/// Example:
/// `AOXC.UTIL.TREASURY.0001`
///
/// Security rationale:
/// The asset code is treated as a policy-bearing identifier rather than
/// unstructured metadata. It is parsed and validated into its own type to
/// prevent malformed or semantically inconsistent codes from entering the
/// registry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetCode(String);

impl AssetCode {
    /// Creates and validates a canonical asset code against the supplied
    /// class and supply model.
    pub fn new(
        value: impl Into<String>,
        asset_class: AssetClass,
        supply_model: SupplyModel,
    ) -> Result<Self, AssetRegistryError> {
        let value = value.into();
        validate_asset_code(&value, asset_class, supply_model)?;
        Ok(Self(value))
    }

    /// Returns the canonical asset code string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Asset category recognized by the registry.
///
/// Security rationale:
/// The asset class drives policy boundaries, governance expectations, risk
/// treatment, and lifecycle assumptions. This field is therefore a primary
/// policy selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetClass {
    Native,
    Constitutional,
    System,
    Treasury,
    Governance,
    Utility,
    Synthetic,
    Wrapped,
    Experimental,
}

/// Issuance model declared for the asset.
///
/// Security rationale:
/// Supply semantics are protocol-critical. The supply model must remain
/// consistent with mint authority, optional supply caps, and asset class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupplyModel {
    FixedGenesis,
    TreasuryAuthorizedEmission,
    GovernanceAuthorizedEmission,
    ProgrammaticEmission,
    WrappedBacked,
}

/// Authorized minter classification.
///
/// Security rationale:
/// Mint authority defines the authorization boundary for supply expansion and
/// therefore must match the declared supply model and class-level policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MintAuthority {
    ProtocolOnly,
    Treasury,
    Governance,
    Bridge,
}

/// Registry lifecycle state.
///
/// Security rationale:
/// Registry status is an explicit state machine, not a free-form label.
/// Invalid or reversible transitions can cause governance confusion and
/// operational risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegistryStatus {
    Proposed,
    Registered,
    Active,
    Frozen,
    Deprecated,
    Revoked,
}

/// Coarse risk signal used for operator visibility and downstream policy.
///
/// Security rationale:
/// Risk grade does not replace authorization or economic review, but it
/// remains useful for policy gating and should be consistent with class-level
/// expectations where defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskGrade {
    Low,
    Medium,
    High,
    Critical,
}

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

fn validate_asset_code(
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
