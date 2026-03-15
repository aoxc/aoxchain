use aoxcore::identity::certificate::Certificate;
use aoxcore::identity::passport::Passport;
use serde::{Deserialize, Serialize};

/// Runtime key material resolved for the local node.
///
/// This structure intentionally separates:
/// - secret-bearing encrypted persistence payloads,
/// - public identity material used by runtime services,
/// - optional trust artifacts such as certificates and passports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    /// Canonical actor identifier derived from the public key, role, and zone.
    pub actor_id: String,

    /// Canonical role code used during actor-id derivation and certificate issuance.
    pub role: String,

    /// Canonical zone code used during actor-id derivation and certificate issuance.
    pub zone: String,

    /// Public key encoded as uppercase hexadecimal.
    pub public_key_hex: String,

    /// Secret key encrypted into a serialized AOXC keyfile envelope.
    pub encrypted_secret_key: String,

    /// Optional signed certificate issued for this actor.
    pub certificate: Option<Certificate>,

    /// Optional runtime passport derived from the certificate.
    pub passport: Option<Passport>,
}

impl KeyMaterial {
    /// Returns a lightweight runtime summary suitable for logs and diagnostics.
    #[must_use]
    pub fn summary(&self) -> KeyMaterialSummary {
        KeyMaterialSummary {
            actor_id: self.actor_id.clone(),
            role: self.role.clone(),
            zone: self.zone.clone(),
            public_key_hex: self.public_key_hex.clone(),
            has_certificate: self.certificate.is_some(),
            has_passport: self.passport.is_some(),
        }
    }
}

/// Non-secret summary view intended for operator-facing diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterialSummary {
    pub actor_id: String,
    pub role: String,
    pub zone: String,
    pub public_key_hex: String,
    pub has_certificate: bool,
    pub has_passport: bool,
}
