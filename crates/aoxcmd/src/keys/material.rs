use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub name: String,
    pub profile: String,
    pub created_at: String,
    pub fingerprint: String,
    pub public_key: String,
    pub encrypted_private_key: String,
}

impl KeyMaterial {
    pub fn generate(name: &str, profile: &str, password: &str) -> Self {
        let created_at = Utc::now().to_rfc3339();

        let mut public_hasher = Sha3_256::new();
        public_hasher.update(name.as_bytes());
        public_hasher.update(profile.as_bytes());
        public_hasher.update(created_at.as_bytes());
        let public_key = hex::encode(public_hasher.finalize());

        let mut private_hasher = Sha3_256::new();
        private_hasher.update(public_key.as_bytes());
        private_hasher.update(password.as_bytes());
        let encrypted_private_key = hex::encode(private_hasher.finalize());

        let mut fp_hasher = Sha3_256::new();
        fp_hasher.update(public_key.as_bytes());
        let fingerprint_full = hex::encode(fp_hasher.finalize());

        Self {
            name: name.to_string(),
            profile: profile.to_string(),
            created_at,
            fingerprint: fingerprint_full[..16].to_string(),
            public_key,
            encrypted_private_key,
        }
    }
}
