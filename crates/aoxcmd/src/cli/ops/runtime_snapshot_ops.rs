use super::*;
use std::process::Command;

#[derive(serde::Serialize)]
struct RuntimeSnapshotResponse {
    action: String,
    runtime_root: String,
    snapshot_root: String,
    status: &'static str,
    output: String,
}

pub fn cmd_runtime_snapshot(args: &[String]) -> Result<(), AppError> {
    cmd_runtime_snapshot_action(args, Some("snapshot"))
}

pub fn cmd_runtime_snapshot_list(args: &[String]) -> Result<(), AppError> {
    cmd_runtime_snapshot_action(args, Some("list"))
}

pub fn cmd_runtime_snapshot_prune(args: &[String]) -> Result<(), AppError> {
    cmd_runtime_snapshot_action(args, Some("prune"))
}

pub fn cmd_runtime_restore_latest(args: &[String]) -> Result<(), AppError> {
    cmd_runtime_snapshot_action(args, Some("restore-latest"))
}

pub fn cmd_query_runtime(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return cmd_runtime_snapshot_action(args, Some("list"));
    };

    match subcommand.as_str() {
        "snapshot" => cmd_runtime_snapshot_action(tail, None),
        "status" => cmd_runtime_snapshot_action(tail, Some("list")),
        _ => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Invalid query runtime command: supported subcommands are snapshot|status",
        )),
    }
}

fn cmd_runtime_snapshot_action(
    args: &[String],
    default_action: Option<&str>,
) -> Result<(), AppError> {
    let action = arg_value(args, "--action")
        .or_else(|| default_action.map(str::to_string))
        .unwrap_or_else(|| "snapshot".to_string());

    let keep = arg_value(args, "--keep");

    let home = resolve_home()?;
    let runtime_root = arg_value(args, "--runtime-root")
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("runtime"));
    let snapshot_root = arg_value(args, "--snapshot-dir")
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| runtime_root.join("snapshots"));

    let output = run_snapshot_guard(&action, keep.as_deref(), &runtime_root, &snapshot_root)?;

    let response = RuntimeSnapshotResponse {
        action,
        runtime_root: runtime_root.display().to_string(),
        snapshot_root: snapshot_root.display().to_string(),
        status: "ok",
        output,
    };

    emit_serialized(&response, output_format(args))
}

fn run_snapshot_guard(
    action: &str,
    keep: Option<&str>,
    runtime_root: &Path,
    snapshot_root: &Path,
) -> Result<String, AppError> {
    if !matches!(action, "snapshot" | "list" | "prune" | "restore-latest") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Invalid --action value: {action}. Allowed: snapshot|list|prune|restore-latest"
            ),
        ));
    }

    if let Some(raw) = keep {
        if !raw.chars().all(|ch| ch.is_ascii_digit()) || raw.parse::<u32>().ok().unwrap_or(0) == 0 {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Invalid --keep value: {raw}. Expected positive integer."),
            ));
        }
    }

    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/runtime_snapshot_guard.sh")
        .canonicalize()
        .map_err(|error| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to resolve runtime snapshot script path: {error}"),
            )
        })?;

    let mut command = Command::new(script_path);
    command.arg(action);
    command.env("AOXC_RUNTIME_ROOT", runtime_root.as_os_str());
    command.env("AOXC_RUNTIME_SNAPSHOTS_DIR", snapshot_root.as_os_str());

    if let Some(keep_value) = keep {
        command.env("AOXC_SNAPSHOT_KEEP", keep_value);
    }

    let output = command.output().map_err(|error| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to execute runtime snapshot script: {error}"),
        )
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        return Ok(stdout);
    }

    let message = if stderr.is_empty() {
        format!("runtime snapshot operation failed: action={action}")
    } else {
        format!("runtime snapshot operation failed: action={action}: {stderr}")
    };

    Err(AppError::new(ErrorCode::FilesystemIoFailed, message))
}

#[cfg(test)]
mod tests {
    use super::run_snapshot_guard;
    use std::path::Path;

    #[test]
    fn rejects_invalid_action() {
        let err = run_snapshot_guard(
            "invalid",
            None,
            Path::new("/tmp/aoxc-runtime"),
            Path::new("/tmp/aoxc-runtime/snapshots"),
        )
        .expect_err("invalid action should fail");

        assert!(format!("{err}").contains("Invalid --action value"));
    }

    #[test]
    fn rejects_invalid_keep_value() {
        let err = run_snapshot_guard(
            "prune",
            Some("0"),
            Path::new("/tmp/aoxc-runtime"),
            Path::new("/tmp/aoxc-runtime/snapshots"),
        )
        .expect_err("invalid keep should fail");

        assert!(format!("{err}").contains("Invalid --keep value"));
    }
}
