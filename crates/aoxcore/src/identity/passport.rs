use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

/// Current passport format version
pub const PASSPORT_VERSION: u8 = 1;

/// Represents a node identity passport.
///
/// A passport bundles actor metadata together with its certificate
/// and minimal runtime identity information used during handshake.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Passport {
    pub version: u8,
    pub actor_id: String,
    pub role: String,
    pub zone: String,
    pub certificate: String,
    pub issued_at: u64,
    pub expires_at: u64,
}

impl Passport {
    /// Creates a new passport.
    pub fn new(
        actor_id: String,
        role: String,
        zone: String,
        certificate: String,
        issued_at: u64,
        expires_at: u64,
    ) -> Self {
        Self {
            version: PASSPORT_VERSION,
            actor_id,
            role,
            zone,
            certificate,
            issued_at,
            expires_at,
        }
    }

    /// Returns true if the passport has expired.
    pub fn is_expired(&self, now: u64) -> bool {
        now > self.expires_at
    }

    /// Computes a deterministic fingerprint for the passport.
    ///
    /// Useful for logging and debugging.
    pub fn fingerprint(&self) -> String {
        let encoded = serde_json::to_vec(self).unwrap_or_default();

        let mut hasher = Sha3_256::new();

        hasher.update(encoded);

        let digest = hasher.finalize();

        hex::encode_upper(&digest[..8])
    }

    /// Serializes the passport to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("PASSPORT_SERIALIZE_ERROR: {}", e))
    }

    /// Restores passport from JSON.
    pub fn from_json(data: &str) -> Result<Self, String> {
        serde_json::from_str(data).map_err(|e| format!("PASSPORT_PARSE_ERROR: {}", e))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn passport_roundtrip() {
        let passport = Passport::new(
            "AOXC-VAL-EU-1234".into(),
            "validator".into(),
            "EU".into(),
            "CERT_DATA".into(),
            100,
            200,
        );

        let json = passport.to_json().unwrap();

        let restored = Passport::from_json(&json).unwrap();

        assert_eq!(passport.actor_id, restored.actor_id);
    }

    #[test]
    fn expiration_check() {
        let passport = Passport::new(
            "actor".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            200,
        );

        assert!(passport.is_expired(300));
        assert!(!passport.is_expired(150));
    }
}
