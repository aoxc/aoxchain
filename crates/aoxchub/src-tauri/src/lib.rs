use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

type AppResult<T> = Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadinessTrack {
    name: String,
    percent: u8,
    summary: String,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchBlocker {
    title: String,
    detail: String,
    command: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileStatus {
    label: String,
    path: String,
    exists: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchSnapshot {
    stage: String,
    verdict: String,
    overall_percent: u8,
    profile: String,
    summary: String,
    tracks: Vec<ReadinessTrack>,
    blockers: Vec<LaunchBlocker>,
    files: Vec<FileStatus>,
}

#[tauri::command]
fn load_launch_snapshot() -> AppResult<LaunchSnapshot> {
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
    let summary = if blockers.is_empty() {
        "No blockers listed in AOXC_PROGRESS_REPORT.md".to_string()
    } else {
        format!(
            "{} active blocker(s) still need closure before a full launch claim.",
            blockers.len()
        )
    };

    Ok(LaunchSnapshot {
        stage,
        verdict,
        overall_percent,
        profile,
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
                summary: "Testnet should close non-mainnet blockers and sustain AOXHub parity."
                    .into(),
                status: status_from_percent(testnet_percent).into(),
            },
            ReadinessTrack {
                name: "Overall program".into(),
                percent: overall_percent,
                summary: format!("Current release stage: {stage}. Active profile: {profile}."),
                status: status_from_percent(overall_percent).into(),
            },
        ],
        blockers,
        files: vec![
            file_status(&repo_root, "Progress report", "AOXC_PROGRESS_REPORT.md"),
            file_status(&repo_root, "Mainnet profile", "configs/mainnet.toml"),
            file_status(&repo_root, "Testnet profile", "configs/testnet.toml"),
            file_status(
                &repo_root,
                "AOXHub mainnet profile",
                "configs/aoxhub-mainnet.toml",
            ),
            file_status(
                &repo_root,
                "AOXHub testnet profile",
                "configs/aoxhub-testnet.toml",
            ),
        ],
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
    let mut in_section = false;
    let mut blockers = Vec::new();

    for line in report.lines() {
        if line.trim() == "## Remaining blockers" {
            in_section = true;
            continue;
        }

        if in_section && line.starts_with("## ") {
            break;
        }

        if !in_section {
            continue;
        }

        let Some(rest) = line.strip_prefix("- ") else {
            continue;
        };
        let mut parts = rest.splitn(2, ':');
        let key = parts.next().unwrap_or_default().trim();
        let detail = parts.next().unwrap_or_default().trim();
        if key.is_empty() || detail.is_empty() {
            continue;
        }
        blockers.push(LaunchBlocker {
            title: humanize_key(key),
            detail: detail.to_string(),
            command: remediation_for(key).to_string(),
        });
    }

    blockers
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

fn file_status(repo_root: &Path, label: &str, relative_path: &str) -> FileStatus {
    let path = repo_root.join(relative_path);
    FileStatus {
        label: label.to_string(),
        path: relative_path.to_string(),
        exists: path.exists(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_launch_snapshot])
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
