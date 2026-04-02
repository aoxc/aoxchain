//! Deterministic ML-DSA parameter metadata used by AOXCVM policy layers.

use crate::errors::{AoxcvmError, AoxcvmResult};

/// Supported ML-DSA parameter sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MlDsaParameterSet {
    MlDsa44,
    MlDsa65,
    MlDsa87,
}

impl MlDsaParameterSet {
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::MlDsa44 => "ml-dsa-44",
            Self::MlDsa65 => "ml-dsa-65",
            Self::MlDsa87 => "ml-dsa-87",
        }
    }

    /// Conservative deterministic limits for admission checks.
    pub const fn max_signature_bytes(self) -> usize {
        match self {
            Self::MlDsa44 => 2_700,
            Self::MlDsa65 => 3_500,
            Self::MlDsa87 => 5_000,
        }
    }

    /// Conservative deterministic limits for admission checks.
    pub const fn max_public_key_bytes(self) -> usize {
        match self {
            Self::MlDsa44 => 1_600,
            Self::MlDsa65 => 2_100,
            Self::MlDsa87 => 2_700,
        }
    }
}

/// Bounded metadata carried for ML-DSA witnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlDsaSignatureMeta {
    pub parameter_set: MlDsaParameterSet,
    pub signature_len: usize,
    pub public_key_len: usize,
}

impl MlDsaSignatureMeta {
    pub fn validate_bounds(self) -> AoxcvmResult<()> {
        if self.signature_len == 0 {
            return Err(AoxcvmError::InvalidSignatureMetadata(
                "ml-dsa signature_len must be non-zero",
            ));
        }
        if self.public_key_len == 0 {
            return Err(AoxcvmError::InvalidSignatureMetadata(
                "ml-dsa public_key_len must be non-zero",
            ));
        }
        if self.signature_len > self.parameter_set.max_signature_bytes() {
            return Err(AoxcvmError::AuthLimitExceeded {
                limit: "ml_dsa.max_signature_bytes",
                got: self.signature_len,
                max: self.parameter_set.max_signature_bytes(),
            });
        }
        if self.public_key_len > self.parameter_set.max_public_key_bytes() {
            return Err(AoxcvmError::AuthLimitExceeded {
                limit: "ml_dsa.max_public_key_bytes",
                got: self.public_key_len,
                max: self.parameter_set.max_public_key_bytes(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MlDsaParameterSet, MlDsaSignatureMeta};

    #[test]
    fn wire_ids_are_stable() {
        assert_eq!(MlDsaParameterSet::MlDsa44.wire_id(), "ml-dsa-44");
        assert_eq!(MlDsaParameterSet::MlDsa65.wire_id(), "ml-dsa-65");
        assert_eq!(MlDsaParameterSet::MlDsa87.wire_id(), "ml-dsa-87");
    }

    #[test]
    fn accepts_lengths_within_bounds() {
        let meta = MlDsaSignatureMeta {
            parameter_set: MlDsaParameterSet::MlDsa65,
            signature_len: 3_300,
            public_key_len: 1_952,
        };

        assert!(meta.validate_bounds().is_ok());
    }

    #[test]
    fn rejects_signature_length_over_limit() {
        let meta = MlDsaSignatureMeta {
            parameter_set: MlDsaParameterSet::MlDsa44,
            signature_len: 2_701,
            public_key_len: 1_312,
        };

        assert!(meta.validate_bounds().is_err());
    }
}
