use super::*;

const CORE7_TOPOLOGY_ROLES: [&str; 7] = [
    "core_val",   // quorum
    "core_prop",  // forge
    "core_guard", // seal
    "data_arch",  // archive
    "sec_sent",   // sentinel
    "net_relay",  // relay
    "serv_rpc",   // pocket
];

const CORE7_CANONICAL_KERNEL_ROLES: [&str; 7] = [
    "quorum", "forge", "seal", "archive", "sentinel", "relay", "pocket",
];

pub fn cmd_role_list(args: &[String]) -> Result<(), AppError> {
    let profile = target_profile(args)?;
    let path = role_topology_path(&profile);
    let topology = read_to_string(&path)?;
    let states = parse_role_enabled_states(&topology);

    let mut details = BTreeMap::new();
    details.insert("profile".to_string(), profile);
    details.insert("role_file".to_string(), path.display().to_string());
    details.insert(
        "kernel_core7".to_string(),
        CORE7_CANONICAL_KERNEL_ROLES.join(","),
    );
    details.insert("active_roles".to_string(), active_roles_csv(&states));
    details.insert("all_roles".to_string(), all_roles_csv(&states));

    emit_serialized(
        &text_envelope("role-list", "ok", details),
        output_format(args),
    )
}

pub fn cmd_role_model_status(args: &[String]) -> Result<(), AppError> {
    let profile = target_profile(args)?;
    let path = role_topology_path(&profile);
    let topology = read_to_string(&path)?;
    let states = parse_role_enabled_states(&topology);

    let mut details = BTreeMap::new();
    details.insert("profile".to_string(), profile);
    details.insert("role_file".to_string(), path.display().to_string());
    details.insert(
        "kernel_core7".to_string(),
        CORE7_CANONICAL_KERNEL_ROLES.join(","),
    );

    let missing = CORE7_TOPOLOGY_ROLES
        .iter()
        .filter(|role| !states.get(**role).copied().unwrap_or(false))
        .map(|role| role.to_string())
        .collect::<Vec<_>>();

    let non_core_active = states
        .iter()
        .filter(|(role, enabled)| **enabled && !CORE7_TOPOLOGY_ROLES.contains(&role.as_str()))
        .map(|(role, _)| role.clone())
        .collect::<Vec<_>>();

    details.insert(
        "core7_ready".to_string(),
        (missing.is_empty() && non_core_active.is_empty()).to_string(),
    );
    details.insert(
        "missing_core7_roles".to_string(),
        if missing.is_empty() {
            "none".to_string()
        } else {
            missing.join(",")
        },
    );
    details.insert(
        "non_core_active_roles".to_string(),
        if non_core_active.is_empty() {
            "none".to_string()
        } else {
            non_core_active.join(",")
        },
    );

    emit_serialized(
        &text_envelope("role-model-status", "ok", details),
        output_format(args),
    )
}

pub fn cmd_role_activate_core7(args: &[String]) -> Result<(), AppError> {
    let profile = target_profile(args)?;
    let dry_run = has_flag(args, "--dry-run");
    let path = role_topology_path(&profile);
    let original = read_to_string(&path)?;
    let updated = rewrite_role_enabled_states(&original, &CORE7_TOPOLOGY_ROLES);

    if !dry_run {
        apply_role_topology_transactionally(&path, &updated)?;
    }

    let states = if dry_run {
        parse_role_enabled_states(&updated)
    } else {
        let persisted = read_to_string(&path)?;
        parse_role_enabled_states(&persisted)
    };

    let status = core7_status(&states);
    if !status.missing.is_empty() || !status.non_core_active.is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Core7 activation verification failed: missing={} non_core_active={}",
                if status.missing.is_empty() {
                    "none".to_string()
                } else {
                    status.missing.join(",")
                },
                if status.non_core_active.is_empty() {
                    "none".to_string()
                } else {
                    status.non_core_active.join(",")
                }
            ),
        ));
    }

    let mut details = BTreeMap::new();
    details.insert("profile".to_string(), profile);
    details.insert("role_file".to_string(), path.display().to_string());
    details.insert(
        "kernel_core7".to_string(),
        CORE7_CANONICAL_KERNEL_ROLES.join(","),
    );
    details.insert(
        "mode".to_string(),
        if dry_run { "dry-run" } else { "applied" }.to_string(),
    );
    details.insert("active_roles".to_string(), active_roles_csv(&states));
    details.insert(
        "verification".to_string(),
        "core7-active-and-exclusive".to_string(),
    );

    emit_serialized(
        &text_envelope("role-activate-core7", "ok", details),
        output_format(args),
    )
}

fn target_profile(args: &[String]) -> Result<String, AppError> {
    if let Some(raw) = arg_value(args, "--profile") {
        return normalize_text(&raw, true).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --profile must not be blank",
            )
        });
    }

    match load() {
        Ok(settings) => Ok(settings.profile),
        Err(_) => Ok("validation".to_string()),
    }
}

fn role_topology_path(profile: &str) -> PathBuf {
    PathBuf::from("configs")
        .join("environments")
        .join(profile)
        .join("topology")
        .join("role-topology.toml")
}

fn read_to_string(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read role topology file {}", path.display()),
            error,
        )
    })
}

fn parse_role_enabled_states(content: &str) -> BTreeMap<String, bool> {
    let mut states = BTreeMap::new();
    let mut current_role: Option<String> = None;

    for raw in content.lines() {
        let line = raw.trim();
        if let Some(role) = parse_role_header(line) {
            current_role = Some(role.to_string());
            states.entry(role.to_string()).or_insert(false);
            continue;
        }

        if let Some(role) = current_role.as_ref() {
            if line.starts_with("enabled") {
                states.insert(role.clone(), line.ends_with("true"));
            }
        }
    }

    states
}

fn parse_role_header(line: &str) -> Option<&str> {
    let prefix = "[roles.";
    if !line.starts_with(prefix) || !line.ends_with(']') {
        return None;
    }

    let inner = &line[prefix.len()..line.len() - 1];
    if inner.is_empty() { None } else { Some(inner) }
}

fn rewrite_role_enabled_states(content: &str, core7_roles: &[&str]) -> String {
    let mut lines = content.lines().map(ToString::to_string).collect::<Vec<_>>();
    let mut role_ranges = Vec::<(usize, usize, String)>::new();

    let mut idx = 0;
    while idx < lines.len() {
        if let Some(role) = parse_role_header(lines[idx].trim()).map(str::to_string) {
            let start = idx;
            idx += 1;
            while idx < lines.len() && !lines[idx].trim_start().starts_with("[") {
                idx += 1;
            }
            role_ranges.push((start, idx, role));
            continue;
        }
        idx += 1;
    }

    for (start, end, role) in role_ranges {
        let should_enable = core7_roles.contains(&role.as_str());
        let mut replaced = false;

        for line in lines.iter_mut().take(end).skip(start + 1) {
            let trimmed = line.trim_start();
            if trimmed.starts_with("enabled") {
                let indent_len = line.len() - trimmed.len();
                let indent = " ".repeat(indent_len);
                *line = format!("{indent}enabled = {should_enable}");
                replaced = true;
                break;
            }
        }

        if !replaced {
            lines.insert(start + 1, format!("enabled = {should_enable}"));
        }
    }

    lines.join("\n") + "\n"
}

fn apply_role_topology_transactionally(path: &Path, updated: &str) -> Result<(), AppError> {
    let backup_path = path.with_extension("toml.bak.core7");
    let temp_path = path.with_extension("toml.tmp.core7");

    let original = read_to_string(path)?;
    fs::write(&backup_path, original.as_bytes()).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create role topology backup {}",
                backup_path.display()
            ),
            error,
        )
    })?;

    if let Err(error) = fs::write(&temp_path, updated.as_bytes()) {
        let _ = fs::remove_file(&temp_path);
        return Err(AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to write role topology temp file {}",
                temp_path.display()
            ),
            error,
        ));
    }

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        let _ = fs::write(path, original.as_bytes());
        return Err(AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to apply role topology update to {}", path.display()),
            error,
        ));
    }

    Ok(())
}

struct Core7Status {
    missing: Vec<String>,
    non_core_active: Vec<String>,
}

fn core7_status(states: &BTreeMap<String, bool>) -> Core7Status {
    let missing = CORE7_TOPOLOGY_ROLES
        .iter()
        .filter(|role| !states.get(**role).copied().unwrap_or(false))
        .map(|role| role.to_string())
        .collect::<Vec<_>>();

    let non_core_active = states
        .iter()
        .filter(|(role, enabled)| **enabled && !CORE7_TOPOLOGY_ROLES.contains(&role.as_str()))
        .map(|(role, _)| role.clone())
        .collect::<Vec<_>>();

    Core7Status {
        missing,
        non_core_active,
    }
}

fn active_roles_csv(states: &BTreeMap<String, bool>) -> String {
    let active = states
        .iter()
        .filter(|(_, enabled)| **enabled)
        .map(|(role, _)| role.clone())
        .collect::<Vec<_>>();

    if active.is_empty() {
        "none".to_string()
    } else {
        active.join(",")
    }
}

fn all_roles_csv(states: &BTreeMap<String, bool>) -> String {
    let values = states
        .iter()
        .map(|(role, enabled)| format!("{role}:{}", if *enabled { "on" } else { "off" }))
        .collect::<Vec<_>>();

    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::{core7_status, parse_role_enabled_states, rewrite_role_enabled_states};

    #[test]
    fn rewrite_role_enabled_states_applies_core7_and_disables_others() {
        let original = r#"
[roles.core_val]
enabled = true

[roles.core_prop]
enabled = false

[roles.core_guard]
enabled = false

[roles.net_relay]
enabled = true

[roles.net_gate]
enabled = true

[roles.data_arch]
enabled = false

[roles.data_da]
enabled = true

[roles.serv_rpc]
enabled = true

[roles.serv_idx]
enabled = true

[roles.sec_sent]
enabled = false

[roles.x_orcl]
enabled = true

[roles.x_bridge]
enabled = true
"#;

        let rewritten = rewrite_role_enabled_states(
            original,
            &[
                "core_val",
                "core_prop",
                "core_guard",
                "data_arch",
                "sec_sent",
                "net_relay",
                "serv_rpc",
            ],
        );

        let states = parse_role_enabled_states(&rewritten);
        assert_eq!(states.get("core_val"), Some(&true));
        assert_eq!(states.get("core_prop"), Some(&true));
        assert_eq!(states.get("core_guard"), Some(&true));
        assert_eq!(states.get("data_arch"), Some(&true));
        assert_eq!(states.get("sec_sent"), Some(&true));
        assert_eq!(states.get("net_relay"), Some(&true));
        assert_eq!(states.get("serv_rpc"), Some(&true));

        assert_eq!(states.get("net_gate"), Some(&false));
        assert_eq!(states.get("data_da"), Some(&false));
        assert_eq!(states.get("serv_idx"), Some(&false));
        assert_eq!(states.get("x_orcl"), Some(&false));
        assert_eq!(states.get("x_bridge"), Some(&false));
    }

    #[test]
    fn core7_status_reports_missing_and_non_core_roles() {
        let mut states = std::collections::BTreeMap::new();
        states.insert("core_val".to_string(), true);
        states.insert("core_prop".to_string(), false);
        states.insert("core_guard".to_string(), true);
        states.insert("data_arch".to_string(), true);
        states.insert("sec_sent".to_string(), true);
        states.insert("net_relay".to_string(), true);
        states.insert("serv_rpc".to_string(), true);
        states.insert("x_bridge".to_string(), true);

        let status = core7_status(&states);
        assert_eq!(status.missing, vec!["core_prop".to_string()]);
        assert_eq!(status.non_core_active, vec!["x_bridge".to_string()]);
    }
}
