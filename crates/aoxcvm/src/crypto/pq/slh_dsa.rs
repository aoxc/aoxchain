//! Deterministic SLH-DSA parameter metadata used by AOXCVM policy layers.

use crate::errors::{AoxcvmError, AoxcvmResult};

/// Supported SLH-DSA parameter sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlhDsaParameterSet {
    Shake128s,
    Shake192s,
    Shake256s,
}

impl SlhDsaParameterSet {
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::Shake128s => "slh-dsa-shake-128s",
            Self::Shake192s => "slh-dsa-shake-192s",
            Self::Shake256s => "slh-dsa-shake-256s",
        }
    }

    pub const fn max_signature_bytes(self) -> usize {
        match self {
            Self::Shake128s => 9_000,
            Self::Shake192s => 18_000,
            Self::Shake256s => 36_000,
        }
    }

    pub const fn max_public_key_bytes(self) -> usize {
        match self {
            Self::Shake128s => 64,
            Self::Shake192s => 96,
            Self::Shake256s => 128,
        }
    }
}

/// Bounded metadata carried for SLH-DSA witnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlhDsaSignatureMeta {
    pub parameter_set: SlhDsaParameterSet,
    pub signature_len: usize,
    pub public_key_len: usize,
}

impl SlhDsaSignatureMeta {
    pub fn validate_bounds(self) -> AoxcvmResult<()> {
        if self.signature_len == 0 {
            return Err(AoxcvmError::InvalidSignatureMetadata(
                "slh-dsa signature_len must be non-zero",
            ));
        }
        if self.public_key_len == 0 {
            return Err(AoxcvmError::InvalidSignatureMetadata(
                "slh-dsa public_key_len must be non-zero",
            ));
        }
        if self.signature_len > self.parameter_set.max_signature_bytes() {
            return Err(AoxcvmError::AuthLimitExceeded {
                limit: "slh_dsa.max_signature_bytes",
                got: self.signature_len,
                max: self.parameter_set.max_signature_bytes(),
            });
        }
        if self.public_key_len > self.parameter_set.max_public_key_bytes() {
            return Err(AoxcvmError::AuthLimitExceeded {
                limit: "slh_dsa.max_public_key_bytes",
                got: self.public_key_len,
                max: self.parameter_set.max_public_key_bytes(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SlhDsaParameterSet, SlhDsaSignatureMeta};

    #[test]
    fn wire_ids_are_stable() {
        assert_eq!(
            SlhDsaParameterSet::Shake128s.wire_id(),
            "slh-dsa-shake-128s"
        );
        assert_eq!(
            SlhDsaParameterSet::Shake192s.wire_id(),
            "slh-dsa-shake-192s"
        );
        assert_eq!(
            SlhDsaParameterSet::Shake256s.wire_id(),
            "slh-dsa-shake-256s"
        );
    }

    #[test]
    fn accepts_lengths_within_bounds() {
        let meta = SlhDsaSignatureMeta {
            parameter_set: SlhDsaParameterSet::Shake128s,
            signature_len: 8_500,
            public_key_len: 32,
        };

        assert!(meta.validate_bounds().is_ok());
    }

    #[test]
    fn rejects_public_key_length_over_limit() {
        let meta = SlhDsaSignatureMeta {
            parameter_set: SlhDsaParameterSet::Shake192s,
            signature_len: 16_000,
            public_key_len: 97,
        };

        assert!(meta.validate_bounds().is_err());
    }
}
