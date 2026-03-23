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
struct WorkspaceSurface {
    name: String,
    path: String,
    category: String,
    status: String,
    summary: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiSurface {
    name: String,
    area: String,
    status: String,
    summary: String,
    command: String,
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
    workspaces: Vec<WorkspaceSurface>,
    ai_surfaces: Vec<AiSurface>,
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
    let workspaces = discover_workspace_surfaces(&repo_root)?;
    let ai_surfaces = discover_ai_surfaces(&repo_root)?;

    let summary = format!(
        "{} blocker(s), {} workspace surface(s), {} AI surface(s), {} node control surface(s), and {} report asset(s) are currently exposed through AOXHub desktop.",
        blockers.len(),
        workspaces.len(),
        ai_surfaces.len(),
        nodes.len(),
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
        workspaces,
        ai_surfaces,
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
            detail: "Prometheus, alerts, and closure telemetry evidence should be exported here."
                .into(),
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
    let node_bonus = u8::try_from(nodes.iter().filter(|node| node.status == "online").count())
        .unwrap_or(0)
        .saturating_mul(5);
    let report_bonus = u8::try_from(
        reports
            .iter()
            .filter(|report| report.status == "ready")
            .count(),
    )
    .unwrap_or(0)
    .saturating_mul(3);
    overall_percent
        .saturating_add(node_bonus)
        .saturating_add(report_bonus)
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
    let Some(start) = content
        .lines()
        .position(|line| line.trim_start().starts_with(&format!("{key} = [")))
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
        _ => "cluster member",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        capture_area_progress, capture_manifest_package_name, config_value, desktop_percent,
        list_entry_count, parse_area_progress, workspace_members, NodeControl, ReportAsset,
    };

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
        assert_eq!(desktop_percent(60, &nodes, &reports), 68);
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
