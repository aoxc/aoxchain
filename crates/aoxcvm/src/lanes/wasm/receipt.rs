/// WASM lane receipt extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmLaneReceipt {
    pub uploaded_code: Option<[u8; 32]>,
    pub instantiated_contract: Option<[u8; 32]>,
}
