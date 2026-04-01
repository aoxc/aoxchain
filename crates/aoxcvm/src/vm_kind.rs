// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::language::LanguageKind;

/// Identifies the execution lane selected for a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VmKind {
    Evm,
    SuiMove,
    Wasm,
    Cardano,
}

impl VmKind {
    /// Returns a stable namespace prefix used by the host storage layer.
    pub const fn as_prefix(self) -> &'static [u8] {
        match self {
            Self::Evm => b"evm",
            Self::SuiMove => b"sui",
            Self::Wasm => b"wasm",
            Self::Cardano => b"cardano",
        }
    }

    /// Returns a human-readable static name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evm => "evm",
            Self::SuiMove => "sui_move",
            Self::Wasm => "wasm",
            Self::Cardano => "cardano",
        }
    }

    /// Returns true when the lane executes Move-family bytecode.
    ///
    /// Note: the current runtime lane is Sui/Move-oriented rather than a
    /// generic "any Move chain" VM profile.
    pub const fn is_move_family(self) -> bool {
        matches!(self, Self::SuiMove)
    }

    /// Returns true when the lane can be used as a cross-chain relay endpoint.
    ///
    /// Today this expresses lane capability at routing level. Full relay-grade
    /// safety still requires chain-specific finality/proof verification layers
    /// at kernel and adapter boundaries.
    pub const fn supports_cross_chain_endpoint(self) -> bool {
        true
    }

    /// Maps a lane into its language-first kernel interoperability family.
    pub const fn language_kind(self) -> LanguageKind {
        match self {
            Self::Evm => LanguageKind::EvmBytecode,
            Self::SuiMove => LanguageKind::Move,
            Self::Wasm => LanguageKind::Wasm,
            Self::Cardano => LanguageKind::PlutusUtxoScript,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VmKind;
    use crate::language::LanguageKind;

    #[test]
    fn move_family_detection_is_explicit() {
        assert!(VmKind::SuiMove.is_move_family());
        assert!(!VmKind::Evm.is_move_family());
        assert!(!VmKind::Wasm.is_move_family());
        assert!(!VmKind::Cardano.is_move_family());
    }

    #[test]
    fn all_registered_lanes_can_be_selected_as_relay_endpoints() {
        for lane in [VmKind::Evm, VmKind::SuiMove, VmKind::Wasm, VmKind::Cardano] {
            assert!(lane.supports_cross_chain_endpoint());
        }
    }

    #[test]
    fn language_mapping_is_stable_for_each_lane() {
        assert_eq!(VmKind::Evm.language_kind(), LanguageKind::EvmBytecode);
        assert_eq!(VmKind::SuiMove.language_kind(), LanguageKind::Move);
        assert_eq!(VmKind::Wasm.language_kind(), LanguageKind::Wasm);
        assert_eq!(VmKind::Cardano.language_kind(), LanguageKind::PlutusUtxoScript);
    }
}
