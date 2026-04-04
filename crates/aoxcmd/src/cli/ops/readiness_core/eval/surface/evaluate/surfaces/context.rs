use std::path::{Path, PathBuf};

pub(super) struct SurfaceContext {
    pub(super) release_dir: PathBuf,
    pub(super) closure_dir: PathBuf,
    pub(super) mainnet_config: PathBuf,
    pub(super) testnet_config: PathBuf,
    pub(super) devnet_config: PathBuf,
    pub(super) aoxhub_mainnet: PathBuf,
    pub(super) aoxhub_testnet: PathBuf,
    pub(super) testnet_fixture_v1: PathBuf,
    pub(super) testnet_fixture_exists: bool,
    pub(super) devnet_fixture: PathBuf,
    pub(super) testnet_launch: PathBuf,
    pub(super) multi_host: PathBuf,
    pub(super) frontend_rpc_doc: PathBuf,
    pub(super) mainnet_checklist: PathBuf,
}

impl SurfaceContext {
    pub(super) fn new(repo_root: &Path) -> Self {
        let testnet_fixture_v1 = repo_root
            .join("configs")
            .join("environments")
            .join("testnet")
            .join("genesis.v1.json");

        Self {
            release_dir: repo_root.join("artifacts").join("release-evidence"),
            closure_dir: repo_root
                .join("artifacts")
                .join("network-production-closure"),
            mainnet_config: repo_root
                .join("configs")
                .join("environments")
                .join("mainnet")
                .join("profile.toml"),
            testnet_config: repo_root
                .join("configs")
                .join("environments")
                .join("testnet")
                .join("profile.toml"),
            devnet_config: repo_root
                .join("configs")
                .join("environments")
                .join("devnet")
                .join("profile.toml"),
            aoxhub_mainnet: repo_root
                .join("configs")
                .join("aoxhub")
                .join("mainnet.toml"),
            aoxhub_testnet: repo_root
                .join("configs")
                .join("aoxhub")
                .join("testnet.toml"),
            testnet_fixture_exists: testnet_fixture_v1.exists(),
            testnet_fixture_v1,
            devnet_fixture: repo_root
                .join("configs")
                .join("environments")
                .join("devnet")
                .join("genesis.v1.json"),
            testnet_launch: repo_root
                .join("configs")
                .join("environments")
                .join("localnet")
                .join("launch-localnet.sh"),
            multi_host: repo_root
                .join("scripts")
                .join("validation")
                .join("multi_host_validation.sh"),
            frontend_rpc_doc: repo_root
                .join("docs")
                .join("src")
                .join("FRONTEND_RPC_API_INTEGRATION_TR.md"),
            mainnet_checklist: repo_root
                .join("docs")
                .join("src")
                .join("MAINNET_READINESS_CHECKLIST.md"),
        }
    }
}
