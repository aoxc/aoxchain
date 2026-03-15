/// Cardano-style lane receipt extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoLaneReceipt {
    pub created_utxo: Option<[u8; 32]>,
    pub spent_utxo: Option<[u8; 32]>,
}
