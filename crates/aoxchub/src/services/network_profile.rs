use crate::services::rpc_client::RpcClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkProfile {
    Mainnet,
    Devnet,
    Testnet,
}

impl NetworkProfile {
    pub fn title(self) -> &'static str {
        match self {
            Self::Mainnet => "Mainnet",
            Self::Devnet => "Devnet",
            Self::Testnet => "Testnet",
        }
    }

    pub fn source_label(self) -> &'static str {
        match self {
            Self::Mainnet => "profile-registry/mainnet",
            Self::Devnet => "profile-registry/devnet",
            Self::Testnet => "profile-registry/testnet",
        }
    }
}

pub fn resolve_profile() -> NetworkProfile {
    let endpoint = RpcClient::endpoint().to_ascii_lowercase();
    if endpoint.contains("devnet") {
        NetworkProfile::Devnet
    } else if endpoint.contains("testnet") {
        NetworkProfile::Testnet
    } else {
        NetworkProfile::Mainnet
    }
}
