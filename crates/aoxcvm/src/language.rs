// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// Language/runtime family used by execution lanes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LanguageKind {
    EvmBytecode,
    Move,
    Wasm,
    PlutusUtxoScript,
}

/// Proposed canonical name for the language-first kernel model.
pub const KERNEL_MODEL_NAME: &str = "AOXCLang";

/// Kernel-level interoperability policy profile keyed by language family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageInteropProfile {
    pub canonical_abi: &'static str,
    pub state_model: &'static str,
    pub deterministic_by_default: bool,
    pub requires_finality_proof_for_relay: bool,
}

impl LanguageKind {
    /// Returns a policy profile for language-first kernel scheduling.
    pub const fn relay_profile(self) -> LanguageInteropProfile {
        match self {
            Self::EvmBytecode => LanguageInteropProfile {
                canonical_abi: "evm_abi",
                state_model: "account",
                deterministic_by_default: true,
                requires_finality_proof_for_relay: true,
            },
            Self::Move => LanguageInteropProfile {
                canonical_abi: "move_abi",
                state_model: "object",
                deterministic_by_default: true,
                requires_finality_proof_for_relay: true,
            },
            Self::Wasm => LanguageInteropProfile {
                canonical_abi: "wasm_contract_abi",
                state_model: "kv_or_module",
                deterministic_by_default: true,
                requires_finality_proof_for_relay: true,
            },
            Self::PlutusUtxoScript => LanguageInteropProfile {
                canonical_abi: "utxo_validator",
                state_model: "utxo",
                deterministic_by_default: true,
                requires_finality_proof_for_relay: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LanguageKind;

    #[test]
    fn relay_profiles_require_proofs_for_all_language_families() {
        for family in [
            LanguageKind::EvmBytecode,
            LanguageKind::Move,
            LanguageKind::Wasm,
            LanguageKind::PlutusUtxoScript,
        ] {
            assert!(family.relay_profile().requires_finality_proof_for_relay);
        }
    }
}
