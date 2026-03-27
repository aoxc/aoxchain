use crate::config::RpcConfig;
use crate::types::{HealthResponse, RpcSecurityPosture};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Returns the current health status exposed by the HTTP RPC interface.
#[must_use]
pub fn health() -> HealthResponse {
    health_with_context(&RpcConfig::default(), 0)
}

/// Returns a detailed health payload suitable for production-grade health probes.
#[must_use]
pub fn health_with_context(config: &RpcConfig, uptime_secs: u64) -> HealthResponse {
    let validation = config.validate();
    let tls_cert_exists = Path::new(&config.tls_cert_path).exists();
    let tls_key_exists = Path::new(&config.tls_key_path).exists();
    let mtls_enabled = config.mtls_ca_cert_path.is_some();

    let status = if !validation.errors.is_empty() {
        "error"
    } else if !validation.warnings.is_empty() {
        "degraded"
    } else {
        "ok"
    }
    .to_string();

    HealthResponse {
        status,
        chain_id: config.chain_id.clone(),
        genesis_hash: config.genesis_hash.clone(),
        tls_enabled: tls_cert_exists && tls_key_exists,
        mtls_enabled,
        tls_cert_sha256: certificate_fingerprint_sha256_from_path(&config.tls_cert_path),
        readiness_score: validation.readiness_score(),
        warnings: validation.warnings.clone(),
        errors: validation.errors.clone(),
        recommendations: recommendations_from_validation(&validation.warnings, &validation.errors),
        security_posture: security_posture_from_validation(&validation.warnings, &validation.errors),
        uptime_secs,
    }
}

fn recommendations_from_validation(warnings: &[String], errors: &[String]) -> Vec<String> {
    let mut recommendations = Vec::new();

    if warnings
        .iter()
        .any(|warning| warning.contains("genesis_hash"))
        || errors.iter().any(|error| error.contains("genesis_hash"))
    {
        recommendations.push(
            "Set a canonical 0x-prefixed genesis_hash in RpcConfig and enforce it at node startup"
                .to_string(),
        );
    }

    if warnings.iter().any(|warning| warning.contains("tls")) {
        recommendations.push(
            "Provision TLS certificate and private key files with strict filesystem permissions"
                .to_string(),
        );
    }

    if warnings.iter().any(|warning| warning.contains("mTLS")) {
        recommendations
            .push("Enable mTLS and configure a trusted CA chain for client auth".to_string());
    }

    if errors
        .iter()
        .any(|error| error.contains("max_requests_per_minute"))
    {
        recommendations.push(
            "Set max_requests_per_minute to a non-zero baseline aligned with traffic SLOs"
                .to_string(),
        );
    }

    if errors
        .iter()
        .any(|error| error.contains("rate_limiter_window_secs"))
    {
        recommendations
            .push("Set rate_limiter_window_secs to a non-zero duration (e.g. 60)".to_string());
    }

    if errors
        .iter()
        .any(|error| error.contains("rate_limiter_max_tracked_keys"))
    {
        recommendations.push(
            "Set rate_limiter_max_tracked_keys to a non-zero bounded capacity (e.g. 100000)"
                .to_string(),
        );
    }

    if errors
        .iter()
        .any(|error| error.contains("bind_addr"))
    {
        recommendations
            .push("Use explicit ip:port bindings for HTTP, WebSocket and gRPC listeners".to_string());
    }

    if warnings
        .iter()
        .any(|warning| warning.contains("mTLS CA certificate"))
    {
        recommendations.push(
            "Provide a trusted CA file for mTLS or explicitly disable mTLS for non-production profiles"
                .to_string(),
        );
    }

    recommendations
}

fn security_posture_from_validation(warnings: &[String], errors: &[String]) -> RpcSecurityPosture {
    let level = if !errors.is_empty() {
        "critical"
    } else if warnings.is_empty() {
        "hardened"
    } else {
        "guarded"
    };

    let score_band = if !errors.is_empty() {
        "0-49"
    } else if warnings.len() > 2 {
        "50-79"
    } else {
        "80-100"
    };

    let blockers = errors
        .iter()
        .filter(|error| {
            error.contains("genesis_hash")
                || error.contains("bind_addr")
                || error.contains("mTLS")
                || error.contains("max_requests_per_minute")
        })
        .cloned()
        .collect();

    RpcSecurityPosture {
        level: level.to_string(),
        score_band: score_band.to_string(),
        blockers,
    }
}

fn certificate_fingerprint_sha256_from_path(path: &str) -> Option<String> {
    let cert_bytes = fs::read(path).ok()?;
    Some(certificate_fingerprint_sha256(&cert_bytes))
}

fn certificate_fingerprint_sha256(cert_bytes: &[u8]) -> String {
    let digest = Sha256::digest(cert_bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_health_is_degraded_without_production_inputs() {
        let health = health();

        assert_eq!(health.status, "degraded");
        assert!(health.readiness_score < 100);
        assert!(health.errors.is_empty());
        assert!(
            health
                .warnings
                .iter()
                .any(|warning| warning.contains("genesis_hash"))
        );
        assert!(!health.recommendations.is_empty());
        assert_eq!(health.security_posture.level, "guarded");
        assert_eq!(health.security_posture.score_band, "50-79");
    }

    #[test]
    fn certificate_fingerprint_is_deterministic() {
        let fingerprint_a = certificate_fingerprint_sha256(b"dummy-cert");
        let fingerprint_b = certificate_fingerprint_sha256(b"dummy-cert");
        let fingerprint_c = certificate_fingerprint_sha256(b"dummy-cert-2");

        assert_eq!(fingerprint_a, fingerprint_b);
        assert_ne!(fingerprint_a, fingerprint_c);
        assert_eq!(fingerprint_a.len(), 64);
    }

    #[test]
    fn health_is_ok_when_all_critical_controls_are_ready() {
        let config = RpcConfig {
            genesis_hash: Some(format!("0x{}", "ab".repeat(32))),
            tls_cert_path: "Cargo.toml".to_string(),
            tls_key_path: "README.md".to_string(),
            mtls_ca_cert_path: Some("Cargo.toml".to_string()),
            ..RpcConfig::default()
        };

        let health = health_with_context(&config, 42);

        assert_eq!(health.status, "ok");
        assert_eq!(health.readiness_score, 100);
        assert!(health.warnings.is_empty());
        assert!(health.errors.is_empty());
        assert!(health.recommendations.is_empty());
        assert_eq!(health.uptime_secs, 42);
        assert!(health.tls_cert_sha256.is_some());
        assert_eq!(health.security_posture.level, "hardened");
        assert!(health.security_posture.blockers.is_empty());
    }

    #[test]
    fn health_is_error_with_invalid_limits() {
        let config = RpcConfig {
            max_requests_per_minute: 0,
            ..RpcConfig::default()
        };

        let health = health_with_context(&config, 0);

        assert_eq!(health.status, "error");
        assert_eq!(health.readiness_score, 0);
        assert!(!health.errors.is_empty());
        assert_eq!(health.security_posture.level, "critical");
    }

    #[test]
    fn health_reports_error_and_guidance_for_malformed_genesis_hash() {
        let config = RpcConfig {
            genesis_hash: Some("0x1234".to_string()),
            ..RpcConfig::default()
        };

        let health = health_with_context(&config, 7);

        assert_eq!(health.status, "error");
        assert!(
            health
                .errors
                .iter()
                .any(|error| error.contains("genesis_hash is malformed"))
        );
        assert!(health.recommendations.iter().any(|recommendation| {
            recommendation.contains("genesis_hash") && recommendation.contains("node startup")
        }));
        assert_eq!(health.readiness_score, 0);
        assert!(
            health
                .security_posture
                .blockers
                .iter()
                .any(|entry| entry.contains("genesis_hash"))
        );
    }

    #[test]
    fn health_omits_certificate_fingerprint_when_cert_file_missing() {
        let config = RpcConfig::default();
        let health = health_with_context(&config, 0);

        assert!(health.tls_cert_sha256.is_none());
        assert_eq!(health.uptime_secs, 0);
    }
}
