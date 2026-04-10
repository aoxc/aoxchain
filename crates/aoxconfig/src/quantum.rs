// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

/// Post-quantum signature primitives supported by AOXC policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureScheme {
    Dilithium2,
    Dilithium3,
    Falcon1024,
    HybridEd25519Dilithium3,
}

impl SignatureScheme {
    /// Estimated quantum-resistant security level in bits.
    pub fn security_bits(self) -> u16 {
        match self {
            Self::Dilithium2 => 128,
            Self::Dilithium3 => 192,
            Self::Falcon1024 => 256,
            Self::HybridEd25519Dilithium3 => 192,
        }
    }
}

/// Post-quantum key exchange primitives supported by AOXC policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KexScheme {
    Kyber768,
    Kyber1024,
    HybridX25519Kyber768,
}

impl KexScheme {
    /// Estimated quantum-resistant security level in bits.
    pub fn security_bits(self) -> u16 {
        match self {
            Self::Kyber768 => 128,
            Self::Kyber1024 => 256,
            Self::HybridX25519Kyber768 => 128,
        }
    }
}

/// Runtime key management rules for quantum-hardened operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantumKeyPolicy {
    pub min_key_rotation_epochs: u64,
    pub required_signatures: u8,
    pub enable_hybrid_signatures: bool,
    pub allowed_signature_schemes: Vec<SignatureScheme>,
    pub allowed_kex_schemes: Vec<KexScheme>,
}

/// Audit and monitoring settings tied to quantum security controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantumAuditPolicy {
    pub enforce_attestation: bool,
    pub require_hsm_for_validators: bool,
    pub max_clock_skew_secs: u64,
    pub periodic_crypto_audit_epochs: u64,
}

/// Full post-quantum runtime policy bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantumSecurityConfig {
    pub enabled: bool,
    pub require_pq_for_validators: bool,
    pub require_pq_for_rpc: bool,
    pub min_security_level: u16,
    pub key_policy: QuantumKeyPolicy,
    pub audit_policy: QuantumAuditPolicy,
}

impl QuantumSecurityConfig {
    /// Returns true if the scheme is explicitly allowed by policy.
    pub fn is_signature_scheme_allowed(&self, scheme: SignatureScheme) -> bool {
        self.key_policy.allowed_signature_schemes.contains(&scheme)
    }

    /// Returns true if the key exchange is explicitly allowed by policy.
    pub fn is_kex_scheme_allowed(&self, scheme: KexScheme) -> bool {
        self.key_policy.allowed_kex_schemes.contains(&scheme)
    }

    /// Returns the strongest permitted signature security level.
    pub fn strongest_signature_level(&self) -> Option<u16> {
        self.key_policy
            .allowed_signature_schemes
            .iter()
            .map(|s| s.security_bits())
            .max()
    }

    /// Validate policy integrity and return all violations.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if !self.enabled {
            return errors;
        }

        if !matches!(self.min_security_level, 128 | 192 | 256) {
            errors.push("min_security_level must be one of 128, 192, 256".to_string());
        }

        if self.key_policy.min_key_rotation_epochs == 0 {
            errors.push("key_policy.min_key_rotation_epochs must be greater than zero".to_string());
        }

        if self.key_policy.required_signatures == 0 {
            errors.push("key_policy.required_signatures must be greater than zero".to_string());
        }

        if self.key_policy.allowed_signature_schemes.is_empty() {
            errors.push("key_policy.allowed_signature_schemes must not be empty".to_string());
        }

        if self.key_policy.allowed_kex_schemes.is_empty() {
            errors.push("key_policy.allowed_kex_schemes must not be empty".to_string());
        }

        if self.audit_policy.max_clock_skew_secs == 0 {
            errors.push("audit_policy.max_clock_skew_secs must be greater than zero".to_string());
        }

        if self.audit_policy.periodic_crypto_audit_epochs == 0 {
            errors.push(
                "audit_policy.periodic_crypto_audit_epochs must be greater than zero".to_string(),
            );
        }

        if self.require_pq_for_validators
            && self.key_policy.allowed_signature_schemes.iter().all(|s| {
                s.security_bits() < self.min_security_level
            })
        {
            errors.push(
                "no allowed signature scheme satisfies min_security_level for validators"
                    .to_string(),
            );
        }

        if self.require_pq_for_rpc
            && self
                .key_policy
                .allowed_kex_schemes
                .iter()
                .all(|k| k.security_bits() < self.min_security_level)
        {
            errors.push("no allowed kex scheme satisfies min_security_level for rpc".to_string());
        }

        errors
    }
}

impl Default for QuantumSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_pq_for_validators: true,
            require_pq_for_rpc: true,
            min_security_level: 192,
            key_policy: QuantumKeyPolicy {
                min_key_rotation_epochs: 720,
                required_signatures: 2,
                enable_hybrid_signatures: true,
                allowed_signature_schemes: vec![
                    SignatureScheme::Dilithium3,
                    SignatureScheme::Falcon1024,
                    SignatureScheme::HybridEd25519Dilithium3,
                ],
                allowed_kex_schemes: vec![KexScheme::Kyber1024, KexScheme::HybridX25519Kyber768],
            },
            audit_policy: QuantumAuditPolicy {
                enforce_attestation: true,
                require_hsm_for_validators: true,
                max_clock_skew_secs: 5,
                periodic_crypto_audit_epochs: 360,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{KexScheme, QuantumSecurityConfig, SignatureScheme};

    #[test]
    fn default_quantum_config_is_valid() {
        let cfg = QuantumSecurityConfig::default();
        assert!(cfg.validate().is_empty());
        assert_eq!(cfg.strongest_signature_level(), Some(256));
        assert!(cfg.is_signature_scheme_allowed(SignatureScheme::Dilithium3));
        assert!(cfg.is_kex_scheme_allowed(KexScheme::Kyber1024));
    }

    #[test]
    fn invalid_quantum_values_are_reported() {
        let mut cfg = QuantumSecurityConfig::default();
        cfg.min_security_level = 42;
        cfg.key_policy.allowed_signature_schemes.clear();
        cfg.audit_policy.periodic_crypto_audit_epochs = 0;

        let errs = cfg.validate();
        assert!(errs.len() >= 3);
        assert!(errs.iter().any(|e| e.contains("min_security_level")));
        assert!(errs.iter().any(|e| e.contains("allowed_signature_schemes")));
        assert!(errs.iter().any(|e| e.contains("periodic_crypto_audit_epochs")));
    }

    #[test]
    fn disabled_quantum_policy_skips_strict_validation() {
        let mut cfg = QuantumSecurityConfig::default();
        cfg.enabled = false;
        cfg.min_security_level = 7;
        cfg.key_policy.allowed_kex_schemes.clear();

        assert!(cfg.validate().is_empty());
    }
}
