/// Supported Ethereum JSON-RPC methods at the current development phase.
pub const SUPPORTED_ETH_RPC_METHODS: &[&str] = &[
    "eth_chainId",
    "eth_call",
    "eth_estimateGas",
    "eth_getTransactionReceipt",
];

/// Returns whether a method is supported by the current EVM surface.
pub fn is_supported_method(method: &str) -> bool {
    SUPPORTED_ETH_RPC_METHODS.contains(&method)
}
