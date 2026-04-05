// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::config::RpcConfig;
use crate::types::{
    QuantumApiCapability, QuantumCliCapability, QuantumControl, QuantumCryptoProfile,
    QuantumFullProfile, QuantumHashLevel, QuantumKeyLevel, QuantumOpsPlaybook,
    QuantumRateLimitPolicy, QuantumRuntimeCounters, QuantumRuntimePosture,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns a baseline post-quantum cryptography profile for RPC clients.
///
/// This profile is intentionally simple so operators can expose it as a
/// machine-readable "what algorithms are expected" endpoint payload.
#[must_use]
pub fn quantum_crypto_profile() -> QuantumCryptoProfile {
    QuantumCryptoProfile {
        profile_version: "v1".to_string(),
        assurance_target_percent: 99.9999,
        hash_levels: vec![
            QuantumHashLevel {
                algorithm: "SHA3-512".to_string(),
                security_bits_classical: 256,
                security_bits_quantum_estimated: 256,
                purpose: "transaction and state commitment hashing".to_string(),
            },
            QuantumHashLevel {
                algorithm: "Argon2id".to_string(),
                security_bits_classical: 256,
                security_bits_quantum_estimated: 128,
                purpose: "password and keyfile KDF hardening".to_string(),
            },
        ],
        key_levels: vec![
            QuantumKeyLevel {
                primitive: "ML-KEM-768 + X25519 (hybrid)".to_string(),
                security_bits_classical: 192,
                security_bits_quantum_estimated: 192,
                purpose: "session key establishment with migration safety".to_string(),
            },
            QuantumKeyLevel {
                primitive: "ML-DSA-65".to_string(),
                security_bits_classical: 192,
                security_bits_quantum_estimated: 192,
                purpose: "node and transaction signature policy".to_string(),
            },
        ],
        notes: vec![
            "Target expresses operational confidence, not absolute unbreakability".to_string(),
            "Profile must be combined with rotation, audit, and secure implementation controls"
                .to_string(),
        ],
    }
}

/// Returns a full quantum posture profile for advanced automation surfaces.
///
/// The payload is designed for operators who need one machine-readable object
/// that covers API controls, CLI controls, and release/runtime expectations.
#[must_use]
pub fn quantum_full_profile() -> QuantumFullProfile {
    QuantumFullProfile {
        profile_version: "v1".to_string(),
        posture: "hybrid-post-quantum-hardening".to_string(),
        api_capabilities: vec![
            QuantumApiCapability {
                name: "idempotency-key".to_string(),
                status: "required".to_string(),
                rationale: "prevents duplicate execution under retries and transport churn"
                    .to_string(),
            },
            QuantumApiCapability {
                name: "request-signature-envelope".to_string(),
                status: "required".to_string(),
                rationale: "binds payload integrity and caller policy to each write path"
                    .to_string(),
            },
            QuantumApiCapability {
                name: "adaptive-rate-limit".to_string(),
                status: "enforced".to_string(),
                rationale: "protects admission and prevents asymmetric resource exhaustion"
                    .to_string(),
            },
        ],
        cli_capabilities: vec![
            QuantumCliCapability {
                command: "aoxc mainnet-readiness --format json".to_string(),
                status: "required-before-release".to_string(),
                rationale: "binds deployment to an explicit machine-verifiable gate".to_string(),
            },
            QuantumCliCapability {
                command: "aoxc full-surface-gate --format json".to_string(),
                status: "required-before-upgrade".to_string(),
                rationale: "ensures compatibility-sensitive surfaces are evaluated together"
                    .to_string(),
            },
            QuantumCliCapability {
                command: "aoxc operator-evidence-record --type security".to_string(),
                status: "required-for-audit".to_string(),
                rationale: "preserves immutable operator evidence for compliance and response"
                    .to_string(),
            },
        ],
        controls: vec![
            QuantumControl {
                control_id: "QCTRL-001".to_string(),
                objective: "hybrid key agreement".to_string(),
                enforcement: "ML-KEM-768 + X25519 session establishment policy".to_string(),
            },
            QuantumControl {
                control_id: "QCTRL-002".to_string(),
                objective: "signature migration safety".to_string(),
                enforcement: "ML-DSA-65 acceptance path and deterministic policy registry"
                    .to_string(),
            },
            QuantumControl {
                control_id: "QCTRL-003".to_string(),
                objective: "operator replay resistance".to_string(),
                enforcement: "nonce + time-window + request-id integrity checks".to_string(),
            },
        ],
        ops_playbook: QuantumOpsPlaybook {
            release_gate: vec![
                "Compatibility matrix must be emitted and archived".to_string(),
                "Security and readiness evidence bundle must be complete".to_string(),
                "Break-glass override must remain disabled in production profiles".to_string(),
            ],
            runtime_controls: vec![
                "Rate-limiter saturation must page on-call within SLA window".to_string(),
                "Admission rejection ratios must be exported via metrics".to_string(),
                "Key-rotation evidence must be generated on each policy rotation".to_string(),
            ],
            incident_response: vec![
                "Compromised credential path triggers immediate key revoke and rotation workflow"
                    .to_string(),
                "Replay/anomaly spikes trigger admission clamp and forensic snapshot".to_string(),
                "Post-incident report must include control effectiveness assessment".to_string(),
            ],
        },
    }
}

/// Returns a runtime-attested quantum posture derived from active RPC
/// configuration and server counters.
#[must_use]
pub fn quantum_runtime_posture(
    config: &RpcConfig,
    uptime_secs: u64,
    total_requests: u64,
    rejected_requests: u64,
    rate_limited_requests: u64,
    active_rate_limiter_keys: u64,
) -> QuantumRuntimePosture {
    let validation = config.validate();
    let generated_at_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());

    QuantumRuntimePosture {
        generated_at_unix_secs,
        chain_id: config.chain_id.clone(),
        profile: quantum_full_profile(),
        mtls_enabled: config.mtls_ca_cert_path.is_some(),
        tls_artifacts_present: std::path::Path::new(&config.tls_cert_path).exists()
            && std::path::Path::new(&config.tls_key_path).exists(),
        config_readiness_score: validation.readiness_score(),
        config_warnings: validation.warnings,
        config_errors: validation.errors,
        runtime_counters: QuantumRuntimeCounters {
            uptime_secs,
            total_requests,
            rejected_requests,
            rate_limited_requests,
            active_rate_limiter_keys,
        },
        rate_limit_policy: QuantumRateLimitPolicy {
            max_requests_per_minute: config.max_requests_per_minute,
            window_secs: config.rate_limiter_window_secs,
            max_tracked_keys: config.rate_limiter_max_tracked_keys,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantum_profile_contains_expected_baseline_algorithms() {
        let profile = quantum_crypto_profile();

        assert_eq!(profile.profile_version, "v1");
        assert_eq!(profile.assurance_target_percent, 99.9999);
        assert!(
            profile
                .hash_levels
                .iter()
                .any(|entry| entry.algorithm == "SHA3-512")
        );
        assert!(
            profile
                .key_levels
                .iter()
                .any(|entry| entry.primitive.contains("ML-KEM-768"))
        );
    }

    #[test]
    fn quantum_full_profile_has_core_controls() {
        let profile = quantum_full_profile();

        assert_eq!(profile.profile_version, "v1");
        assert_eq!(profile.posture, "hybrid-post-quantum-hardening");
        assert!(
            profile
                .controls
                .iter()
                .any(|control| control.control_id == "QCTRL-001")
        );
        assert!(
            profile
                .api_capabilities
                .iter()
                .any(|entry| entry.name == "idempotency-key")
        );
    }

    #[test]
    fn quantum_runtime_posture_contains_config_and_runtime_state() {
        let config = RpcConfig {
            chain_id: "AOX-QA".to_string(),
            max_requests_per_minute: 1200,
            rate_limiter_window_secs: 30,
            rate_limiter_max_tracked_keys: 2000,
            ..RpcConfig::default()
        };
        let posture = quantum_runtime_posture(&config, 42, 100, 4, 2, 3);

        assert_eq!(posture.chain_id, "AOX-QA");
        assert_eq!(posture.runtime_counters.total_requests, 100);
        assert_eq!(posture.rate_limit_policy.window_secs, 30);
        assert_eq!(posture.profile.posture, "hybrid-post-quantum-hardening");
    }
}
