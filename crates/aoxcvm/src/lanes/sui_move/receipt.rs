/// Sui / Move lane receipt extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiMoveLaneReceipt {
    pub published_package: Option<[u8; 32]>,
    pub mutated_object: Option<[u8; 32]>,
}
