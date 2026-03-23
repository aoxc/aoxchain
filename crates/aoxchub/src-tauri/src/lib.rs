use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

type AppResult<T> = Result<T, String>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadinessTrack {
    name: String,
    percent: u8,
    summary: String,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchBlocker {
    title: String,
    detail: String,
    command: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileStatus {
    label: String,
    path: String,
    exists: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AreaProgress {
    name: String,
    percent: u8,
    detail: String,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletSurface {
    title: String,
    route: String,
    status: String,
    address_hint: String,
    command: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TelemetrySurface {
    title: String,
    status: String,
    target: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportAsset {
    title: String,
    status: String,
    path: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandPreset {
    title: String,
    command: String,
    intent: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseSurface {
    title: String,
    status: String,
    path: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogSurface {
    title: String,
    status: String,
    path: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExplorerSurface {
    title: String,
    status: String,
    target: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalSurface {
    title: String,
    command: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
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
    databases: Vec<DatabaseSurface>,
    logs: Vec<LogSurface>,
    explorer: Vec<ExplorerSurface>,
    terminals: Vec<TerminalSurface>,
}

#[tauri::command]
fn load_control_center_snapshot() -> AppResult<ControlCenterSnapshot> {
    let repo_root = repo_root()?;
    let report_path = repo_root.join("AOXC_PROGRESS_REPORT.md");
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
    let databases = database_surfaces(&repo_root);
    let logs = log_surfaces(&repo_root);
    let explorer = explorer_surfaces(&repo_root);
    let terminals = terminal_surfaces();

    let summary = format!(
        "{} blocker(s), {} node control surface(s), {} wallet lane(s), and {} report asset(s) are currently exposed through AOXHub desktop.",
        blockers.len(),
        nodes.len(),
        wallets.len(),
        reports.len()
    );

    Ok(ControlCenterSnapshot {
        stage: stage.clone(),
        verdict,
        overall_percent,
        profile: profile.clone(),
        summary,
        tracks: vec![
            ReadinessTrack {
                name: "Mainnet readiness".into(),
                percent: mainnet_percent,
                summary: "Production controls must all pass before mainnet promotion.".into(),
                status: status_from_percent(mainnet_percent).into(),
            },
            ReadinessTrack {
                name: "Testnet readiness".into(),
                percent: testnet_percent,
                summary: "Testnet should close non-mainnet blockers and sustain AOXHub/core parity."
                    .into(),
                status: status_from_percent(testnet_percent).into(),
            },
            ReadinessTrack {
                name: "Desktop control center".into(),
                percent: desktop_percent(overall_percent, &nodes, &reports),
                summary: format!(
                    "Current release stage: {stage}. Active profile: {profile}. Desktop panel is expected to unify node, wallet, telemetry, and report operations."
                ),
                status: status_from_percent(desktop_percent(overall_percent, &nodes, &reports))
                    .into(),
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
        databases,
        logs,
        explorer,
        terminals,
    })
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
    line.split("**")
        .next_back()
        .and_then(|tail| tail.split('%').next())
        .and_then(|value| value.trim().parse::<u8>().ok())
        .or_else(|| {
            line.split(':')
                .nth(1)
                .and_then(|tail| tail.split('%').next())
                .and_then(|value| value.trim().parse::<u8>().ok())
        })
        .ok_or_else(|| format!("failed to parse percentage from line: {line}"))
}

fn capture_blockers(report: &str) -> Vec<LaunchBlocker> {
    collect_section_items(report, "## Remaining blockers")
        .into_iter()
        .filter_map(|line| {
            let rest = line.strip_prefix("- ")?;
            let mut parts = rest.splitn(2, ':');
            let key = parts.next()?.trim();
            let detail = parts.next()?.trim();
            Some(LaunchBlocker {
                title: humanize_key(key),
                detail: detail.to_string(),
                command: remediation_for(key).to_string(),
            })
        })
        .collect()
}

fn capture_area_progress(report: &str) -> Vec<AreaProgress> {
    collect_section_items(report, "## Area progress")
        .into_iter()
        .filter_map(|line| parse_area_progress(line))
        .collect()
}

fn collect_section_items(report: &str, header: &str) -> Vec<&str> {
    let mut in_section = false;
    let mut out = Vec::new();

    for line in report.lines() {
        if line.trim() == header {
            in_section = true;
            continue;
        }

        if in_section && line.starts_with("## ") {
            break;
        }

        if in_section && line.trim_start().starts_with('-') {
            out.push(line.trim());
        }
    }

    out
}

fn parse_area_progress(line: &str) -> Option<AreaProgress> {
    let rest = line.strip_prefix("- **")?;
    let (name, tail) = rest.split_once("**:")?;
    let percent = tail
        .split('%')
        .next()?
        .trim()
        .trim_start_matches(|ch: char| !ch.is_ascii_digit())
        .parse()
        .ok()?;
    let detail = tail
        .split('—')
        .next()
        .unwrap_or(tail)
        .trim()
        .trim_matches('-')
        .trim()
        .to_string();
    let status = if line.contains("— ready") {
        "ready"
    } else {
        "in-progress"
    };

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
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect::<Vec<_>>();
    paths.sort();

    paths.into_iter()
        .take(3)
        .map(|path| {
            let content = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            let node_name = config_value(&content, "node_name").unwrap_or_else(|| "node".into());
            let chain_id = config_value(&content, "chain_id").unwrap_or_else(|| "unknown".into());
            let listen_addr =
                config_value(&content, "listen_addr").unwrap_or_else(|| "127.0.0.1:0".into());
            let rpc_addr =
                config_value(&content, "rpc_addr").unwrap_or_else(|| "127.0.0.1:0".into());
            let security_mode = config_value(&content, "security_mode")
                .unwrap_or_else(|| "unknown".into());
            let peer_count = list_entry_count(&content, "peers");
            let status = if security_mode.contains("test_fixture") {
                "degraded"
            } else {
                "online"
            };

            Ok(NodeControl {
                id: node_name.clone(),
                role: node_role(&node_name).to_string(),
                status: status.to_string(),
                chain_id,
                listen_addr,
                rpc_addr: rpc_addr.clone(),
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
        ("Progress report", "AOXC_PROGRESS_REPORT.md"),
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
            title: "Mainnet RPC".into(),
            status: file_exists(repo_root, "configs/mainnet.toml"),
            target: config_value(&mainnet, "rpc_addr").unwrap_or_else(|| "n/a".into()),
            detail: format!(
                "Security mode: {}",
                config_value(&mainnet, "security_mode").unwrap_or_else(|| "unknown".into())
            ),
        },
        TelemetrySurface {
            title: "Testnet RPC".into(),
            status: file_exists(repo_root, "configs/testnet.toml"),
            target: config_value(&testnet, "rpc_addr").unwrap_or_else(|| "n/a".into()),
            detail: format!(
                "Security mode: {}",
                config_value(&testnet, "security_mode").unwrap_or_else(|| "unknown".into())
            ),
        },
        TelemetrySurface {
            title: "Telemetry snapshot".into(),
            status: if closure_dir.join("telemetry-snapshot.json").exists() {
                "ready".into()
            } else {
                "blocked".into()
            },
            target: "artifacts/network-production-closure/telemetry-snapshot.json".into(),
            detail: "Prometheus, alerts, and closure telemetry evidence should be exported here.".into(),
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
            "AOXC_PROGRESS_REPORT.md",
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
                    "ready".into()
                } else {
                    "queued".into()
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
            title: "Operator wallet".into(),
            route: "mainnet guarded".into(),
            status: "connected".into(),
            address_hint: "AOXC1-VAL-OPER-PRIMARY".into(),
            command: "aoxc key-bootstrap --profile mainnet --password <value>".into(),
            detail: "Validator lifecycle, governance, and emergency operator actions should flow through this lane.".into(),
        },
        WalletSurface {
            title: "Treasury wallet".into(),
            route: "dual-route mainnet/testnet".into(),
            status: "attention".into(),
            address_hint: "AOXC1-TREASURY-DESKTOP".into(),
            command: "aoxc wallet inspect --profile mainnet".into(),
            detail: "Treasury moves require policy visibility and audit export before approval.".into(),
        },
        WalletSurface {
            title: "Recovery wallet".into(),
            route: "offline recovery lane".into(),
            status: "locked".into(),
            address_hint: "AOXC1-RECOVERY-ESCROW".into(),
            command: "aoxc diagnostics-bundle --redact".into(),
            detail: "Disaster recovery drills, key rotation, and cold-path verification anchor.".into(),
        },
    ]
}

fn command_presets() -> Vec<CommandPreset> {
    vec![
        CommandPreset {
            title: "Bring up deterministic 3-node cluster".into(),
            command: "configs/deterministic-testnet/launch-testnet.sh".into(),
            intent: "Bootstrap three local nodes and verify cluster orchestration from desktop.".into(),
        },
        CommandPreset {
            title: "Generate production audit".into(),
            command: "cargo run -q -p aoxcmd -- production-audit --format json".into(),
            intent: "Refresh the operator audit surface before release or wallet approval.".into(),
        },
        CommandPreset {
            title: "Produce closure bundle".into(),
            command: "scripts/validation/network_production_closure.sh --scenario soak".into(),
            intent: "Collect telemetry, runtime, and rollout evidence for the admin cockpit reporting tab.".into(),
        },
    ]
}


fn database_surfaces(repo_root: &Path) -> Vec<DatabaseSurface> {
    vec![
        DatabaseSurface {
            title: "Runtime state store".into(),
            status: if repo_root.join("artifacts").exists() { "ready".into() } else { "queued".into() },
            path: "artifacts/".into(),
            detail: "Runtime state, release artifacts, and closure bundles should be queryable from the desktop database view.".into(),
        },
        DatabaseSurface {
            title: "Deterministic testnet fixtures".into(),
            status: if repo_root.join("configs/deterministic-testnet/accounts.json").exists() { "ready".into() } else { "queued".into() },
            path: "configs/deterministic-testnet/accounts.json".into(),
            detail: "Fixture accounts, node identity, and deterministic operator data.".into(),
        },
    ]
}

fn log_surfaces(repo_root: &Path) -> Vec<LogSurface> {
    vec![
        LogSurface {
            title: "Production closure logs".into(),
            status: if repo_root.join("artifacts/network-production-closure").exists() { "ready".into() } else { "queued".into() },
            path: "artifacts/network-production-closure".into(),
            detail: "Soak, telemetry, and recovery evidence should be explorable as log bundles.".into(),
        },
        LogSurface {
            title: "Release evidence logs".into(),
            status: if repo_root.join("artifacts/release-evidence").exists() { "ready".into() } else { "queued".into() },
            path: "artifacts/release-evidence".into(),
            detail: "SBOM, provenance, and compatibility evidence for explorer/audit timelines.".into(),
        },
    ]
}

fn explorer_surfaces(repo_root: &Path) -> Vec<ExplorerSurface> {
    vec![
        ExplorerSurface {
            title: "Progress explorer".into(),
            status: file_exists(repo_root, "AOXC_PROGRESS_REPORT.md"),
            target: "AOXC_PROGRESS_REPORT.md".into(),
            detail: "Readiness explorer for blockers, progress, parity, and remediation order.".into(),
        },
        ExplorerSurface {
            title: "Node fixture explorer".into(),
            status: file_exists(repo_root, "configs/deterministic-testnet/nodes/atlas.toml"),
            target: "configs/deterministic-testnet/nodes".into(),
            detail: "Node topology explorer for local cluster management and RPC endpoint mapping.".into(),
        },
        ExplorerSurface {
            title: "Artifact explorer".into(),
            status: file_exists(repo_root, "artifacts/release-evidence"),
            target: "artifacts/".into(),
            detail: "Evidence explorer for release, audit, telemetry, and network-closure artifacts.".into(),
        },
    ]
}

fn terminal_surfaces() -> Vec<TerminalSurface> {
    vec![
        TerminalSurface {
            title: "Cluster terminal".into(),
            command: "configs/deterministic-testnet/launch-testnet.sh".into(),
            detail: "Bootstraps the deterministic local network from the desktop terminal rail.".into(),
        },
        TerminalSurface {
            title: "Audit terminal".into(),
            command: "cargo run -q -p aoxcmd -- production-audit --format json".into(),
            detail: "Generates the operator audit output consumed by reporting panels.".into(),
        },
        TerminalSurface {
            title: "Closure terminal".into(),
            command: "scripts/validation/network_production_closure.sh --scenario soak".into(),
            detail: "Runs the closure bundle workflow and fills telemetry/report explorers.".into(),
        },
    ]
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
    let node_bonus = u8::try_from(nodes.iter().filter(|node| node.status == "online").count())
        .unwrap_or(0)
        .saturating_mul(5);
    let report_bonus = u8::try_from(reports.iter().filter(|report| report.status == "ready").count())
        .unwrap_or(0)
        .saturating_mul(3);
    overall_percent.saturating_add(node_bonus).saturating_add(report_bonus).min(100)
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
        "ready".into()
    } else {
        "blocked".into()
    }
}

fn config_value(content: &str, key: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix(&format!("{key} = ")))
        .map(|value| value.trim().trim_matches('"').to_string())
}

fn list_entry_count(content: &str, key: &str) -> usize {
    let Some(start) = content.lines().position(|line| line.trim_start().starts_with(&format!("{key} = ["))) else {
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
        _ => "cluster member",
    }
}

#[cfg(test)]
mod tests {
    use super::{capture_area_progress, config_value, desktop_percent, list_entry_count, parse_area_progress, ReportAsset, NodeControl};

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
            ReportAsset { title: "a".into(), status: "ready".into(), path: "p".into(), detail: "d".into() },
            ReportAsset { title: "b".into(), status: "queued".into(), path: "p".into(), detail: "d".into() },
        ];
        assert_eq!(desktop_percent(60, &nodes, &reports), 68);
    }

    #[test]
    fn parse_area_progress_returns_none_for_invalid_line() {
        assert!(parse_area_progress("- not-an-area").is_none());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_control_center_snapshot])
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
