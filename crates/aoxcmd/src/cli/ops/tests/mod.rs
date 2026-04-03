use super::{
    FaucetClaimRecord, FaucetState, build_surface, collect_surface_gate_failures,
    compare_aoxhub_network_profiles, compare_embedded_network_profiles, evaluate_faucet_claim,
    evaluate_full_surface_readiness, evaluate_profile_readiness, full_surface_markdown_report,
    has_desktop_wallet_compat_artifact, has_matching_artifact, has_production_closure_artifacts,
    has_release_evidence, has_release_provenance_bundle, has_security_drill_artifact,
    historical_tx_hashes, locate_repo_artifact_dir, open_checklist_items, parse_network_profile,
    parse_positive_u64_arg, parse_required_or_default_text_arg, ports_are_shifted_consistently,
    readiness_markdown_report, rpc_http_get_probe, rpc_jsonrpc_status_probe, surface_check,
    tx_hash_hex, write_readiness_markdown_report,
};
use crate::config::settings::Settings;
use aoxcdata::BlockEnvelope;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("aoxcmd-ops-{label}-{nanos}"))
}

fn touch(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, "{}").expect("fixture artifact should be written");
}

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

mod core;
mod network;
mod readiness;
