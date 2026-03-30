use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Mainnet,
    Testnet,
}

impl Environment {
    pub fn banner_text(self) -> &'static str {
        match self {
            Self::Mainnet => "MAINNET • PRODUCTION CONTROL SURFACE",
            Self::Testnet => "TESTNET • EXPERIMENTAL CONTROL SURFACE",
        }
    }
}
