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
    emit_serialized(&text_envelope("vision", "ok", details), crate::cli_support::OutputFormat::Json)
}

pub fn cmd_build_manifest() -> Result<(), AppError> {
    let info = build_info();
    let registry = default_registry();
    #[derive(serde::Serialize)]
    struct Manifest<'a> {
        build: crate::build_info::BuildInfo,
        services: &'a [crate::services::registry::ServiceDescriptor],
    }
    let manifest = Manifest {
        build: info,
        services: &registry,
    };
    emit_serialized(&manifest, crate::cli_support::OutputFormat::Json)
}

pub fn cmd_node_connection_policy(args: &[String]) -> Result<(), AppError> {
    let settings = load_or_init()?;
    let mut details = BTreeMap::new();
    details.insert(
        "official_only".to_string(),
        if settings.network.enforce_official_peers || args.iter().any(|a| a == "--enforce-official") {
            "true".to_string()
        } else {
            "false".to_string()
        },
    );
    details.insert(
        "allow_remote_peers".to_string(),
        settings.policy.allow_remote_peers.to_string(),
    );
    emit_serialized(&text_envelope("node-connection-policy", "ok", details), output_format(args))
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
    #[derive(serde::Serialize)]
    struct Entry<'a> {
        domain: &'a str,
        status: &'a str,
    }
    let matrix = vec![
        Entry { domain: "local-bootstrap", status: "supported" },
        Entry { domain: "deterministic-fixture", status: "supported" },
        Entry { domain: "multi-node-adversarial", status: "planned" },
        Entry { domain: "external-attestation", status: "planned" },
    ];
    emit_serialized(&matrix, crate::cli_support::OutputFormat::Json)
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
