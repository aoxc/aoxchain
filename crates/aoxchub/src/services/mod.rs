use crate::{
    binaries, commands,
    domain::{
        BinaryCandidate, BinarySourceKind, CommandProgram, CommandView, DashboardSnapshot,
        EnvironmentBinding, HubStateView, InstalledVersions,
    },
    environments::Environment,
    errors::HubError,
    runner::Runner,
};
use std::{
    collections::HashSet,
    net::{SocketAddr, TcpStream},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct HubService {
    pub environment: Arc<RwLock<Environment>>,
    pub binaries: Arc<RwLock<Vec<BinaryCandidate>>>,
    pub selected_binary_id: Arc<RwLock<Option<String>>>,
    pub runner: Runner,
}

impl HubService {
    pub fn new() -> Self {
        let bins = binaries::discover();
        let default_env = Environment::Mainnet;
        let selected = bins
            .iter()
            .find(|b| is_binary_allowed(default_env, &b.kind))
            .or_else(|| bins.first())
            .map(|b| b.id.clone());

        Self {
            environment: Arc::new(RwLock::new(default_env)),
            binaries: Arc::new(RwLock::new(bins)),
            selected_binary_id: Arc::new(RwLock::new(selected)),
            runner: Runner::new(),
        }
    }

    pub async fn state(&self) -> HubStateView {
        let environment = *self.environment.read().await;
        let refreshed = binaries::discover();
        {
            let mut bins_guard = self.binaries.write().await;
            *bins_guard = refreshed;
        }
        let bins = self.binaries.read().await.clone();
        let selected = self.selected_binary_id.read().await.clone();

        let commands = commands::CATALOG
            .iter()
            .map(|spec| {
                let preview = self.command_preview(environment, spec.program.clone(), spec.args);
                let allowed = self.is_command_allowed(environment, spec.id);
                let policy_note = policy_note(environment, spec.id, spec.risk.clone(), allowed);

                CommandView {
                    spec: spec.clone(),
                    preview,
                    allowed,
                    policy_note,
                }
            })
            .collect();

        let node_probe = probe_local_nodes(environment);

        HubStateView {
            environment,
            banner: environment.banner_text(),
            binding: EnvironmentBinding {
                slug: environment.slug(),
                root_config: environment.root_config_path(),
                aoxc_home: environment.aoxc_home(),
                make_scope: environment.make_scope(),
            },
            selected_binary_id: selected.clone(),
            binaries: bins.clone(),
            commands,
            dashboard: dashboard_snapshot(environment, &bins, selected.as_deref(), &node_probe),
        }
    }

    pub async fn set_environment(&self, env: Environment) {
        *self.environment.write().await = env;
        let bins = self.binaries.read().await.clone();
        let selected_current = self.selected_binary_id.read().await.clone();
        let selected_allowed = selected_current
            .as_ref()
            .and_then(|id| bins.iter().find(|b| &b.id == id))
            .map(|b| is_binary_allowed(env, &b.kind))
            .unwrap_or(false);

        if !selected_allowed {
            let fallback = bins
                .iter()
                .find(|b| is_binary_allowed(env, &b.kind))
                .or_else(|| bins.first())
                .map(|b| b.id.clone());
            *self.selected_binary_id.write().await = fallback;
        }
    }

    pub async fn set_binary(&self, id: String) -> Result<(), HubError> {
        let env = *self.environment.read().await;
        let bins = self.binaries.read().await;

        let candidate = bins
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| HubError::Security(String::from("selected binary does not exist")))?;

        if !is_binary_allowed(env, &candidate.kind) {
            return Err(HubError::Security(String::from(
                "selected binary source is forbidden in active environment",
            )));
        }

        *self.selected_binary_id.write().await = Some(id);
        Ok(())
    }

    pub async fn add_custom_binary(&self, path: String) -> Result<(), HubError> {
        let env = *self.environment.read().await;

        if env != Environment::Testnet {
            return Err(HubError::Security(String::from(
                "custom binary path is allowed only on testnet",
            )));
        }

        if self
            .binaries
            .read()
            .await
            .iter()
            .any(|candidate| candidate.path == path)
        {
            return Ok(());
        }

        let candidate = BinaryCandidate {
            id: format!("custom-{}", chrono::Utc::now().timestamp()),
            kind: BinarySourceKind::CustomPath,
            path,
            version: None,
            trust: crate::domain::TrustLevel::Unverified,
            checksum_verified: None,
        };

        self.binaries.write().await.push(candidate);
        Ok(())
    }

    pub fn is_command_allowed(&self, env: Environment, command_id: &str) -> bool {
        let Some(spec) = commands::find(command_id) else {
            return false;
        };

        if env == Environment::Mainnet && command_id == "testnet-start" {
            return false;
        }

        if env == Environment::Testnet && command_id == "mainnet-start" {
            return false;
        }

        if env == Environment::Mainnet
            && matches!(spec.risk, crate::domain::RiskClass::High)
            && !matches!(
                command_id,
                "mainnet-start" | "aoxc-node-start" | "aoxc-node-stop"
            )
        {
            return false;
        }

        true
    }

    fn command_preview(
        &self,
        env: Environment,
        program: CommandProgram,
        args: &[&'static str],
    ) -> String {
        let prefix = environment_prefix(env);

        match program {
            CommandProgram::Aoxc => format!("{prefix} aoxc {}", args.join(" ")),
            CommandProgram::Make => format!("{prefix} make {}", args.join(" ")),
        }
    }

    pub async fn execute(&self, command_id: String) -> Result<String, HubError> {
        let env = *self.environment.read().await;

        if !self.is_command_allowed(env, &command_id) {
            return Err(HubError::Security(String::from(
                "command is not available in active environment",
            )));
        }

        let spec = commands::find(&command_id)
            .ok_or_else(|| HubError::UnknownCommand(command_id.clone()))?;

        let (program, args) = match spec.program {
            CommandProgram::Make => (
                String::from("make"),
                spec.args.iter().map(|s| s.to_string()).collect(),
            ),
            CommandProgram::Aoxc => {
                let selected_id = self
                    .selected_binary_id
                    .read()
                    .await
                    .clone()
                    .ok_or_else(|| HubError::Security(String::from("aoxc binary not selected")))?;

                let bins = self.binaries.read().await;
                let bin = bins.iter().find(|b| b.id == selected_id).ok_or_else(|| {
                    HubError::Security(String::from("selected aoxc binary is unavailable"))
                })?;

                if !is_binary_allowed(env, &bin.kind) {
                    return Err(HubError::Security(String::from(
                        "selected binary source violates environment policy",
                    )));
                }

                (
                    bin.path.clone(),
                    spec.args.iter().map(|s| s.to_string()).collect(),
                )
            }
        };

        let id = format!("job-{}", chrono::Utc::now().timestamp_millis());

        self.runner
            .launch(
                id.clone(),
                command_id,
                program,
                args,
                environment_bindings(env),
                String::from("/workspace/aoxchain"),
            )
            .await?;

        Ok(id)
    }
}

fn merge_binaries_preserving_custom(
    existing: &[BinaryCandidate],
    discovered: Vec<BinaryCandidate>,
) -> Vec<BinaryCandidate> {
    let mut out = discovered;
    let mut seen_paths: HashSet<String> = out
        .iter()
        .map(|candidate| candidate.path.clone())
        .collect();
    for candidate in existing {
        if matches!(candidate.kind, BinarySourceKind::CustomPath)
            && seen_paths.insert(candidate.path.clone())
        {
            out.push(candidate.clone());
        }
    }
    out
}

impl Default for HubService {
    fn default() -> Self {
        Self::new()
    }
}

fn environment_prefix(env: Environment) -> String {
    format!(
        "AOXC_ENV={} AOXC_HOME={} AOXHUB_CONFIG={}",
        env.slug(),
        env.aoxc_home(),
        env.root_config_path()
    )
}

fn environment_bindings(env: Environment) -> Vec<(String, String)> {
    vec![
        (String::from("AOXC_ENV"), String::from(env.slug())),
        (String::from("AOXC_HOME"), String::from(env.aoxc_home())),
        (
            String::from("AOXHUB_CONFIG"),
            String::from(env.root_config_path()),
        ),
    ]
}

fn policy_note(
    env: Environment,
    command_id: &str,
    risk: crate::domain::RiskClass,
    allowed: bool,
) -> String {
    if !allowed {
        return String::from("Blocked by active environment policy");
    }

    if env == Environment::Mainnet && matches!(risk, crate::domain::RiskClass::High) {
        return format!("Allowed high-risk command in mainnet: {}", command_id);
    }

    String::from("Allowed by active environment policy")
}

pub fn is_binary_allowed(env: Environment, kind: &BinarySourceKind) -> bool {
    match env {
        Environment::Mainnet => matches!(
            kind,
            BinarySourceKind::InstalledRelease
                | BinarySourceKind::VersionedBundle
                | BinarySourceKind::GithubRelease
        ),
        Environment::Testnet => matches!(
            kind,
            BinarySourceKind::InstalledRelease
                | BinarySourceKind::VersionedBundle
                | BinarySourceKind::GithubRelease
                | BinarySourceKind::LocalReleaseBuild
                | BinarySourceKind::CustomPath
        ),
    }
}

/// Builds a deterministic dashboard snapshot for the active AOXCHUB environment.
///
/// Security rationale:
/// - This function is intentionally pure and side-effect free.
/// - No filesystem probing, subprocess execution, RPC access, or network I/O is performed.
/// - The snapshot is derived only from already-discovered in-memory state, which keeps
///   state rendering deterministic and prevents UI aggregation from mutating runtime state.
/// - Telemetry values are conservative placeholders until a dedicated metrics source is wired in.
fn dashboard_snapshot(
    env: Environment,
    bins: &[BinaryCandidate],
    selected_binary_id: Option<&str>,
    node_probe: &NodeProbeSnapshot,
) -> DashboardSnapshot {
    let installed_versions = installed_versions_snapshot(bins);

    let (chain_name, network_kind, network_id, validator_count, observer_count) = match env {
        Environment::Mainnet => (
            String::from("AOXC Mainnet"),
            String::from("mainnet"),
            String::from("aoxc-mainnet"),
            21,
            3,
        ),
        Environment::Testnet => (
            String::from("AOXC Testnet"),
            String::from("testnet"),
            String::from("aoxc-testnet"),
            21,
            0,
        ),
    };

    let binary_count = bins.len();
    let allowed_binary_count = bins
        .iter()
        .filter(|candidate| is_binary_allowed(env, &candidate.kind))
        .count();

    let mut last_events = vec![
        format!("Discovered {} binary candidate(s)", binary_count),
        format!(
            "{} binary candidate(s) allowed by active environment policy",
            allowed_binary_count
        ),
        format!("Active environment set to {}", env.slug()),
        format!(
            "Discovered {} local node endpoint candidate(s)",
            node_probe.total_candidates
        ),
    ];

    let mut last_warnings = Vec::new();
    let mut selected_binary_path = None;
    let mut selected_binary_allowed = None;

    if let Some(selected_id) = selected_binary_id {
        if let Some(selected_candidate) = bins.iter().find(|candidate| candidate.id == selected_id)
        {
            selected_binary_path = Some(selected_candidate.path.clone());
            selected_binary_allowed = Some(is_binary_allowed(env, &selected_candidate.kind));
            last_events.push(format!(
                "Selected binary source: {} ({:?})",
                selected_candidate.path, selected_candidate.kind
            ));
        } else {
            last_warnings.push(String::from(
                "Selected binary is missing from discovery snapshot; refresh binary selection",
            ));
        }
    } else {
        last_warnings.push(String::from(
            "No AOXC binary is currently selected; command execution is blocked until selection",
        ));
    }

    if binary_count == 0 {
        last_warnings.push(String::from(
            "No AOXC binary candidates were discovered in the current host context",
        ));
    }

    if allowed_binary_count == 0 {
        last_warnings.push(String::from(
            "No discovered binary satisfies the active environment execution policy",
        ));
    }

    if bins
        .iter()
        .any(|candidate| matches!(candidate.kind, BinarySourceKind::CustomPath))
    {
        last_warnings.push(String::from(
            "Custom-path binaries require explicit operator trust validation before execution",
        ));
    }

    if selected_binary_allowed.is_some_and(|allowed| !allowed) {
        last_warnings.push(String::from(
            "Selected binary source is not allowed in the active environment policy",
        ));
    }

    if let Some(selected_endpoint) = &node_probe.selected_endpoint {
        let latency_note = node_probe
            .selected_latency_ms
            .map(|latency| format!(" ({}ms)", latency))
            .unwrap_or_default();
        last_events.push(format!(
            "Auto-selected local node endpoint: {}{}",
            selected_endpoint, latency_note
        ));
    } else {
        last_warnings.push(String::from(
            "No reachable local node endpoint detected; node actions stay available but telemetry remains offline",
        ));
    }

    let last_txs = vec![String::from(
        "No recent transaction data available in offline dashboard mode",
    )];

    let quick_actions = match env {
        Environment::Mainnet => vec![
            String::from("mainnet-start"),
            String::from("mainnet-status"),
            String::from("aoxc-node-start"),
            String::from("aoxc-node-stop"),
        ],
        Environment::Testnet => vec![
            String::from("testnet-start"),
            String::from("testnet-status"),
            String::from("aoxc-node-start"),
            String::from("aoxc-node-stop"),
        ],
    };

    DashboardSnapshot {
        selected_binary_id: selected_binary_id.map(ToOwned::to_owned),
        selected_binary_path,
        selected_binary_allowed,
        chain_name,
        network_kind,
        network_id,
        current_height: 0,
        finalized_height: 0,
        current_round: 0,
        validator_count,
        observer_count,
        connected_peers: 0,
        local_node_status: node_probe.local_node_status.clone(),
        rpc_status: node_probe.rpc_status.clone(),
        p2p_status: node_probe.p2p_status.clone(),
        genesis_fingerprint: genesis_fingerprint(env),
        health_status: health_status(binary_count, allowed_binary_count, selected_binary_allowed),
        installed_versions,
        last_events,
        last_txs,
        last_warnings,
        quick_actions,
    }
}

#[derive(Clone, Debug)]
struct NodeProbeSnapshot {
    total_candidates: usize,
    selected_endpoint: Option<String>,
    selected_latency_ms: Option<u128>,
    local_node_status: String,
    rpc_status: String,
    p2p_status: String,
}

fn probe_local_nodes(env: Environment) -> NodeProbeSnapshot {
    let mut rpc_candidates = endpoint_candidates(env);
    let p2p_candidates = p2p_candidates(env);

    if let Ok(raw) = std::env::var("AOXCHUB_NODE_RPC_CANDIDATES") {
        let custom: Vec<String> = raw
            .split(',')
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        if !custom.is_empty() {
            rpc_candidates = custom;
        }
    }

    let best_rpc = select_reachable_endpoint(&rpc_candidates);
    let p2p = select_reachable_endpoint(&p2p_candidates);

    let selected_endpoint = best_rpc.as_ref().map(|probe| probe.endpoint.clone());
    let selected_latency_ms = best_rpc.as_ref().map(|probe| probe.latency_ms);
    let local_node_status = if selected_endpoint.is_some() {
        String::from("online")
    } else {
        String::from("idle")
    };

    let rpc_status = best_rpc
        .as_ref()
        .map(|probe| format!("connected:{} ({}ms)", probe.endpoint, probe.latency_ms))
        .unwrap_or_else(|| String::from("not_connected"));

    let p2p_status = p2p
        .as_ref()
        .map(|probe| format!("reachable:{} ({}ms)", probe.endpoint, probe.latency_ms))
        .unwrap_or_else(|| String::from("not_connected"));

    NodeProbeSnapshot {
        total_candidates: rpc_candidates.len(),
        selected_endpoint,
        selected_latency_ms,
        local_node_status,
        rpc_status,
        p2p_status,
    }
}

#[derive(Clone, Debug)]
struct EndpointProbe {
    endpoint: String,
    latency_ms: u128,
}

fn select_reachable_endpoint(candidates: &[String]) -> Option<EndpointProbe> {
    candidates
        .iter()
        .filter_map(|endpoint| probe_endpoint(endpoint))
        .min_by_key(|probe| probe.latency_ms)
}

fn probe_endpoint(endpoint: &str) -> Option<EndpointProbe> {
    let addr = SocketAddr::from_str(endpoint).ok()?;
    let started = Instant::now();
    let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(220)).ok()?;
    let _ = stream.set_nonblocking(true);
    let latency_ms = started.elapsed().as_millis();
    Some(EndpointProbe {
        endpoint: endpoint.to_string(),
        latency_ms,
    })
}

fn endpoint_candidates(env: Environment) -> Vec<String> {
    match env {
        Environment::Mainnet => vec![
            String::from("127.0.0.1:8545"),
            String::from("127.0.0.1:26657"),
            String::from("127.0.0.1:9933"),
        ],
        Environment::Testnet => vec![
            String::from("127.0.0.1:18545"),
            String::from("127.0.0.1:36657"),
            String::from("127.0.0.1:19933"),
        ],
    }
}

fn p2p_candidates(env: Environment) -> Vec<String> {
    match env {
        Environment::Mainnet => vec![
            String::from("127.0.0.1:30333"),
            String::from("127.0.0.1:26656"),
        ],
        Environment::Testnet => vec![
            String::from("127.0.0.1:31333"),
            String::from("127.0.0.1:36656"),
        ],
    }
}

/// Produces a conservative installed-version snapshot from locally discovered binaries.
///
/// Security rationale:
/// - Only already-discovered in-memory metadata is used.
/// - No binary execution is performed to infer versions.
/// - Unknown values are represented explicitly rather than guessed.
fn installed_versions_snapshot(bins: &[BinaryCandidate]) -> InstalledVersions {
    let aoxc_version = bins
        .iter()
        .find_map(|candidate| candidate.version.clone())
        .unwrap_or_else(|| String::from("unknown"));

    InstalledVersions {
        aoxc: aoxc_version,
        aoxchub: env!("CARGO_PKG_VERSION").to_string(),
        runtime: String::from("rust"),
    }
}

/// Returns a deterministic environment fingerprint label for dashboard rendering.
///
/// This value is informational only and must not be treated as a cryptographic genesis hash.
fn genesis_fingerprint(env: Environment) -> String {
    match env {
        Environment::Mainnet => String::from("mainnet-genesis-unavailable"),
        Environment::Testnet => String::from("testnet-genesis-unavailable"),
    }
}

/// Derives a coarse-grained health label from locally available discovery data.
///
/// The result is intentionally conservative and does not claim live chain liveness.
fn health_status(
    binary_count: usize,
    allowed_binary_count: usize,
    selected_binary_allowed: Option<bool>,
) -> String {
    if binary_count == 0 {
        return String::from("degraded");
    }

    if allowed_binary_count == 0 {
        return String::from("restricted");
    }

    if matches!(selected_binary_allowed, Some(false)) {
        return String::from("restricted");
    }

    if selected_binary_allowed.is_none() {
        return String::from("degraded");
    }

    String::from("nominal")
}
