use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Canonical BIP44 purpose for AOXC paths.
/// m/44/2626/...
pub const AOXC_HD_BIP44_PURPOSE: u32 = 44;

/// Canonical AOXC coin type under the BIP44-style path.
/// m/44/2626/...
pub const AOXC_HD_PURPOSE: u32 = 2626;

/// Maximum allowed index value.
pub const MAX_HD_INDEX: u32 = 0x7FFF_FFFF;

/// Hardened offset used by many HD derivation schemes.
pub const HARDENED_OFFSET: u32 = 0x8000_0000;

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
/// m        → root
/// 44       → BIP44 purpose
/// 2626     → AOXC coin type
/// chain    → chain identifier
/// role     → actor role identifier
/// zone     → geographic / logical zone
/// index    → sequential key index
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HdPath {
    pub chain: u32,
    pub role: u32,
    pub zone: u32,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HdPathError {
    InvalidFormat,
    InvalidPurpose,
    InvalidComponent,
    IndexOverflow,
}

impl fmt::Display for HdPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "HD_PATH_INVALID_FORMAT"),
            Self::InvalidPurpose => write!(f, "HD_PATH_INVALID_PURPOSE"),
            Self::InvalidComponent => write!(f, "HD_PATH_INVALID_COMPONENT"),
            Self::IndexOverflow => write!(f, "HD_PATH_INDEX_OVERFLOW"),
        }
    }
}

impl std::error::Error for HdPathError {}

impl HdPath {
    /// Creates a new HD path with validation.
    pub fn new(chain: u32, role: u32, zone: u32, index: u32) -> Result<Self, HdPathError> {
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

    /// Returns canonical string representation.
    ///
    /// Example:
    ///
    /// m/44/2626/1/1/2/0
    pub fn to_string_path(&self) -> String {
        format!(
            "m/{}/{}/{}/{}/{}/{}",
            AOXC_HD_BIP44_PURPOSE, AOXC_HD_PURPOSE, self.chain, self.role, self.zone, self.index
        )
    }

    /// Returns a hardened version of the index.
    pub fn hardened_index(&self) -> u32 {
        self.index | HARDENED_OFFSET
    }

    /// Returns next sequential path.
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

    /// Returns true if the index is hardened.
    pub fn is_hardened(&self) -> bool {
        self.index >= HARDENED_OFFSET
    }
}

impl FromStr for HdPath {
    type Err = HdPathError;

    /// Parses canonical AOXC HD path string.
    ///
    /// Example:
    ///
    /// m/44/2626/1/1/2/0
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split('/').collect();

        if parts.len() != 7 {
            return Err(HdPathError::InvalidFormat);
        }

        if parts[0] != "m" {
            return Err(HdPathError::InvalidFormat);
        }

        let bip44_purpose: u32 = parts[1]
            .parse()
            .map_err(|_| HdPathError::InvalidComponent)?;

        if bip44_purpose != AOXC_HD_BIP44_PURPOSE {
            return Err(HdPathError::InvalidPurpose);
        }

        let purpose: u32 = parts[2]
            .parse()
            .map_err(|_| HdPathError::InvalidComponent)?;

        if purpose != AOXC_HD_PURPOSE {
            return Err(HdPathError::InvalidPurpose);
        }

        let chain = parts[3]
            .parse()
            .map_err(|_| HdPathError::InvalidComponent)?;
        let role = parts[4]
            .parse()
            .map_err(|_| HdPathError::InvalidComponent)?;
        let zone = parts[5]
            .parse()
            .map_err(|_| HdPathError::InvalidComponent)?;
        let index = parts[6]
            .parse()
            .map_err(|_| HdPathError::InvalidComponent)?;

        HdPath::new(chain, role, zone, index)
    }
}

impl fmt::Display for HdPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hd_path_creation() {
        let path = HdPath::new(1, 2, 3, 0).unwrap();
        assert_eq!(path.chain, 1);
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
    fn hardened_index() {
        let path = HdPath::new(1, 1, 1, 0).unwrap();
        assert_eq!(path.hardened_index(), HARDENED_OFFSET);
    }
}
