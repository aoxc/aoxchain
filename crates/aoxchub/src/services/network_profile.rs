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
    resolve_profile_from_endpoint(&endpoint)
}

pub fn resolve_profile_from_endpoint(endpoint: &str) -> NetworkProfile {
    let endpoint = endpoint.to_ascii_lowercase();
    if endpoint.contains("devnet") {
        NetworkProfile::Devnet
    } else if endpoint.contains("testnet") {
        NetworkProfile::Testnet
    } else {
        NetworkProfile::Mainnet
    }
}

#[cfg(test)]
mod tests {
    use super::{NetworkProfile, resolve_profile_from_endpoint};

    #[test]
    fn resolves_devnet_from_endpoint() {
        let resolved = resolve_profile_from_endpoint("https://rpc.devnet.aoxchain.io");
        assert_eq!(resolved, NetworkProfile::Devnet);
    }

    #[test]
    fn resolves_testnet_from_endpoint() {
        let resolved = resolve_profile_from_endpoint("https://rpc.testnet.aoxchain.io");
        assert_eq!(resolved, NetworkProfile::Testnet);
    }

    #[test]
    fn falls_back_to_mainnet_when_unknown() {
        let resolved = resolve_profile_from_endpoint("http://127.0.0.1:8545");
        assert_eq!(resolved, NetworkProfile::Mainnet);
    }
}
