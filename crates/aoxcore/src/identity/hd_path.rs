// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Canonical BIP44 purpose for AOXC paths.
/// m/44/2626/...
pub const AOXC_HD_BIP44_PURPOSE: u32 = 44;

/// Canonical AOXC coin type under the BIP44-style path.
/// m/44/2626/...
pub const AOXC_HD_PURPOSE: u32 = 2626;

/// Maximum allowed canonical component value.
///
/// AOXC canonical textual HD paths intentionally store unhardened variable
/// components only. Hardened derivation is represented as a projection via
/// `HARDENED_OFFSET` rather than as persisted canonical path text.
pub const MAX_HD_INDEX: u32 = 0x7FFF_FFFF;

/// Hardened offset used by many HD derivation schemes.
pub const HARDENED_OFFSET: u32 = 0x8000_0000;

/// Total number of slash-separated components in a canonical AOXC HD path.
const HD_PATH_PART_COUNT: usize = 7;

/// Root marker used by canonical AOXC HD path strings.
const HD_PATH_ROOT: &str = "m";

/// Canonical AOXC HD path:
///
/// m / 44 / 2626 / chain / role / zone / index
///
/// Example:
///
/// m/44/2626/1/1/2/0
///
/// Components:
///
/// m        -> root
/// 44       -> BIP44 purpose
/// 2626     -> AOXC coin type
/// chain    -> chain identifier
/// role     -> actor role identifier
/// zone     -> geographic / logical zone
/// index    -> sequential key index
///
/// Design notes:
/// - canonical AOXC path text is intentionally unhardened,
/// - hardened derivation remains available through projection helpers,
/// - all variable components are bounded to the unhardened range.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HdPath {
    pub chain: u32,
    pub role: u32,
    pub zone: u32,
    pub index: u32,
}

/// Canonical error surface for AOXC HD path parsing and validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HdPathError {
    EmptyInput,
    InvalidFormat,
    InvalidPurpose,
    InvalidComponent,
    ComponentOverflow,
    IndexOverflow,
}

impl HdPathError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "HD_PATH_EMPTY_INPUT",
            Self::InvalidFormat => "HD_PATH_INVALID_FORMAT",
            Self::InvalidPurpose => "HD_PATH_INVALID_PURPOSE",
            Self::InvalidComponent => "HD_PATH_INVALID_COMPONENT",
            Self::ComponentOverflow => "HD_PATH_COMPONENT_OVERFLOW",
            Self::IndexOverflow => "HD_PATH_INDEX_OVERFLOW",
        }
    }
}

impl fmt::Display for HdPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "HD_PATH_EMPTY_INPUT"),
            Self::InvalidFormat => write!(f, "HD_PATH_INVALID_FORMAT"),
            Self::InvalidPurpose => write!(f, "HD_PATH_INVALID_PURPOSE"),
            Self::InvalidComponent => write!(f, "HD_PATH_INVALID_COMPONENT"),
            Self::ComponentOverflow => write!(f, "HD_PATH_COMPONENT_OVERFLOW"),
            Self::IndexOverflow => write!(f, "HD_PATH_INDEX_OVERFLOW"),
        }
    }
}

impl std::error::Error for HdPathError {}

impl HdPath {
    /// Creates a new canonical AOXC HD path with validation.
    ///
    /// Validation policy:
    /// - all variable components must remain within the canonical unhardened range,
    /// - `index` has a dedicated overflow error to preserve compatibility with
    ///   existing call sites and tests.
    pub fn new(chain: u32, role: u32, zone: u32, index: u32) -> Result<Self, HdPathError> {
        validate_variable_component(chain)?;
        validate_variable_component(role)?;
        validate_variable_component(zone)?;

        if index > MAX_HD_INDEX {
            return Err(HdPathError::IndexOverflow);
        }

        Ok(Self {
            chain,
            role,
            zone,
            index,
        })
    }

    /// Validates an already-constructed path instance.
    pub fn validate(&self) -> Result<(), HdPathError> {
        Self::new(self.chain, self.role, self.zone, self.index).map(|_| ())
    }

    /// Returns canonical string representation.
    ///
    /// Example:
    ///
    /// m/44/2626/1/1/2/0
    #[must_use]
    pub fn to_string_path(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}/{}/{}",
            HD_PATH_ROOT,
            AOXC_HD_BIP44_PURPOSE,
            AOXC_HD_PURPOSE,
            self.chain,
            self.role,
            self.zone,
            self.index
        )
    }

    /// Returns the index projected into hardened derivation space.
    ///
    /// This helper does not mutate the canonical stored path. It only returns
    /// the hardened numeric projection of the `index` component.
    #[must_use]
    pub fn hardened_index(&self) -> u32 {
        self.index | HARDENED_OFFSET
    }

    /// Returns whether the stored `index` already carries the hardened bit.
    ///
    /// Canonical AOXC paths created via `new()` or `FromStr` are intentionally
    /// unhardened, so this method is primarily useful for defensive inspection
    /// of externally constructed or deserialized values.
    #[must_use]
    pub fn is_hardened(&self) -> bool {
        (self.index & HARDENED_OFFSET) != 0
    }

    /// Returns true if the path is canonical and fully unhardened.
    #[must_use]
    pub fn is_canonical_unhardened(&self) -> bool {
        self.chain <= MAX_HD_INDEX
            && self.role <= MAX_HD_INDEX
            && self.zone <= MAX_HD_INDEX
            && self.index <= MAX_HD_INDEX
    }

    /// Returns the next sequential path.
    pub fn next(&self) -> Result<Self, HdPathError> {
        if self.index == MAX_HD_INDEX {
            return Err(HdPathError::IndexOverflow);
        }

        Ok(Self {
            chain: self.chain,
            role: self.role,
            zone: self.zone,
            index: self.index + 1,
        })
    }

    /// Returns the path shifted by `step` indexes.
    ///
    /// This helper provides a deterministic checked advance without forcing
    /// callers to iterate repeatedly when reserving multiple child paths.
    pub fn next_n(&self, step: u32) -> Result<Self, HdPathError> {
        let next_index = self
            .index
            .checked_add(step)
            .ok_or(HdPathError::IndexOverflow)?;

        if next_index > MAX_HD_INDEX {
            return Err(HdPathError::IndexOverflow);
        }

        Ok(Self {
            chain: self.chain,
            role: self.role,
            zone: self.zone,
            index: next_index,
        })
    }
}

impl FromStr for HdPath {
    type Err = HdPathError;

    /// Parses canonical AOXC HD path string.
    ///
    /// Example:
    ///
    /// m/44/2626/1/1/2/0
    ///
    /// Parsing policy:
    /// - surrounding whitespace is rejected,
    /// - the path must match the canonical AOXC root and purpose values,
    /// - variable components must be decimal numeric values in canonical range.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err(HdPathError::EmptyInput);
        }

        if value.trim().is_empty() {
            return Err(HdPathError::EmptyInput);
        }

        if value != value.trim() {
            return Err(HdPathError::InvalidFormat);
        }

        let parts: Vec<&str> = value.split('/').collect();

        if parts.len() != HD_PATH_PART_COUNT {
            return Err(HdPathError::InvalidFormat);
        }

        if parts[0] != HD_PATH_ROOT {
            return Err(HdPathError::InvalidFormat);
        }

        let bip44_purpose = parse_path_component(parts[1])?;
        if bip44_purpose != AOXC_HD_BIP44_PURPOSE {
            return Err(HdPathError::InvalidPurpose);
        }

        let purpose = parse_path_component(parts[2])?;
        if purpose != AOXC_HD_PURPOSE {
            return Err(HdPathError::InvalidPurpose);
        }

        let chain = parse_variable_component(parts[3])?;
        let role = parse_variable_component(parts[4])?;
        let zone = parse_variable_component(parts[5])?;
        let index = parse_index_component(parts[6])?;

        HdPath::new(chain, role, zone, index)
    }
}

impl fmt::Display for HdPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_path())
    }
}

/// Parses a canonical decimal path component.
fn parse_path_component(value: &str) -> Result<u32, HdPathError> {
    if value.is_empty() {
        return Err(HdPathError::InvalidComponent);
    }

    if !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(HdPathError::InvalidComponent);
    }

    value.parse().map_err(|_| HdPathError::InvalidComponent)
}

/// Parses and validates a non-index variable component.
fn parse_variable_component(value: &str) -> Result<u32, HdPathError> {
    let parsed = parse_path_component(value)?;
    validate_variable_component(parsed)?;
    Ok(parsed)
}

/// Parses and validates the index component.
fn parse_index_component(value: &str) -> Result<u32, HdPathError> {
    let parsed = parse_path_component(value)?;

    if parsed > MAX_HD_INDEX {
        return Err(HdPathError::IndexOverflow);
    }

    Ok(parsed)
}

/// Validates a canonical non-index variable component.
///
/// Current policy:
/// - AOXC canonical textual paths do not allow hardened variable components,
/// - values must remain within the unhardened range.
fn validate_variable_component(value: u32) -> Result<(), HdPathError> {
    if value > MAX_HD_INDEX {
        return Err(HdPathError::ComponentOverflow);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};

    #[test]
    fn hd_path_creation() {
        let path = HdPath::new(1, 2, 3, 0).unwrap();
        assert_eq!(path.chain, 1);
        assert_eq!(path.role, 2);
        assert_eq!(path.zone, 3);
        assert_eq!(path.index, 0);
    }

    #[test]
    fn hd_path_string() {
        let path = HdPath::new(1, 2, 3, 4).unwrap();
        assert_eq!(path.to_string(), "m/44/2626/1/2/3/4");
    }

    #[test]
    fn hd_path_parse() {
        let path: HdPath = "m/44/2626/1/2/3/4".parse().unwrap();
        assert_eq!(path.index, 4);
    }

    #[test]
    fn hd_path_next() {
        let path = HdPath::new(1, 1, 1, 0).unwrap();
        let next = path.next().unwrap();
        assert_eq!(next.index, 1);
    }

    #[test]
    fn hd_path_next_n() {
        let path = HdPath::new(1, 1, 1, 5).unwrap();
        let next = path.next_n(10).unwrap();
        assert_eq!(next.index, 15);
    }

    #[test]
    fn hardened_index_projection_sets_hardened_bit() {
        let path = HdPath::new(1, 1, 1, 0).unwrap();
        assert_eq!(path.hardened_index(), HARDENED_OFFSET);
    }

    #[test]
    fn canonical_paths_are_unhardened() {
        let path = HdPath::new(1, 1, 1, 0).unwrap();
        assert!(!path.is_hardened());
        assert!(path.is_canonical_unhardened());
    }

    #[test]
    fn new_rejects_out_of_range_index() {
        assert_eq!(
            HdPath::new(1, 1, 1, MAX_HD_INDEX + 1),
            Err(HdPathError::IndexOverflow)
        );
    }

    #[test]
    fn new_rejects_out_of_range_variable_components() {
        assert_eq!(
            HdPath::new(MAX_HD_INDEX + 1, 1, 1, 1),
            Err(HdPathError::ComponentOverflow)
        );
        assert_eq!(
            HdPath::new(1, MAX_HD_INDEX + 1, 1, 1),
            Err(HdPathError::ComponentOverflow)
        );
        assert_eq!(
            HdPath::new(1, 1, MAX_HD_INDEX + 1, 1),
            Err(HdPathError::ComponentOverflow)
        );
    }

    #[test]
    fn parse_rejects_invalid_shapes_and_prefixes() {
        assert_eq!("".parse::<HdPath>(), Err(HdPathError::EmptyInput));
        assert_eq!("   ".parse::<HdPath>(), Err(HdPathError::EmptyInput));
        assert_eq!(
            "m/44/2626/1/2/3".parse::<HdPath>(),
            Err(HdPathError::InvalidFormat)
        );
        assert_eq!(
            "root/44/2626/1/2/3/4".parse::<HdPath>(),
            Err(HdPathError::InvalidFormat)
        );
    }

    #[test]
    fn parse_rejects_surrounding_whitespace() {
        assert_eq!(
            " m/44/2626/1/2/3/4 ".parse::<HdPath>(),
            Err(HdPathError::InvalidFormat)
        );
    }

    #[test]
    fn parse_rejects_wrong_purpose_values() {
        assert_eq!(
            "m/43/2626/1/2/3/4".parse::<HdPath>(),
            Err(HdPathError::InvalidPurpose)
        );
        assert_eq!(
            "m/44/9999/1/2/3/4".parse::<HdPath>(),
            Err(HdPathError::InvalidPurpose)
        );
    }

    #[test]
    fn parse_rejects_non_numeric_components() {
        assert_eq!(
            "m/44/2626/x/2/3/4".parse::<HdPath>(),
            Err(HdPathError::InvalidComponent)
        );
        assert_eq!(
            "m/44/2626/1/x/3/4".parse::<HdPath>(),
            Err(HdPathError::InvalidComponent)
        );
        assert_eq!(
            "m/44/2626/1/2/x/4".parse::<HdPath>(),
            Err(HdPathError::InvalidComponent)
        );
        assert_eq!(
            "m/44/2626/1/2/3/x".parse::<HdPath>(),
            Err(HdPathError::InvalidComponent)
        );
    }

    #[test]
    fn parse_rejects_component_overflow() {
        let overflow_path = format!("m/44/2626/{}/2/3/4", MAX_HD_INDEX + 1);
        assert_eq!(
            overflow_path.parse::<HdPath>(),
            Err(HdPathError::ComponentOverflow)
        );
    }

    #[test]
    fn parse_rejects_index_overflow() {
        let overflow_path = format!("m/44/2626/1/2/3/{}", MAX_HD_INDEX + 1);
        assert_eq!(
            overflow_path.parse::<HdPath>(),
            Err(HdPathError::IndexOverflow)
        );
    }

    #[test]
    fn next_rejects_max_index() {
        let path = HdPath::new(10, 20, 30, MAX_HD_INDEX).unwrap();
        assert_eq!(path.next(), Err(HdPathError::IndexOverflow));
    }

    #[test]
    fn next_n_rejects_overflow() {
        let path = HdPath::new(1, 1, 1, MAX_HD_INDEX - 1).unwrap();
        assert_eq!(path.next_n(2), Err(HdPathError::IndexOverflow));
    }

    #[test]
    fn randomized_roundtrip_stress_for_canonical_paths() {
        let mut rng = StdRng::seed_from_u64(0xA0C0_2026_u64);

        for _ in 0..2_000 {
            let chain = rng.next_u32() & MAX_HD_INDEX;
            let role = rng.next_u32() & MAX_HD_INDEX;
            let zone = rng.next_u32() & MAX_HD_INDEX;
            let index = rng.next_u32() & MAX_HD_INDEX;

            let original = HdPath::new(chain, role, zone, index).unwrap();
            let serialized = original.to_string();
            let parsed: HdPath = serialized.parse().unwrap();

            assert_eq!(parsed, original);
        }
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(HdPathError::EmptyInput.code(), "HD_PATH_EMPTY_INPUT");
        assert_eq!(HdPathError::InvalidFormat.code(), "HD_PATH_INVALID_FORMAT");
        assert_eq!(HdPathError::InvalidPurpose.code(), "HD_PATH_INVALID_PURPOSE");
        assert_eq!(HdPathError::InvalidComponent.code(), "HD_PATH_INVALID_COMPONENT");
        assert_eq!(HdPathError::ComponentOverflow.code(), "HD_PATH_COMPONENT_OVERFLOW");
        assert_eq!(HdPathError::IndexOverflow.code(), "HD_PATH_INDEX_OVERFLOW");
    }
}
