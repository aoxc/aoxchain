// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    build_info::build_info,
    cli::AOXC_RELEASE_NAME,
    cli_support::{emit_serialized, output_format, text_envelope},
    config::loader::load_or_init,
    error::AppError,
    logging::init::trace_for,
    services::registry::default_registry,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
struct CompatibilityMatrix<'a> {
    binary_version: &'a str,
    release_name: &'a str,
    protocol: ProtocolCompatibility<'a>,
    policy: UpgradePolicy<'a>,
    network_validation: Vec<ValidationTrack<'a>>,
    release_trust_chain: ReleaseTrustChain<'a>,
}

#[derive(Debug, Clone, Serialize)]
struct ProtocolCompatibility<'a> {
    core_line: &'a str,
    kernel_line: &'a str,
    block_format: &'a str,
    vote_format: &'a str,
    certificate_format: &'a str,
    enforcement: Vec<EnforcementRule<'a>>,
}

#[derive(Debug, Clone, Serialize)]
struct EnforcementRule<'a> {
    surface: &'a str,
    expected_version: &'a str,
    behavior_on_mismatch: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct UpgradePolicy<'a> {
    backward_compatibility: &'a str,
    rollback_window: &'a str,
    migration_guarantee: &'a str,
    supported_upgrade_paths: Vec<UpgradePath<'a>>,
}

#[derive(Debug, Clone, Serialize)]
struct UpgradePath<'a> {
    from: &'a str,
    to: &'a str,
    status: &'a str,
    guarantee: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationTrack<'a> {
    name: &'a str,
    status: &'a str,
    evidence_hint: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct ReleaseTrustChain<'a> {
    reproducible_build: &'a str,
    artifact_signature: &'a str,
    provenance_attestation: &'a str,
    release_evidence_gate: &'a str,
    compatibility_matrix: &'a str,
}

use serde::Serialize;

fn compatibility_matrix() -> CompatibilityMatrix<'static> {
    let info = build_info();
    CompatibilityMatrix {
        binary_version: info.package_version,
        release_name: AOXC_RELEASE_NAME,
        protocol: ProtocolCompatibility {
            core_line: info.canonical_core.line,
            kernel_line: info.covenant_kernel.line,
            block_format: info.canonical_core.block_format,
            vote_format: info.covenant_kernel.vote_format,
            certificate_format: info.covenant_kernel.certificate_format,
            enforcement: vec![
                EnforcementRule {
                    surface: "block-header.version",
                    expected_version: "1",
                    behavior_on_mismatch: "reject block construction or verification",
                },
                EnforcementRule {
                    surface: "p2p-envelope.protocol_version",
                    expected_version: "1",
                    behavior_on_mismatch: "drop frame before gossip admission",
                },
                EnforcementRule {
                    surface: "vote/certificate format line",
                    expected_version: "AOXC-VOTE-FMT-V1-draft / AOXC-CERT-FMT-V1-draft",
                    behavior_on_mismatch: "treat quorum material as incompatible and non-canonical",
                },
            ],
        },
        policy: UpgradePolicy {
            backward_compatibility: "patch releases within the same protocol line must preserve on-disk state and operator CLI automation contracts",
            rollback_window: "same release line only; protocol-line downgrades require explicit snapshot restore",
            migration_guarantee: "state migrations must be deterministic, evidence-backed, and coupled with snapshot recovery rehearsal",
            supported_upgrade_paths: vec![
                UpgradePath {
                    from: "0.1.1-akdeniz",
                    to: "0.1.1-akdeniz+patch",
                    status: "supported",
                    guarantee: "no protocol line change; rolling restart permitted",
                },
                UpgradePath {
                    from: "AOXC-CORE-V1 / AOXC-COVENANT-KERNEL-V1-draft",
                    to: "same protocol line",
                    status: "supported",
                    guarantee: "wire and block formats must remain canonical-identical",
                },
                UpgradePath {
                    from: "protocol line N",
                    to: "protocol line N+1",
                    status: "gated",
                    guarantee: "requires explicit migration plan, compatibility evidence, and rollback snapshot",
                },
            ],
        },
        network_validation: vec![
            ValidationTrack {
                name: "multi-host validation",
                status: "scripted",
                evidence_hint: "scripts/validation/multi_host_validation.sh",
            },
            ValidationTrack {
                name: "fault injection and partition",
                status: "scripted",
                evidence_hint: "scripts/validation/network_production_closure.sh --scenario partition|delay|drop|restart",
            },
            ValidationTrack {
                name: "state sync and snapshot recovery",
                status: "scripted",
                evidence_hint: "scripts/validation/network_production_closure.sh --scenario recovery",
            },
            ValidationTrack {
                name: "soak and telemetry evidence",
                status: "scripted",
                evidence_hint: "scripts/validation/network_production_closure.sh --scenario soak",
            },
        ],
        release_trust_chain: ReleaseTrustChain {
            reproducible_build: "required via scripts/release/generate_release_evidence.sh",
            artifact_signature: "required via AOXC_SIGNING_CMD or pre-signed artifact injection",
            provenance_attestation: "required via AOXC_PROVENANCE_CMD or generated placeholder failure",
            release_evidence_gate: "release bundle is incomplete until checksums, signatures, provenance, and compatibility reports are present",
            compatibility_matrix: "published by `aoxc compat-matrix --format json` and bundled into release evidence",
        },
    }
}

pub fn cmd_version() -> Result<(), AppError> {
    emit_serialized(&build_info(), crate::cli_support::OutputFormat::Json)
}

pub fn cmd_vision() -> Result<(), AppError> {
    let trace = trace_for("vision");
    let mut details = BTreeMap::new();
    details.insert("release".to_string(), AOXC_RELEASE_NAME.to_string());
    details.insert(
        "statement".to_string(),
        "AOXCMD is the deterministic operator command plane for bootstrap, diagnostics, and audit evidence generation.".to_string(),
    );
    details.insert("correlation_id".to_string(), trace.correlation_id);
    emit_serialized(
        &text_envelope("vision", "ok", details),
        crate::cli_support::OutputFormat::Json,
    )
}

pub fn cmd_build_manifest() -> Result<(), AppError> {
    let info = build_info();
    let registry = default_registry();
    #[derive(serde::Serialize)]
    struct Manifest<'a> {
        build: crate::build_info::BuildInfo,
        services: &'a [crate::services::registry::ServiceDescriptor],
        compatibility: CompatibilityMatrix<'a>,
    }
    let manifest = Manifest {
        build: info,
        services: &registry,
        compatibility: compatibility_matrix(),
    };
    emit_serialized(&manifest, crate::cli_support::OutputFormat::Json)
}

pub fn cmd_node_connection_policy(args: &[String]) -> Result<(), AppError> {
    let settings = load_or_init()?;
    let mut details = BTreeMap::new();
    details.insert(
        "official_only".to_string(),
        if settings.network.enforce_official_peers || args.iter().any(|a| a == "--enforce-official")
        {
            "true".to_string()
        } else {
            "false".to_string()
        },
    );
    details.insert(
        "allow_remote_peers".to_string(),
        settings.policy.allow_remote_peers.to_string(),
    );
    emit_serialized(
        &text_envelope("node-connection-policy", "ok", details),
        output_format(args),
    )
}

pub fn cmd_sovereign_core() -> Result<(), AppError> {
    let roots = vec![
        "identity",
        "supply",
        "governance",
        "relay",
        "security",
        "settlement",
        "treasury",
    ];
    emit_serialized(&roots, crate::cli_support::OutputFormat::Json)
}

pub fn cmd_module_architecture() -> Result<(), AppError> {
    let modules = vec![
        "app",
        "cli",
        "config",
        "keys",
        "node",
        "runtime",
        "economy",
        "telemetry",
        "services",
        "audit",
    ];
    emit_serialized(&modules, crate::cli_support::OutputFormat::Json)
}

pub fn cmd_compat_matrix() -> Result<(), AppError> {
    emit_serialized(
        &compatibility_matrix(),
        crate::cli_support::OutputFormat::Json,
    )
}

pub fn cmd_port_map() -> Result<(), AppError> {
    let settings = load_or_init()?;
    #[derive(serde::Serialize)]
    struct PortMap<'a> {
        bind_host: &'a str,
        p2p_port: u16,
        rpc_port: u16,
        prometheus_port: u16,
    }
    let map = PortMap {
        bind_host: &settings.network.bind_host,
        p2p_port: settings.network.p2p_port,
        rpc_port: settings.network.rpc_port,
        prometheus_port: settings.telemetry.prometheus_port,
    };
    emit_serialized(&map, crate::cli_support::OutputFormat::Json)
}

#[cfg(test)]
mod tests {
    use super::compatibility_matrix;

    #[test]
    fn compatibility_matrix_captures_protocol_enforcement() {
        let matrix = compatibility_matrix();
        assert_eq!(matrix.protocol.enforcement.len(), 3);
        assert!(matrix
            .protocol
            .enforcement
            .iter()
            .any(|rule| rule.surface == "p2p-envelope.protocol_version"));
    }

    #[test]
    fn compatibility_matrix_requires_release_trust_chain_controls() {
        let matrix = compatibility_matrix();
        assert!(matrix
            .release_trust_chain
            .artifact_signature
            .contains("required"));
        assert!(matrix
            .network_validation
            .iter()
            .any(|track| track.name == "state sync and snapshot recovery"));
    }
}
