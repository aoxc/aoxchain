use crate::data_home;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub data_dir: String,
    pub key_name: String,
    pub chain: String,
    pub role: String,
    pub zone: String,
    pub ca_issuer: String,
    pub cert_validity_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_dir: data_home::default_data_home()
                .to_string_lossy()
                .into_owned(),
            key_name: "relay-1".to_string(),
            chain: "AOXC-MAIN".to_string(),
            role: "relay".to_string(),
            zone: "global".to_string(),
            ca_issuer: "AOXC-ROOT-CA".to_string(),
            cert_validity_secs: 31_536_000,
        }
    }
}

impl Settings {
    #[must_use]
    pub fn keys_dir(&self) -> String {
        format!("{}/keys", self.data_dir)
    }

    #[must_use]
    pub fn genesis_path(&self) -> String {
        format!("{}/identity/genesis.json", self.data_dir)
    }
}
