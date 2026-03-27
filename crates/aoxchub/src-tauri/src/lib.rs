#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::process::Command;

type AppResult<T> = Result<T, String>;

const AOXC_BIN_PACKAGE: &str = "aoxcmd";
const AOXC_PROGRESS_REPORT: &str = "AOXC_PROGRESS_REPORT.md";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadinessTrack {
    name: String,
    percent: u8,
    summary: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LaunchBlocker {
    title: String,
    detail: String,
    command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileStatus {
    label: String,
    path: String,
    exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AreaProgress {
    name: String,
    percent: u8,
    detail: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NodeControl {
    id: String,
    role: String,
    status: String,
    chain_id: String,
    listen_addr: String,
    rpc_addr: String,
    peer_count: usize,
    security_mode: String,
    command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WalletSurface {
    title: String,
    route: String,
    status: String,
    address_hint: String,
    command: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelemetrySurface {
    title: String,
    status: String,
    target: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportAsset {
    title: String,
    status: String,
    path: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandPreset {
    title: String,
    command: String,
    intent: String,
    risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSurface {
    name: String,
    path: String,
    category: String,
    status: String,
    summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiSurface {
    name: String,
    area: String,
    status: String,
    summary: String,
    command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentSurface {
    kind: EnvironmentKind,
    label: String,
    profile: String,
    config_path: String,
    home_path: String,
    chain_id: String,
    rpc_addr: String,
    security_mode: String,
    readiness_status: String,
    config_exists: bool,
    home_exists: bool,
    genesis_exists: bool,
    notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecutionArtifact {
    label: String,
    path: String,
    exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandExecutionResult {
    action: DesktopAction,
    environment: EnvironmentKind,
    status: ExecutionStatus,
    risk_level: RiskLevel,
    command: String,
    args: Vec<String>,
    working_directory: String,
    started_at_unix_ms: u128,
    finished_at_unix_ms: u128,
    duration_ms: u128,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    artifacts: Vec<ExecutionArtifact>,
    next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandExecutionRequest {
    action: DesktopAction,
    environment: EnvironmentKind,
    options: Option<ActionOptions>,
    confirmation_phrase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ActionOptions {
    home: Option<String>,
    profile: Option<String>,
    format: Option<String>,
    rounds: Option<u32>,
    sleep_ms: Option<u64>,
    redact: Option<bool>,
    enforce: Option<bool>,
    scenario: Option<String>,
    backend: Option<String>,
    block_file: Option<String>,
    height: Option<u64>,
    hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ControlCenterSnapshot {
    stage: String,
    verdict: String,
    overall_percent: u8,
    profile: String,
    summary: String,
    tracks: Vec<ReadinessTrack>,
    blockers: Vec<LaunchBlocker>,
    files: Vec<FileStatus>,
    areas: Vec<AreaProgress>,
    nodes: Vec<NodeControl>,
    wallets: Vec<WalletSurface>,
    telemetry: Vec<TelemetrySurface>,
    reports: Vec<ReportAsset>,
    commands: Vec<CommandPreset>,
    workspaces: Vec<WorkspaceSurface>,
    ai_surfaces: Vec<AiSurface>,
    environments: Vec<EnvironmentSurface>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum EnvironmentKind {
    Localnet,
    Testnet,
    Mainnet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum DesktopAction {
    ProductionBootstrap,
    MainnetReadiness,
    FullSurfaceReadiness,
    RuntimeStatus,
    ProductionAudit,
    DiagnosticsBundle,
    ConfigValidate,
    ConfigPrint,
    NodeBootstrap,
    ProduceOnce,
    NodeHealth,
    NodeRun,
    NetworkSmoke,
    RealNetworkValidation,
    DbInit,
    DbStatus,
    DbPutBlock,
    DbGetHeight,
    DbGetHash,
    DbCompact,
    LaunchDeterministicCluster,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum ExecutionStatus {
    Succeeded,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum RiskLevel {
    Low,
    Medium,
    High,
}

#[tauri::command]
fn load_control_center_snapshot() -> AppResult<ControlCenterSnapshot> {
    let repo_root = repo_root()?;
    let report_path = repo_root.join(AOXC_PROGRESS_REPORT);
    let report = fs::read_to_string(&report_path)
        .map_err(|err| format!("failed to read {}: {err}", report_path.display()))?;

    let profile = capture_value(&report, "- Profile:")?;
    let stage = capture_value(&report, "- Stage:")?;
    let verdict = capture_value(&report, "- Verdict:")?
        .trim_matches('`')
        .to_string();

    let overall_percent = capture_percent(&report, "- Overall readiness:")?;
    let testnet_percent = capture_percent(&report, "- **testnet**:")?;
    let mainnet_percent = capture_percent(&report, "- **mainnet**:")?;

    let blockers = capture_blockers(&report);
    let areas = capture_area_progress(&report);
    let nodes = discover_node_controls(&repo_root)?;
    let files = control_files(&repo_root);
    let reports = discover_report_assets(&repo_root);
    let telemetry = discover_telemetry_surfaces(&repo_root);
    let wallets = wallet_surfaces();
    let commands = command_presets();
    let workspaces = discover_workspace_surfaces(&repo_root)?;
    let ai_surfaces = discover_ai_surfaces(&repo_root)?;
    let environments = discover_environments(&repo_root);

    let desktop_surface_percent = desktop_percent(overall_percent, &nodes, &reports);

    let summary = format!(
        "{} blocker(s), {} environment(s), {} node surface(s), {} report asset(s), {} workspace surface(s), and {} AI surface(s) are exposed through AOXHub desktop.",
        blockers.len(),
        environments.len(),
        nodes.len(),
        reports.len(),
        workspaces.len(),
        ai_surfaces.len()
    );

    Ok(ControlCenterSnapshot {
        stage: stage.clone(),
        verdict,
        overall_percent,
        profile: profile.clone(),
        summary,
        tracks: vec![
            ReadinessTrack {
                name: "Mainnet readiness".to_string(),
                percent: mainnet_percent,
                summary: "Production controls must all pass before mainnet promotion.".to_string(),
                status: status_from_percent(mainnet_percent).to_string(),
            },
            ReadinessTrack {
                name: "Testnet readiness".to_string(),
                percent: testnet_percent,
                summary:
                    "Testnet should close non-mainnet blockers and sustain AOXHub/core parity."
                        .to_string(),
                status: status_from_percent(testnet_percent).to_string(),
            },
            ReadinessTrack {
                name: "Desktop control center".to_string(),
                percent: desktop_surface_percent,
                summary: format!(
                    "Current release stage: {stage}. Active profile: {profile}. Desktop panel unifies node, wallet, telemetry, evidence, workspace, and operator control planes."
                ),
                status: status_from_percent(desktop_surface_percent).to_string(),
            },
        ],
        blockers,
        files,
        areas,
        nodes,
        wallets,
        telemetry,
        reports,
        commands,
        workspaces,
        ai_surfaces,
        environments,
    })
}

#[tauri::command]
async fn run_desktop_action(request: CommandExecutionRequest) -> AppResult<CommandExecutionResult> {
    let repo_root = repo_root()?;
    validate_action_request(&request)?;

    let spec = build_action_spec(&repo_root, &request)?;
    let started = unix_ms_now();

    let mut command = if spec.program == "script" {
        let script_path = spec
            .script_path
            .clone()
            .ok_or_else(|| "missing script path for script action".to_string())?;
        let cmd = Command::new(script_path);
        cmd
    } else {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("-q")
            .arg("-p")
            .arg(AOXC_BIN_PACKAGE)
            .arg("--");
        for arg in &spec.args {
            cmd.arg(arg);
        }
        cmd
    };

    command
        .current_dir(&repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (key, value) in spec.env_vars.iter() {
        command.env(key, value);
    }

    let output = command.output().await.map_err(|err| {
        format!(
            "failed to execute desktop action {:?}: {err}",
            request.action
        )
    })?;

    let finished = unix_ms_now();
    let duration_ms = finished.saturating_sub(started);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = if output.status.success() {
        ExecutionStatus::Succeeded
    } else {
        ExecutionStatus::Failed
    };

    Ok(CommandExecutionResult {
        action: request.action,
        environment: request.environment,
        status,
        risk_level: risk_level_for(request.action),
        command: spec.rendered_command,
        args: spec.args.clone(),
        working_directory: repo_root.display().to_string(),
        started_at_unix_ms: started,
        finished_at_unix_ms: finished,
        duration_ms,
        exit_code: output.status.code(),
        stdout,
        stderr,
        artifacts: resolve_artifacts(&repo_root, &spec.artifact_hints),
        next_steps: next_steps_for(request.action, request.environment, status),
    })
}

fn validate_action_request(request: &CommandExecutionRequest) -> AppResult<()> {
    let risk = risk_level_for(request.action);

    if risk == RiskLevel::High {
        let provided = request.confirmation_phrase.clone().unwrap_or_default();
        if provided.trim() != "AOXC_DESKTOP_CONFIRM" {
            return Err(
                "high-risk desktop action rejected: confirmation phrase AOXC_DESKTOP_CONFIRM is required"
                    .to_string(),
            );
        }
    }

    if let Some(options) = &request.options {
        if let Some(rounds) = options.rounds {
            if rounds == 0 || rounds > 10_000 {
                return Err("invalid rounds value: expected 1..=10000".to_string());
            }
        }

        if let Some(sleep_ms) = options.sleep_ms {
            if sleep_ms == 0 || sleep_ms > 60_000 {
                return Err("invalid sleepMs value: expected 1..=60000".to_string());
            }
        }

        if let Some(profile) = &options.profile {
            if !is_safe_token(profile) {
                return Err("invalid profile value".to_string());
            }
        }

        if let Some(format) = &options.format {
            if !matches!(format.as_str(), "json" | "text") {
                return Err("invalid format value: expected json or text".to_string());
            }
        }

        if let Some(scenario) = &options.scenario {
            if !is_safe_token(scenario) {
                return Err("invalid scenario value".to_string());
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct ActionSpec {
    program: String,
    args: Vec<String>,
    rendered_command: String,
    script_path: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
    artifact_hints: Vec<(String, String)>,
}

fn build_action_spec(repo_root: &Path, request: &CommandExecutionRequest) -> AppResult<ActionSpec> {
    let env_surface = environment_surface_for(repo_root, request.environment);
    let options = request.options.clone().unwrap_or_default();

    let format = options.format.unwrap_or_else(|| "json".to_string());
    let profile = options
        .profile
        .unwrap_or_else(|| env_surface.profile.clone());
    let home = options
        .home
        .unwrap_or_else(|| env_surface.home_path.clone());

    let mut env_vars = vec![("AOXC_HOME".to_string(), home.clone())];

    let spec = match request.action {
        DesktopAction::MainnetReadiness => {
            let mut args = vec!["mainnet-readiness".to_string()];
            if options.enforce.unwrap_or(true) {
                args.push("--enforce".to_string());
            }
            args.push("--format".to_string());
            args.push(format.clone());

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![(
                    "Progress report".to_string(),
                    AOXC_PROGRESS_REPORT.to_string(),
                )],
            }
        }
        DesktopAction::ProductionBootstrap => {
            let scenario = options
                .scenario
                .clone()
                .unwrap_or_else(|| "desktop-bootstrap-anchor".to_string());
            let args = vec![
                "production-bootstrap".to_string(),
                "--home".to_string(),
                home.clone(),
                "--profile".to_string(),
                profile.clone(),
                "--name".to_string(),
                "desktop-operator".to_string(),
                "--password".to_string(),
                "Desktop#2026!".to_string(),
                "--produce-once-tx".to_string(),
                scenario,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![
                    ("Node home".to_string(), home.clone()),
                    (
                        "Progress report".to_string(),
                        AOXC_PROGRESS_REPORT.to_string(),
                    ),
                ],
            }
        }
        DesktopAction::FullSurfaceReadiness => {
            let mut args = vec!["full-surface-readiness".to_string()];
            if options.enforce.unwrap_or(true) {
                args.push("--enforce".to_string());
            }
            args.push("--format".to_string());
            args.push(format.clone());

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![
                    (
                        "Progress report".to_string(),
                        AOXC_PROGRESS_REPORT.to_string(),
                    ),
                    (
                        "Release evidence bundle".to_string(),
                        "artifacts/release-evidence".to_string(),
                    ),
                ],
            }
        }
        DesktopAction::RuntimeStatus => {
            let args = vec![
                "runtime-status".to_string(),
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![(
                    "Network closure runtime status".to_string(),
                    "artifacts/network-production-closure/runtime-status.json".to_string(),
                )],
            }
        }
        DesktopAction::ProductionAudit => {
            let args = vec![
                "production-audit".to_string(),
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![
                    (
                        "Release evidence bundle".to_string(),
                        "artifacts/release-evidence".to_string(),
                    ),
                    (
                        "Network closure production audit".to_string(),
                        "artifacts/network-production-closure/production-audit.json".to_string(),
                    ),
                ],
            }
        }
        DesktopAction::DiagnosticsBundle => {
            let mut args = vec!["diagnostics-bundle".to_string()];
            if options.redact.unwrap_or(true) {
                args.push("--redact".to_string());
            }

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Diagnostics bundle".to_string(), "artifacts".to_string())],
            }
        }
        DesktopAction::ConfigValidate => {
            let args = vec![
                "config-validate".to_string(),
                "--profile".to_string(),
                profile.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![(
                    "Configuration profile".to_string(),
                    env_surface.config_path.clone(),
                )],
            }
        }
        DesktopAction::ConfigPrint => {
            let args = vec![
                "config-print".to_string(),
                "--profile".to_string(),
                profile.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![(
                    "Configuration profile".to_string(),
                    env_surface.config_path.clone(),
                )],
            }
        }
        DesktopAction::NodeBootstrap => {
            let args = vec![
                "node-bootstrap".to_string(),
                "--home".to_string(),
                home.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![
                    ("Node home".to_string(), home.clone()),
                    (
                        "Progress report".to_string(),
                        AOXC_PROGRESS_REPORT.to_string(),
                    ),
                ],
            }
        }
        DesktopAction::ProduceOnce => {
            let tx = options
                .scenario
                .clone()
                .unwrap_or_else(|| "desktop-produce-once".to_string());
            let args = vec![
                "produce-once".to_string(),
                "--home".to_string(),
                home.clone(),
                "--tx".to_string(),
                tx,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Node home".to_string(), home.clone())],
            }
        }
        DesktopAction::NodeHealth => {
            let args = vec![
                "node-health".to_string(),
                "--home".to_string(),
                home.clone(),
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Node home".to_string(), home.clone())],
            }
        }
        DesktopAction::NodeRun => {
            let rounds = options.rounds.unwrap_or(12);
            let sleep_ms = options.sleep_ms.unwrap_or(200);

            let args = vec![
                "node-run".to_string(),
                "--home".to_string(),
                home.clone(),
                "--rounds".to_string(),
                rounds.to_string(),
                "--sleep-ms".to_string(),
                sleep_ms.to_string(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Node home".to_string(), home.clone())],
            }
        }
        DesktopAction::NetworkSmoke => {
            let args = vec![
                "network-smoke".to_string(),
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![(
                    "Network production closure".to_string(),
                    "artifacts/network-production-closure".to_string(),
                )],
            }
        }
        DesktopAction::RealNetworkValidation => {
            let args = vec![
                "real-network".to_string(),
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![(
                    "Network production closure".to_string(),
                    "artifacts/network-production-closure".to_string(),
                )],
            }
        }
        DesktopAction::DbInit => {
            let backend = options
                .backend
                .clone()
                .unwrap_or_else(|| "sqlite".to_string());
            let args = vec![
                "db-init".to_string(),
                "--backend".to_string(),
                backend,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Runtime DB root".to_string(), format!("{home}/runtime/db"))],
            }
        }
        DesktopAction::DbStatus => {
            let backend = options
                .backend
                .clone()
                .unwrap_or_else(|| "sqlite".to_string());
            let args = vec![
                "db-status".to_string(),
                "--backend".to_string(),
                backend,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Runtime DB root".to_string(), format!("{home}/runtime/db"))],
            }
        }
        DesktopAction::DbPutBlock => {
            let backend = options
                .backend
                .clone()
                .unwrap_or_else(|| "sqlite".to_string());
            let block_file = options.block_file.clone().ok_or_else(|| {
                "dbPutBlock action requires options.blockFile (path to block envelope json)"
                    .to_string()
            })?;
            let args = vec![
                "db-put-block".to_string(),
                "--block-file".to_string(),
                block_file,
                "--backend".to_string(),
                backend,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Runtime DB root".to_string(), format!("{home}/runtime/db"))],
            }
        }
        DesktopAction::DbGetHeight => {
            let backend = options
                .backend
                .clone()
                .unwrap_or_else(|| "sqlite".to_string());
            let height = options.height.ok_or_else(|| {
                "dbGetHeight action requires options.height (u64 block height)".to_string()
            })?;
            let args = vec![
                "db-get-height".to_string(),
                "--height".to_string(),
                height.to_string(),
                "--backend".to_string(),
                backend,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Runtime DB root".to_string(), format!("{home}/runtime/db"))],
            }
        }
        DesktopAction::DbGetHash => {
            let backend = options
                .backend
                .clone()
                .unwrap_or_else(|| "sqlite".to_string());
            let hash = options
                .hash
                .clone()
                .ok_or_else(|| "dbGetHash action requires options.hash (hex hash)".to_string())?;
            let args = vec![
                "db-get-hash".to_string(),
                "--hash".to_string(),
                hash,
                "--backend".to_string(),
                backend,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Runtime DB root".to_string(), format!("{home}/runtime/db"))],
            }
        }
        DesktopAction::DbCompact => {
            let backend = options
                .backend
                .clone()
                .unwrap_or_else(|| "sqlite".to_string());
            let args = vec![
                "db-compact".to_string(),
                "--backend".to_string(),
                backend,
                "--format".to_string(),
                format.clone(),
            ];

            ActionSpec {
                program: "cargo".to_string(),
                rendered_command: render_cargo_command(&args),
                args,
                script_path: None,
                env_vars,
                artifact_hints: vec![("Runtime DB root".to_string(), format!("{home}/runtime/db"))],
            }
        }
        DesktopAction::LaunchDeterministicCluster => {
            if request.environment == EnvironmentKind::Mainnet {
                return Err(
                    "launch-deterministic-cluster is not permitted for mainnet environment"
                        .to_string(),
                );
            }

            let script_rel = "configs/environments/localnet/launch-localnet.sh";
            let script_path = repo_root.join(script_rel);

            if !script_path.exists() {
                return Err(format!(
                    "missing deterministic cluster launcher: {}",
                    script_path.display()
                ));
            }

            env_vars.push(("AOXC_PROFILE".to_string(), env_surface.profile.clone()));

            ActionSpec {
                program: "script".to_string(),
                rendered_command: script_rel.to_string(),
                args: vec![script_rel.to_string()],
                script_path: Some(script_path),
                env_vars,
                artifact_hints: vec![
                    ("Deterministic launcher".to_string(), script_rel.to_string()),
                    (
                        "Deterministic nodes".to_string(),
                        "configs/environments/localnet/nodes".to_string(),
                    ),
                    (
                        "Deterministic homes".to_string(),
                        "configs/environments/localnet/homes".to_string(),
                    ),
                ],
            }
        }
    };

    Ok(spec)
}

fn render_cargo_command(args: &[String]) -> String {
    let mut out = vec![
        "cargo".to_string(),
        "run".to_string(),
        "-q".to_string(),
        "-p".to_string(),
        AOXC_BIN_PACKAGE.to_string(),
        "--".to_string(),
    ];
    out.extend(args.iter().cloned());
    out.join(" ")
}

fn resolve_artifacts(repo_root: &Path, hints: &[(String, String)]) -> Vec<ExecutionArtifact> {
    hints
        .iter()
        .map(|(label, rel)| {
            let full = repo_root.join(rel);
            ExecutionArtifact {
                label: label.clone(),
                path: rel.clone(),
                exists: full.exists(),
            }
        })
        .collect()
}

fn next_steps_for(
    action: DesktopAction,
    environment: EnvironmentKind,
    status: ExecutionStatus,
) -> Vec<String> {
    if status == ExecutionStatus::Failed {
        return vec![
            "Inspect stderr and exit code in the execution panel.".to_string(),
            "Verify AOXC_HOME, config profile, and environment-specific artifacts.".to_string(),
            "Run production-audit or diagnostics-bundle before retrying.".to_string(),
        ];
    }

    match action {
        DesktopAction::ProductionBootstrap => vec![
            "Run mainnet-readiness and full-surface-readiness after bootstrap.".to_string(),
            "Verify produced block metadata from runtime-status.".to_string(),
        ],
        DesktopAction::MainnetReadiness => vec![
            "Review remaining blockers in AOXC_PROGRESS_REPORT.md.".to_string(),
            "Run full-surface-readiness for a broader closure decision.".to_string(),
            "Export release evidence before any promotion decision.".to_string(),
        ],
        DesktopAction::FullSurfaceReadiness => vec![
            "Review release evidence and closure artifacts.".to_string(),
            "Verify signature/provenance posture before promotion.".to_string(),
            format!(
                "Confirm {:?} environment policy gates are fully closed.",
                environment
            ),
        ],
        DesktopAction::RuntimeStatus => vec![
            "Check node-health for the selected home.".to_string(),
            "Compare runtime state against current readiness score.".to_string(),
        ],
        DesktopAction::ProductionAudit => vec![
            "Store audit output under release evidence records.".to_string(),
            "Re-run after any configuration or key lifecycle change.".to_string(),
        ],
        DesktopAction::DiagnosticsBundle => vec![
            "Archive the bundle for incident and recovery review.".to_string(),
            "Redact sensitive output before sharing externally.".to_string(),
        ],
        DesktopAction::ConfigValidate => vec![
            "If validation passed, print the resolved profile for operator review.".to_string(),
            "Proceed to node-bootstrap after configuration closure.".to_string(),
        ],
        DesktopAction::ConfigPrint => vec![
            "Review chain ID, RPC target, and security mode.".to_string(),
            "Run config-validate before bootstrapping the node.".to_string(),
        ],
        DesktopAction::NodeBootstrap => vec![
            "Run node-health after bootstrapping.".to_string(),
            "Run node-run for local validation or controlled smoke testing.".to_string(),
        ],
        DesktopAction::ProduceOnce => vec![
            "Run runtime-status to confirm height increment and consensus fields.".to_string(),
            "Run node-health and network-smoke after block production.".to_string(),
        ],
        DesktopAction::NodeHealth => vec![
            "If health is degraded, inspect logs and diagnostics.".to_string(),
            "Compare health output with runtime-status and network-smoke.".to_string(),
        ],
        DesktopAction::NodeRun => vec![
            "Check node-health and runtime-status immediately after the run.".to_string(),
            "Capture artifacts if this run is part of a readiness closure.".to_string(),
        ],
        DesktopAction::NetworkSmoke => vec![
            "Escalate to real-network validation after smoke closure.".to_string(),
            "Attach any generated artifacts to the closure report.".to_string(),
        ],
        DesktopAction::RealNetworkValidation => vec![
            "Store results under network-production-closure artifacts.".to_string(),
            "Review closure evidence before testnet or mainnet promotion.".to_string(),
        ],
        DesktopAction::DbInit => vec![
            "Ingest canonical block envelopes with db-put-block.".to_string(),
            "Run db-status to verify index and CAS surfaces.".to_string(),
        ],
        DesktopAction::DbStatus => vec![
            "If object/index counts drift, run db-compact.".to_string(),
            "Use db-get-height/hash to verify deterministic retrieval.".to_string(),
        ],
        DesktopAction::DbPutBlock => vec![
            "Use db-get-height and db-get-hash to verify the inserted block.".to_string(),
            "Compact the index periodically for long-running nodes.".to_string(),
        ],
        DesktopAction::DbGetHeight => vec![
            "Cross-check payload and hash against expected chain records.".to_string(),
            "Use db-get-hash for direct hash-based verification.".to_string(),
        ],
        DesktopAction::DbGetHash => vec![
            "Validate returned block height and parent linkage.".to_string(),
            "Run db-status to monitor CAS/index growth.".to_string(),
        ],
        DesktopAction::DbCompact => vec![
            "Re-run db-status to confirm compaction result.".to_string(),
            "Archive snapshots as part of release evidence when needed.".to_string(),
        ],
        DesktopAction::LaunchDeterministicCluster => vec![
            "Verify node cards and telemetry surfaces in AOXHub.".to_string(),
            "Run network-smoke and production-audit from the desktop control plane.".to_string(),
        ],
    }
}

fn risk_level_for(action: DesktopAction) -> RiskLevel {
    match action {
        DesktopAction::MainnetReadiness
        | DesktopAction::ProductionBootstrap
        | DesktopAction::FullSurfaceReadiness
        | DesktopAction::RuntimeStatus
        | DesktopAction::ProductionAudit
        | DesktopAction::DiagnosticsBundle
        | DesktopAction::ConfigValidate
        | DesktopAction::ConfigPrint
        | DesktopAction::NodeHealth
        | DesktopAction::NetworkSmoke
        | DesktopAction::DbInit
        | DesktopAction::DbStatus
        | DesktopAction::DbPutBlock
        | DesktopAction::DbGetHeight
        | DesktopAction::DbGetHash
        | DesktopAction::DbCompact => RiskLevel::Low,

        DesktopAction::NodeBootstrap
        | DesktopAction::ProduceOnce
        | DesktopAction::NodeRun
        | DesktopAction::RealNetworkValidation
        | DesktopAction::LaunchDeterministicCluster => RiskLevel::Medium,
    }
}

fn repo_root() -> AppResult<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .ok_or_else(|| "failed to resolve repository root".to_string())
}

fn capture_value(report: &str, prefix: &str) -> AppResult<String> {
    report
        .lines()
        .find_map(|line| line.strip_prefix(prefix).map(str::trim))
        .map(str::to_string)
        .ok_or_else(|| format!("missing line starting with {prefix}"))
}

fn capture_percent(report: &str, prefix: &str) -> AppResult<u8> {
    let line = report
        .lines()
        .find(|line| line.starts_with(prefix))
        .ok_or_else(|| format!("missing percentage line starting with {prefix}"))?;

    parse_percent_from_line(line)
        .ok_or_else(|| format!("failed to parse percentage from line: {line}"))
}

fn parse_percent_from_line(line: &str) -> Option<u8> {
    let bytes = line.as_bytes();
    let mut idx = 0usize;

    while idx < bytes.len() {
        if bytes[idx].is_ascii_digit() {
            let start = idx;
            while idx < bytes.len() && bytes[idx].is_ascii_digit() {
                idx += 1;
            }

            let number = &line[start..idx];
            let remainder = &line[idx..];

            if remainder.trim_start().starts_with('%') {
                return number.parse::<u8>().ok();
            }
        } else {
            idx += 1;
        }
    }

    None
}

fn capture_blockers(report: &str) -> Vec<LaunchBlocker> {
    collect_section_items(report, "## Remaining blockers")
        .into_iter()
        .filter_map(|line| {
            let rest = line.strip_prefix("- ")?;
            let (key, detail) = rest.split_once(':')?;
            let normalized_key = key.trim();
            let normalized_detail = detail.trim();

            if normalized_key.is_empty() || normalized_detail.is_empty() {
                return None;
            }

            Some(LaunchBlocker {
                title: humanize_key(normalized_key),
                detail: normalized_detail.to_string(),
                command: remediation_for(normalized_key).to_string(),
            })
        })
        .collect()
}

fn capture_area_progress(report: &str) -> Vec<AreaProgress> {
    collect_section_items(report, "## Area progress")
        .into_iter()
        .filter_map(parse_area_progress)
        .collect()
}

fn collect_section_items<'a>(report: &'a str, header: &str) -> Vec<&'a str> {
    let mut in_section = false;
    let mut out = Vec::new();

    for line in report.lines() {
        let trimmed = line.trim();

        if trimmed == header {
            in_section = true;
            continue;
        }

        if in_section && trimmed.starts_with("## ") {
            break;
        }

        if in_section && trimmed.starts_with("- ") {
            out.push(trimmed);
        }
    }

    out
}

fn parse_area_progress(line: &str) -> Option<AreaProgress> {
    let rest = line.strip_prefix("- **")?;
    let (name, tail) = rest.split_once("**:")?;

    let percent = parse_percent_from_line(tail)?;
    let status = if tail.contains("— ready") {
        "ready"
    } else {
        "in-progress"
    };

    let detail = tail
        .split('—')
        .next()
        .unwrap_or(tail)
        .trim()
        .trim_matches('-')
        .trim()
        .to_string();

    Some(AreaProgress {
        name: humanize_key(name),
        percent,
        detail,
        status: status.to_string(),
    })
}

fn discover_node_controls(repo_root: &Path) -> AppResult<Vec<NodeControl>> {
    let nodes_dir = repo_root.join("configs/deterministic-testnet/nodes");

    let mut paths = fs::read_dir(&nodes_dir)
        .map_err(|err| format!("failed to read {}: {err}", nodes_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("toml")))
        .collect::<Vec<_>>();

    paths.sort();

    paths.into_iter()
        .map(|path| {
            let content = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;

            let node_name = config_value(&content, "node_name").unwrap_or_else(|| "node".into());
            let chain_id = config_value(&content, "chain_id").unwrap_or_else(|| "unknown".into());
            let listen_addr =
                config_value(&content, "listen_addr").unwrap_or_else(|| "127.0.0.1:0".into());
            let rpc_addr =
                config_value(&content, "rpc_addr").unwrap_or_else(|| "127.0.0.1:0".into());
            let security_mode =
                config_value(&content, "security_mode").unwrap_or_else(|| "unknown".into());
            let peer_count = list_entry_count(&content, "peers");

            let status = if security_mode.contains("test_fixture") {
                "degraded"
            } else if listen_addr == "127.0.0.1:0" || rpc_addr == "127.0.0.1:0" {
                "offline"
            } else {
                "online"
            };

            Ok(NodeControl {
                id: node_name.clone(),
                role: node_role(&node_name).to_string(),
                status: status.to_string(),
                chain_id,
                listen_addr,
                rpc_addr,
                peer_count,
                security_mode,
                command: format!(
                    "cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/{node_name} --rounds 12 --sleep-ms 200"
                ),
            })
        })
        .collect()
}

fn control_files(repo_root: &Path) -> Vec<FileStatus> {
    [
        ("Progress report", AOXC_PROGRESS_REPORT),
        ("Mainnet profile", "configs/mainnet.toml"),
        ("Testnet profile", "configs/testnet.toml"),
        ("AOXHub mainnet profile", "configs/aoxhub-mainnet.toml"),
        ("AOXHub testnet profile", "configs/aoxhub-testnet.toml"),
        (
            "Deterministic testnet launcher",
            "configs/deterministic-testnet/launch-testnet.sh",
        ),
        (
            "Production closure runbook",
            "scripts/validation/network_production_closure.sh",
        ),
    ]
    .into_iter()
    .map(|(label, path)| file_status(repo_root, label, path))
    .collect()
}

fn discover_telemetry_surfaces(repo_root: &Path) -> Vec<TelemetrySurface> {
    let mainnet = fs::read_to_string(repo_root.join("configs/mainnet.toml")).unwrap_or_default();
    let testnet = fs::read_to_string(repo_root.join("configs/testnet.toml")).unwrap_or_default();
    let closure_dir = repo_root.join("artifacts/network-production-closure");

    vec![
        TelemetrySurface {
            title: "Mainnet RPC".to_string(),
            status: file_exists(repo_root, "configs/mainnet.toml"),
            target: config_value(&mainnet, "rpc_addr").unwrap_or_else(|| "n/a".into()),
            detail: format!(
                "Security mode: {}",
                config_value(&mainnet, "security_mode").unwrap_or_else(|| "unknown".into())
            ),
        },
        TelemetrySurface {
            title: "Testnet RPC".to_string(),
            status: file_exists(repo_root, "configs/testnet.toml"),
            target: config_value(&testnet, "rpc_addr").unwrap_or_else(|| "n/a".into()),
            detail: format!(
                "Security mode: {}",
                config_value(&testnet, "security_mode").unwrap_or_else(|| "unknown".into())
            ),
        },
        TelemetrySurface {
            title: "Telemetry snapshot".to_string(),
            status: if closure_dir.join("telemetry-snapshot.json").exists() {
                "ready".to_string()
            } else {
                "blocked".to_string()
            },
            target: "artifacts/network-production-closure/telemetry-snapshot.json".to_string(),
            detail: "Prometheus, alerting, and closure telemetry evidence should be exported here."
                .to_string(),
        },
    ]
}

fn discover_report_assets(repo_root: &Path) -> Vec<ReportAsset> {
    let candidates = [
        (
            "Release evidence bundle",
            "artifacts/release-evidence",
            "Signed release evidence, compatibility matrix, and provenance artifacts.",
        ),
        (
            "Network production closure",
            "artifacts/network-production-closure",
            "Soak, telemetry, recovery, and multi-host closure evidence.",
        ),
        (
            "Progress report",
            AOXC_PROGRESS_REPORT,
            "Current readiness summary consumed by AOXHub desktop.",
        ),
    ];

    candidates
        .into_iter()
        .map(|(title, path, detail)| {
            let target = repo_root.join(path);
            ReportAsset {
                title: title.to_string(),
                status: if target.exists() {
                    "ready".to_string()
                } else {
                    "queued".to_string()
                },
                path: path.to_string(),
                detail: detail.to_string(),
            }
        })
        .collect()
}

fn wallet_surfaces() -> Vec<WalletSurface> {
    vec![
        WalletSurface {
            title: "Operator wallet".to_string(),
            route: "mainnet guarded".to_string(),
            status: "connected".to_string(),
            address_hint: "AOXC1-VAL-OPER-PRIMARY".to_string(),
            command: "aoxc key-bootstrap --profile mainnet --password <value>".to_string(),
            detail: "Validator lifecycle, governance, and emergency operator actions should flow through this lane.".to_string(),
        },
        WalletSurface {
            title: "Treasury wallet".to_string(),
            route: "dual-route mainnet/testnet".to_string(),
            status: "attention".to_string(),
            address_hint: "AOXC1-TREASURY-DESKTOP".to_string(),
            command: "aoxc wallet inspect --profile mainnet".to_string(),
            detail: "Treasury movements require policy visibility and audit export before approval.".to_string(),
        },
        WalletSurface {
            title: "Recovery wallet".to_string(),
            route: "offline recovery lane".to_string(),
            status: "locked".to_string(),
            address_hint: "AOXC1-RECOVERY-ESCROW".to_string(),
            command: "aoxc diagnostics-bundle --redact".to_string(),
            detail: "Disaster recovery drills, key rotation, and cold-path verification anchor.".to_string(),
        },
    ]
}

fn command_presets() -> Vec<CommandPreset> {
    vec![
        CommandPreset {
            title: "Bring up deterministic local cluster".to_string(),
            command: "configs/environments/localnet/launch-localnet.sh".to_string(),
            intent: "Bootstrap the deterministic local cluster and verify cluster orchestration from desktop.".to_string(),
            risk_level: RiskLevel::Medium,
        },
        CommandPreset {
            title: "Production bootstrap + first block".to_string(),
            command: "cargo run -q -p aoxcmd -- production-bootstrap --profile mainnet --password <value> --produce-once-tx bootstrap-mainnet-anchor --format json".to_string(),
            intent: "Create production profile artifacts and produce the first deterministic block in one flow.".to_string(),
            risk_level: RiskLevel::Medium,
        },
        CommandPreset {
            title: "Run mainnet readiness".to_string(),
            command: "cargo run -q -p aoxcmd -- mainnet-readiness --enforce --format json".to_string(),
            intent: "Refresh the mainnet readiness surface before any promotion decision.".to_string(),
            risk_level: RiskLevel::Low,
        },
        CommandPreset {
            title: "Run runtime status".to_string(),
            command: "cargo run -q -p aoxcmd -- runtime-status --format json".to_string(),
            intent: "Refresh runtime health and operational posture for the selected environment.".to_string(),
            risk_level: RiskLevel::Low,
        },
        CommandPreset {
            title: "Generate production audit".to_string(),
            command: "cargo run -q -p aoxcmd -- production-audit --format json".to_string(),
            intent: "Refresh the operator audit surface before release or wallet approval.".to_string(),
            risk_level: RiskLevel::Low,
        },
        CommandPreset {
            title: "Produce one block".to_string(),
            command: "cargo run -q -p aoxcmd -- produce-once --tx desktop-produce-once --format json".to_string(),
            intent: "Run single-block production directly from desktop control workflows.".to_string(),
            risk_level: RiskLevel::Medium,
        },
        CommandPreset {
            title: "Initialize runtime DB".to_string(),
            command: "cargo run -q -p aoxcmd -- db-init --backend sqlite --format json".to_string(),
            intent: "Prepare the runtime data store under AOXC_HOME for real-chain local flows.".to_string(),
            risk_level: RiskLevel::Low,
        },
        CommandPreset {
            title: "Check runtime DB status".to_string(),
            command: "cargo run -q -p aoxcmd -- db-status --backend sqlite --format json".to_string(),
            intent: "Inspect CAS/index surfaces and object count from desktop.".to_string(),
            risk_level: RiskLevel::Low,
        },
    ]
}

fn discover_workspace_surfaces(repo_root: &Path) -> AppResult<Vec<WorkspaceSurface>> {
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml"))
        .map_err(|err| format!("failed to read workspace manifest: {err}"))?;

    workspace_members(&cargo_toml)
        .into_iter()
        .map(|member| {
            let manifest_path = repo_root.join(&member).join("Cargo.toml");
            let manifest = fs::read_to_string(&manifest_path).map_err(|err| {
                format!(
                    "failed to read workspace manifest {}: {err}",
                    manifest_path.display()
                )
            })?;

            let package_name = capture_manifest_package_name(&manifest)
                .unwrap_or_else(|| member.rsplit('/').next().unwrap_or("workspace").to_string());

            let readme_path = repo_root.join(&member).join("README.md");
            let summary = readme_headline(&readme_path).unwrap_or_else(|| {
                format!("{package_name} workspace surface exposed through AOXHub desktop.")
            });

            Ok(WorkspaceSurface {
                name: package_name.clone(),
                path: member.clone(),
                category: workspace_category(&package_name).to_string(),
                status: workspace_status(&package_name).to_string(),
                summary,
            })
        })
        .collect()
}

fn discover_ai_surfaces(repo_root: &Path) -> AppResult<Vec<AiSurface>> {
    let ai_root = repo_root.join("crates/aoxcai/src");
    let lib_rs = fs::read_to_string(ai_root.join("lib.rs"))
        .map_err(|err| format!("failed to read AI library manifest: {err}"))?;

    let mut modules = lib_rs
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|module| module.trim_end_matches(';').trim().to_string())
        .collect::<Vec<_>>();

    modules.sort();

    Ok(modules
        .into_iter()
        .map(|module| AiSurface {
            name: humanize_module_name(&module),
            area: ai_area(&module).to_string(),
            status: ai_status(&module).to_string(),
            summary: ai_summary(&module),
            command: format!("cargo test -p aoxcai {}", ai_command_hint(&module)),
        })
        .collect())
}

fn discover_environments(repo_root: &Path) -> Vec<EnvironmentSurface> {
    vec![
        environment_surface_for(repo_root, EnvironmentKind::Localnet),
        environment_surface_for(repo_root, EnvironmentKind::Testnet),
        environment_surface_for(repo_root, EnvironmentKind::Mainnet),
    ]
}

fn environment_surface_for(repo_root: &Path, kind: EnvironmentKind) -> EnvironmentSurface {
    let (label, profile, config_path, home_path, genesis_path) = match kind {
        EnvironmentKind::Localnet => (
            "Localnet",
            "localnet",
            "configs/deterministic-testnet/nodes/atlas.toml",
            "configs/deterministic-testnet/homes/atlas",
            "configs/deterministic-testnet/genesis.json",
        ),
        EnvironmentKind::Testnet => (
            "Testnet",
            "testnet",
            "configs/testnet.toml",
            ".aoxc-testnet",
            "configs/genesis.json",
        ),
        EnvironmentKind::Mainnet => (
            "Mainnet",
            "mainnet",
            "configs/mainnet.toml",
            ".aoxc-mainnet",
            "configs/genesis.json",
        ),
    };

    let config_content = fs::read_to_string(repo_root.join(config_path)).unwrap_or_default();
    let chain_id = config_value(&config_content, "chain_id").unwrap_or_else(|| "unknown".into());
    let rpc_addr = config_value(&config_content, "rpc_addr").unwrap_or_else(|| "n/a".into());
    let security_mode =
        config_value(&config_content, "security_mode").unwrap_or_else(|| "unknown".into());

    let config_exists = repo_root.join(config_path).exists();
    let home_exists = repo_root.join(home_path).exists();
    let genesis_exists = repo_root.join(genesis_path).exists();

    let readiness_status = if config_exists && genesis_exists {
        if home_exists {
            "in-progress"
        } else {
            "bootstrap"
        }
    } else {
        "blocked"
    };

    let notes = match kind {
        EnvironmentKind::Localnet => {
            "Deterministic fixture environment for desktop bring-up, smoke validation, and cluster workflow rehearsal."
        }
        EnvironmentKind::Testnet => {
            "Shared non-production environment for compatibility, network rehearsal, and pre-mainnet closure."
        }
        EnvironmentKind::Mainnet => {
            "Production promotion environment. All readiness, audit, and evidence gates must be closed before operator approval."
        }
    };

    EnvironmentSurface {
        kind,
        label: label.to_string(),
        profile: profile.to_string(),
        config_path: config_path.to_string(),
        home_path: home_path.to_string(),
        chain_id,
        rpc_addr,
        security_mode,
        readiness_status: readiness_status.to_string(),
        config_exists,
        home_exists,
        genesis_exists,
        notes: notes.to_string(),
    }
}

fn humanize_key(key: &str) -> String {
    key.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn humanize_module_name(key: &str) -> String {
    key.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn remediation_for(key: &str) -> &'static str {
    match key {
        "mainnet-profile" => "aoxc production-bootstrap --profile mainnet --password <value>",
        "structured-logging" => "aoxc config-init --profile mainnet --json-logs",
        "genesis-present" => "aoxc genesis-init",
        "node-state-present" => "aoxc node-bootstrap",
        "operator-key-active" => "aoxc key-bootstrap --profile mainnet --password <value>",
        _ => "Inspect AOXC_PROGRESS_REPORT.md and close the blocker before launch.",
    }
}

fn status_from_percent(percent: u8) -> &'static str {
    if percent >= 100 {
        "ready"
    } else {
        "in-progress"
    }
}

fn desktop_percent(overall_percent: u8, nodes: &[NodeControl], reports: &[ReportAsset]) -> u8 {
    let online_nodes =
        u8::try_from(nodes.iter().filter(|node| node.status == "online").count()).unwrap_or(0);
    let ready_reports = u8::try_from(
        reports
            .iter()
            .filter(|report| report.status == "ready")
            .count(),
    )
    .unwrap_or(0);

    overall_percent
        .saturating_add(online_nodes.saturating_mul(3))
        .saturating_add(ready_reports.saturating_mul(3))
        .min(100)
}

fn file_status(repo_root: &Path, label: &str, relative_path: &str) -> FileStatus {
    let path = repo_root.join(relative_path);

    FileStatus {
        label: label.to_string(),
        path: relative_path.to_string(),
        exists: path.exists(),
    }
}

fn file_exists(repo_root: &Path, relative_path: &str) -> String {
    if repo_root.join(relative_path).exists() {
        "ready".to_string()
    } else {
        "blocked".to_string()
    }
}

fn config_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");

    content
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .map(|value| value.trim().trim_matches('"').to_string())
}

fn workspace_members(content: &str) -> Vec<String> {
    let mut in_members = false;
    let mut members = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("members = [") {
            in_members = true;
            continue;
        }

        if in_members && trimmed == "]" {
            break;
        }

        if in_members {
            let value = trimmed.trim_end_matches(',').trim().trim_matches('"');
            if !value.is_empty() {
                members.push(value.to_string());
            }
        }
    }

    members
}

fn capture_manifest_package_name(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix("name = "))
        .map(|value| value.trim().trim_matches('"').to_string())
}

fn readme_headline(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().and_then(|content| {
        content
            .lines()
            .find(|line| line.trim_start().starts_with('#'))
            .map(|line| line.trim_start_matches('#').trim().to_string())
    })
}

fn workspace_category(package_name: &str) -> &'static str {
    match package_name {
        "aoxcai" => "AI",
        "aoxcmd" => "Operator CLI",
        "aoxcore" | "aoxcunity" | "aoxcvm" => "Core protocol",
        "aoxcrpc" | "aoxcnet" | "aoxconfig" => "Network & RPC",
        "aoxcsdk" | "aoxckit" | "aoxclibs" => "Developer surface",
        "aoxcmob" | "aoxchal" => "Client & access",
        "tests" => "Test surface",
        _ => "Workspace",
    }
}

fn workspace_status(package_name: &str) -> &'static str {
    match package_name {
        "aoxcai" | "aoxcmd" | "aoxcore" | "aoxcnet" | "aoxcrpc" => "ready",
        "tests" => "in-progress",
        _ => "in-progress",
    }
}

fn ai_area(module: &str) -> &'static str {
    match module {
        "backend" | "adapter" => "Execution plane",
        "policy" | "constitution" | "capability" => "Guardrails",
        "audit" | "registry" | "manifest" => "Audit & registry",
        "engine" | "model" | "traits" => "Inference runtime",
        _ => "AI extension",
    }
}

fn ai_status(module: &str) -> &'static str {
    match module {
        "constitution" | "capability" | "audit" | "policy" => "ready",
        _ => "in-progress",
    }
}

fn ai_summary(module: &str) -> String {
    match module {
        "adapter" => {
            "Provider adapters bridge approved AI requests into external or heuristic backends."
                .to_string()
        }
        "audit" => "Every AI invocation should emit auditable records and explicit dispositions."
            .to_string(),
        "backend" => {
            "Backend factory and execution surfaces select local or remote inference providers."
                .to_string()
        }
        "capability" => {
            "Capability grants restrict what AI may do, where it may run, and how it is invoked."
                .to_string()
        }
        "constitution" => {
            "Constitutional rules prevent AI from becoming a kernel authority.".to_string()
        }
        "engine" => {
            "Inference engine coordinates policy, backends, context, and audit flow.".to_string()
        }
        "extension" => {
            "Extension descriptors describe the approved AI feature surface for operators."
                .to_string()
        }
        "manifest" => {
            "Model manifests define model identity, limits, and deployment metadata.".to_string()
        }
        "model" => {
            "Typed request/response and assessment models standardize AI integration.".to_string()
        }
        "policy" => "Policy fusion decides whether an AI request is allowed or denied.".to_string(),
        "registry" => "Registry tracks available AI models and extensions exposed to the platform."
            .to_string(),
        "traits" => "Shared traits define stable AI interfaces for context, policy, and backends."
            .to_string(),
        _ => format!("{module} AI surface is exposed through the desktop control plane."),
    }
}

fn ai_command_hint(module: &str) -> &'static str {
    match module {
        "audit" => "audit",
        "policy" => "policy",
        "backend" => "backend",
        _ => "--lib",
    }
}

fn list_entry_count(content: &str, key: &str) -> usize {
    let marker = format!("{key} = [");

    let Some(start) = content
        .lines()
        .position(|line| line.trim_start().starts_with(&marker))
    else {
        return 0;
    };

    content
        .lines()
        .skip(start + 1)
        .take_while(|line| !line.trim().starts_with(']'))
        .filter(|line| line.contains('"'))
        .count()
}

fn node_role(node_name: &str) -> &'static str {
    match node_name {
        "atlas" => "validator leader",
        "boreal" => "validator follower",
        "cypher" => "observer / telemetry anchor",
        "delta" => "cluster member",
        "ember" => "cluster member",
        _ => "cluster member",
    }
}

fn unix_ms_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn is_safe_token(input: &str) -> bool {
    !input.is_empty()
        && input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
}

#[cfg(test)]
mod tests {
    use super::{
        CommandExecutionRequest, DesktopAction, EnvironmentKind, NodeControl, ReportAsset,
        RiskLevel, capture_area_progress, capture_manifest_package_name, config_value,
        desktop_percent, list_entry_count, parse_area_progress, parse_percent_from_line,
        risk_level_for, workspace_members,
    };

    #[test]
    fn parses_markdown_percent_lines() {
        assert_eq!(
            parse_percent_from_line("- Overall readiness: **60%** (73/121)"),
            Some(60)
        );
        assert_eq!(
            parse_percent_from_line("- **mainnet**: 60% (73/121) — in-progress"),
            Some(60)
        );
        assert_eq!(
            parse_percent_from_line("- **testnet**: 65% (73/111) — in-progress"),
            Some(65)
        );
    }

    #[test]
    fn parses_area_progress_lines() {
        let report = "## Area progress\n- **network**: 100% (1/1 checks, weight 10/10) — ready\n- **identity**: 0% (0/2 checks, weight 0/22) — bootstrap\n";
        let areas = capture_area_progress(report);

        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].name, "Network");
        assert_eq!(areas[0].percent, 100);
        assert_eq!(areas[0].status, "ready");
        assert_eq!(areas[1].name, "Identity");
        assert_eq!(areas[1].percent, 0);
        assert_eq!(areas[1].status, "in-progress");
    }

    #[test]
    fn extracts_config_values_and_peer_counts() {
        let config = r#"
node_name = "atlas"
listen_addr = "127.0.0.1:39001"
peers = [
  "127.0.0.1:39002",
  "127.0.0.1:39003"
]
"#;

        assert_eq!(config_value(config, "node_name").as_deref(), Some("atlas"));
        assert_eq!(list_entry_count(config, "peers"), 2);
    }

    #[test]
    fn desktop_percent_reflects_nodes_and_reports() {
        let nodes = vec![
            NodeControl {
                id: "atlas".into(),
                role: "validator leader".into(),
                status: "online".into(),
                chain_id: "a".into(),
                listen_addr: "l".into(),
                rpc_addr: "r".into(),
                peer_count: 2,
                security_mode: "audit".into(),
                command: "cmd".into(),
            },
            NodeControl {
                id: "cypher".into(),
                role: "observer".into(),
                status: "degraded".into(),
                chain_id: "a".into(),
                listen_addr: "l".into(),
                rpc_addr: "r".into(),
                peer_count: 2,
                security_mode: "audit".into(),
                command: "cmd".into(),
            },
        ];

        let reports = vec![
            ReportAsset {
                title: "a".into(),
                status: "ready".into(),
                path: "p".into(),
                detail: "d".into(),
            },
            ReportAsset {
                title: "b".into(),
                status: "queued".into(),
                path: "p".into(),
                detail: "d".into(),
            },
        ];

        assert_eq!(desktop_percent(60, &nodes, &reports), 66);
    }

    #[test]
    fn parse_area_progress_returns_none_for_invalid_line() {
        assert!(parse_area_progress("- not-an-area").is_none());
    }

    #[test]
    fn workspace_members_are_parsed_from_root_manifest() {
        let manifest = r#"
[workspace]
members = [
  "crates/aoxcai",
  "crates/aoxcore",
  "tests",
]
"#;

        assert_eq!(
            workspace_members(manifest),
            vec!["crates/aoxcai", "crates/aoxcore", "tests"]
        );
    }

    #[test]
    fn package_name_is_extracted_from_manifest() {
        let manifest = r#"
[package]
name = "aoxcai"
version = "0.1.0"
"#;

        assert_eq!(
            capture_manifest_package_name(manifest).as_deref(),
            Some("aoxcai")
        );
    }

    #[test]
    fn action_risk_levels_are_stable() {
        assert_eq!(
            risk_level_for(DesktopAction::MainnetReadiness),
            RiskLevel::Low
        );
        assert_eq!(risk_level_for(DesktopAction::NodeRun), RiskLevel::Medium);
    }

    #[test]
    fn request_type_round_trip_compiles() {
        let req = CommandExecutionRequest {
            action: DesktopAction::RuntimeStatus,
            environment: EnvironmentKind::Localnet,
            options: None,
            confirmation_phrase: None,
        };

        assert!(matches!(req.environment, EnvironmentKind::Localnet));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_control_center_snapshot,
            run_desktop_action
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
