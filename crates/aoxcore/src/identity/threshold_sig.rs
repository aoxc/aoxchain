use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialSignature {
    pub signer_id: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThresholdPolicy {
    pub min_signers: usize,
}

impl ThresholdPolicy {
    #[must_use]
    pub const fn new(min_signers: usize) -> Self {
        Self { min_signers }
    }
}

pub fn verify_threshold_signatures(
    policy: &ThresholdPolicy,
    partials: &[PartialSignature],
) -> Result<(), String> {
    if policy.min_signers == 0 {
        return Err("TSS_INVALID_POLICY".to_string());
    }

    let mut unique_signers = HashSet::new();

    for partial in partials {
        if partial.signer_id.trim().is_empty() || partial.signature.is_empty() {
            return Err("TSS_INVALID_PARTIAL_SIGNATURE".to_string());
        }

        unique_signers.insert(partial.signer_id.clone());
    }

    if unique_signers.len() < policy.min_signers {
        return Err("TSS_QUORUM_NOT_REACHED".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PartialSignature, ThresholdPolicy, verify_threshold_signatures};

    #[test]
    fn threshold_reached() {
        let policy = ThresholdPolicy::new(2);
        let partials = vec![
            PartialSignature {
                signer_id: "validator-1".to_string(),
                signature: vec![1, 2, 3],
            },
            PartialSignature {
                signer_id: "validator-2".to_string(),
                signature: vec![4, 5, 6],
            },
        ];

        assert!(verify_threshold_signatures(&policy, &partials).is_ok());
    }
}
