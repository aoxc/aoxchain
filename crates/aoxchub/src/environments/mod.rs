use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Mainnet,
    Testnet,
}

impl Environment {
    pub fn from_slug(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Some(Self::Mainnet),
            "testnet" => Some(Self::Testnet),
            _ => None,
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }

    pub fn banner_text(self) -> &'static str {
        match self {
            Self::Mainnet => "MAINNET • PRODUCTION CONTROL SURFACE",
            Self::Testnet => "TESTNET • EXPERIMENTAL CONTROL SURFACE",
        }
    }

    pub fn root_config_path(self) -> &'static str {
        match self {
            Self::Mainnet => "configs/aoxhub/mainnet.toml",
            Self::Testnet => "configs/aoxhub/testnet.toml",
        }
    }

    pub fn aoxc_home(self) -> &'static str {
        match self {
            Self::Mainnet => "/mnt/xdbx/aoxc",
            Self::Testnet => "/mnt/xdbx/aoxc",
        }
    }

    pub fn make_scope(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet-*",
            Self::Testnet => "aoxc-q-*",
        }
    }
}
