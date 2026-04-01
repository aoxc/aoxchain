#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetObject {
    pub asset_id: [u8; 32],
    pub symbol: String,
    pub amount: u128,
}
