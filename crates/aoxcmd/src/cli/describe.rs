// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    build_info::{BuildInfo, build_info},
    cli::AOXC_RELEASE_NAME,
    cli_support::{OutputFormat, emit_serialized, output_format, text_envelope},
    config::{loader::load, settings::Settings},
    data_home::resolve_home,
    error::{AppError, ErrorCode},
    logging::init::trace_for,
    services::registry::{ServiceDescriptor, default_registry},
};
use serde::Serialize;
use std::collections::BTreeMap;

/// Operator-facing compatibility matrix published by the describe surface.
///
/// Design intent:
/// - Provide a stable machine-readable compatibility summary for release evidence.
/// - Preserve a compact contract that downstream tooling can consume without
///   re-parsing multiple independent informational surfaces.
/// - Bind protocol, upgrade, and trust-chain expectations into one envelope.
#[derive(Debug, Clone, Serialize)]
struct CompatibilityMatrix<'a> {
    binary_version: &'a str,
    release_name: &'a str,
    protocol: ProtocolCompatibility<'a>,
    policy: UpgradePolicy<'a>,
    network_validation: Vec<ValidationTrack<'a>>,
    release_trust_chain: ReleaseTrustChain<'a>,
}

/// Protocol-line compatibility details surfaced to operators and release tooling.
#[derive(Debug, Clone, Serialize)]
struct ProtocolCompatibility<'a> {
    core_line: &'a str,
    kernel_line: &'a str,
    block_format: &'a str,
    vote_format: &'a str,
    certificate_format: &'a str,
    enforcement: Vec<EnforcementRule<'a>>,
}

/// Version-enforcement rule for a protocol-visible surface.
#[derive(Debug, Clone, Serialize)]
struct EnforcementRule<'a> {
    surface: &'a str,
    expected_version: &'a str,
    behavior_on_mismatch: &'a str,
}

/// Upgrade-policy statement for the currently published release line.
#[derive(Debug, Clone, Serialize)]
struct UpgradePolicy<'a> {
    backward_compatibility: &'a str,
    rollback_window: &'a str,
    migration_guarantee: &'a str,
    supported_upgrade_paths: Vec<UpgradePath<'a>>,
}

/// Supported or gated release transition.
#[derive(Debug, Clone, Serialize)]
struct UpgradePath<'a> {
    from: &'a str,
    to: &'a str,
    status: &'a str,
    guarantee: &'a str,
}

/// Scripted or policy-backed validation track.
#[derive(Debug, Clone, Serialize)]
struct ValidationTrack<'a> {
    name: &'a str,
    status: &'a str,
    evidence_hint: &'a str,
}

/// Release trust-chain obligations published with the binary.
#[derive(Debug, Clone, Serialize)]
struct ReleaseTrustChain<'a> {
    reproducible_build: &'a str,
    artifact_signature: &'a str,
    provenance_attestation: &'a str,
    release_evidence_gate: &'a str,
    compatibility_matrix: &'a str,
}

/// Build manifest emitted by the `build-manifest` describe surface.
#[derive(Debug, Clone, Serialize)]
struct Manifest<'a> {
    build: BuildInfo,
    services: &'a [ServiceDescriptor],
    compatibility: CompatibilityMatrix<'a>,
}

/// Operator-facing port map for the active effective settings surface.
#[derive(Debug, Clone, Serialize)]
struct PortMap<'a> {
    bind_host: &'a str,
    p2p_port: u16,
    rpc_port: u16,
    prometheus_port: u16,
}

/// Unified advanced API/CLI blueprint for operators building automation.
#[derive(Debug, Clone, Serialize)]
struct QuantumAutomationBlueprint<'a> {
    profile_version: &'a str,
    posture: &'a str,
    api_controls: Vec<&'a str>,
    cli_controls: Vec<&'a str>,
    release_gates: Vec<&'a str>,
}

/// Emits JSON for describe-surface commands that intentionally expose a stable
/// machine-readable contract.
fn emit_json<T: Serialize>(value: &T) -> Result<(), AppError> {
    emit_serialized(value, OutputFormat::Json)
}

/// Resolves effective settings for read-only describe surfaces without creating
/// configuration files on disk.
///
/// Behavioral policy:
/// - If canonical settings already exist, load and validate them.
/// - If settings are missing, derive safe in-memory defaults for the active
///   AOXC home without persisting anything.
/// - If settings exist but are invalid, propagate the underlying error.
///
/// Rationale:
/// - Read-only informational commands must not unexpectedly mutate operator
///   state merely to display policy or port metadata.
/// - Missing configuration should still allow deterministic previews based on
///   the canonical AOXC default settings model.
fn effective_settings_for_describe_surface() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            let home = resolve_home()?;
            Ok(Settings::default_for(home.display().to_string()))
        }
        Err(error) => Err(error),
    }
}

/// Constructs the canonical compatibility matrix for the current binary build.
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
                    from: "0.2.0-aoxcq",
                    to: "0.2.0-aoxcq+patch",
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

/// Emits machine-readable build metadata for the current binary.
pub fn cmd_version() -> Result<(), AppError> {
    emit_json(&build_info())
}

/// Emits a machine-readable advanced API+CLI blueprint for operator automation.
pub fn cmd_quantum_blueprint() -> Result<(), AppError> {
    emit_json(&QuantumAutomationBlueprint {
        profile_version: "v1",
        posture: "hybrid-post-quantum-hardening",
        api_controls: vec![
            "idempotency-key required on write operations",
            "request-signature envelope validated before admission",
            "adaptive rate limit and rejection metrics exported",
            "strict request/response schema contracts with compatibility versioning",
        ],
        cli_controls: vec![
            "all readiness and gate commands support JSON output for CI",
            "operator evidence commands are mandatory for release audits",
            "health and status surfaces are deterministic and script-safe",
        ],
        release_gates: vec![
            "compatibility matrix report must be generated",
            "full-surface readiness gate must pass",
            "security evidence bundle must be complete",
        ],
    })
}

/// Emits the high-level vision statement for the AOXC operator command plane.
pub fn cmd_vision() -> Result<(), AppError> {
    let trace = trace_for("vision");
    let mut details = BTreeMap::new();

    details.insert("release".to_string(), AOXC_RELEASE_NAME.to_string());
    details.insert(
        "statement".to_string(),
        "AOXCMD is the deterministic operator command plane for bootstrap, diagnostics, and audit evidence generation."
            .to_string(),
    );
    details.insert("correlation_id".to_string(), trace.correlation_id);

    emit_json(&text_envelope("vision", "ok", details))
}

/// Emits the operator-facing build manifest.
pub fn cmd_build_manifest() -> Result<(), AppError> {
    let registry = default_registry();
    let manifest = Manifest {
        build: build_info(),
        services: &registry,
        compatibility: compatibility_matrix(),
    };

    emit_json(&manifest)
}

/// Emits the effective node connection policy.
///
/// Behavioral note:
/// - Existing validated settings are used when present.
/// - Missing settings are represented by deterministic in-memory defaults for
///   the active AOXC home.
/// - The `--enforce-official` flag is treated as an invocation-level override
///   preview layered over the persisted or default settings surface.
pub fn cmd_node_connection_policy(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_describe_surface()?;
    let mut details = BTreeMap::new();

    let effective_official_only = settings.network.enforce_official_peers
        || args.iter().any(|arg| arg == "--enforce-official");

    details.insert(
        "official_only".to_string(),
        effective_official_only.to_string(),
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

/// Emits the canonical sovereign root modules recognized by AOXC.
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

    emit_json(&roots)
}

/// Emits the high-level module architecture list for the AOXC command plane.
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

    emit_json(&modules)
}

/// Emits the canonical compatibility matrix.
pub fn cmd_compat_matrix() -> Result<(), AppError> {
    emit_json(&compatibility_matrix())
}

/// Emits the effective port map derived from the active settings surface.
///
/// Behavioral policy:
/// - Existing validated settings are preferred.
/// - Missing settings fall back to deterministic in-memory defaults without
///   mutating operator state.
pub fn cmd_port_map() -> Result<(), AppError> {
    let settings = effective_settings_for_describe_surface()?;
    let map = PortMap {
        bind_host: &settings.network.bind_host,
        p2p_port: settings.network.p2p_port,
        rpc_port: settings.network.rpc_port,
        prometheus_port: settings.telemetry.prometheus_port,
    };

    emit_json(&map)
}

#[cfg(test)]
mod tests {
    use super::{
        cmd_quantum_posture, compatibility_matrix, effective_settings_for_describe_surface,
    };
    use crate::test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock};

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn compatibility_matrix_captures_protocol_enforcement() {
        let matrix = compatibility_matrix();

        assert_eq!(matrix.protocol.enforcement.len(), 3);
        assert!(
            matrix
                .protocol
                .enforcement
                .iter()
                .any(|rule| rule.surface == "p2p-envelope.protocol_version")
        );
    }

    #[test]
    fn compatibility_matrix_requires_release_trust_chain_controls() {
        let matrix = compatibility_matrix();

        assert!(
            matrix
                .release_trust_chain
                .artifact_signature
                .contains("required")
        );
        assert!(
            matrix
                .network_validation
                .iter()
                .any(|track| track.name == "state sync and snapshot recovery")
        );
    }

    #[test]
    fn describe_settings_fallback_uses_in_memory_defaults_when_config_is_missing() {
        with_test_home("describe-settings-fallback", |home| {
            let settings = effective_settings_for_describe_surface()
                .expect("missing config should fall back to deterministic defaults");

            assert_eq!(settings.profile, "validation");
            assert_eq!(settings.home_dir, home.path().display().to_string());
            assert!(
                !home.path().join("config").join("settings.json").exists(),
                "read-only describe surfaces must not create configuration files"
            );
        });
    }

    #[test]
    fn quantum_posture_command_succeeds_with_default_validation_profile() {
        with_test_home("describe-quantum-posture-default", |_home| {
            cmd_quantum_posture(&["--format".to_string(), "json".to_string()])
                .expect("quantum posture should render with default in-memory settings");
        });
    }
}
