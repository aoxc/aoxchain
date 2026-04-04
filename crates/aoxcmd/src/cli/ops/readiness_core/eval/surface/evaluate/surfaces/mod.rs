use super::*;
use std::path::Path;

mod aoxhub;
mod context;
mod desktop_wallet;
mod devnet;
mod mainnet;
mod quantum_consensus;
mod telemetry;
mod testnet;

use context::SurfaceContext;

pub(super) fn build_surface_readiness_set(
    settings: &crate::config::settings::Settings,
    mainnet_readiness: &Readiness,
    repo_root: &Path,
) -> Vec<SurfaceReadiness> {
    let context = SurfaceContext::new(repo_root);

    vec![
        mainnet::build(mainnet_readiness, &context, repo_root),
        quantum_consensus::build(repo_root),
        testnet::build(&context),
        aoxhub::build(&context),
        devnet::build(&context),
        desktop_wallet::build(&context),
        telemetry::build(settings, &context),
    ]
}
