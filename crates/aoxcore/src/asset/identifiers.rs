use serde::{Deserialize, Serialize};

use super::{AssetClass, AssetRegistryError, SupplyModel, code::validate_asset_code};

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
pub struct AssetCode(pub(super) String);

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
