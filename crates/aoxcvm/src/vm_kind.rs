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
}
