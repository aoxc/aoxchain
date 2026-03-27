use crate::keyforge::cli::{RegistryCommand, RegistrySubcommand};
use crate::keyforge::util::{read_json_file, write_json_file};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEntry {
    pub actor_id: String,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RegistryState {
    pub entries: Vec<RegistryEntry>,
}

pub fn handle(command: RegistryCommand) -> Result<(), String> {
    match command.command {
        RegistrySubcommand::UpsertEntry {
            registry,
            actor_id,
            status,
            reason,
        } => upsert_entry(&registry, &actor_id, &status, reason.as_deref()),
        RegistrySubcommand::List { registry } => list_entries(&registry),
    }
}

pub fn load_registry(path: &str) -> Result<RegistryState, String> {
    if std::path::Path::new(path).exists() {
        read_json_file(path)
    } else {
        Ok(RegistryState::default())
    }
}

fn normalized_status(raw: &str) -> Result<String, String> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "active" | "revoked" | "suspended" => Ok(value),
        _ => Err("REGISTRY_STATUS_INVALID".to_string()),
    }
}

fn upsert_entry(
    registry_path: &str,
    actor_id: &str,
    status: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    if actor_id.trim().is_empty() {
        return Err("REGISTRY_ACTOR_ID_EMPTY".to_string());
    }

    let status = normalized_status(status)?;
    let mut state = load_registry(registry_path)?;

    let reason = reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    if let Some(existing) = state
        .entries
        .iter_mut()
        .find(|entry| entry.actor_id == actor_id)
    {
        existing.status = status;
        existing.reason = reason;
    } else {
        state.entries.push(RegistryEntry {
            actor_id: actor_id.to_string(),
            status,
            reason,
        });
    }

    state
        .entries
        .sort_by(|left, right| left.actor_id.cmp(&right.actor_id));
    write_json_file(registry_path, &state)?;

    let output = serde_json::json!({
        "registry": registry_path,
        "actor_id": actor_id,
        "result": "updated",
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?
    );

    Ok(())
}

fn list_entries(registry_path: &str) -> Result<(), String> {
    let state = load_registry(registry_path)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&state)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_registry, upsert_entry};

    fn tmp_file(name: &str) -> String {
        let pid = std::process::id();
        std::env::temp_dir()
            .join(format!("aoxckit-registry-{pid}-{name}.json"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn upsert_entry_creates_file_and_writes_entry() {
        let path = tmp_file("create");
        let _ = std::fs::remove_file(&path);

        upsert_entry(&path, "actor-1", "active", Some("bootstrap")).expect("upsert must succeed");

        let state = load_registry(&path).expect("registry load must succeed");
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].actor_id, "actor-1");
        assert_eq!(state.entries[0].status, "active");
        assert_eq!(state.entries[0].reason.as_deref(), Some("bootstrap"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn upsert_entry_updates_existing_actor() {
        let path = tmp_file("update");
        let _ = std::fs::remove_file(&path);

        upsert_entry(&path, "actor-1", "active", None).expect("first upsert must succeed");
        upsert_entry(&path, "actor-1", "revoked", Some("manual"))
            .expect("second upsert must succeed");

        let state = load_registry(&path).expect("registry load must succeed");
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].status, "revoked");
        assert_eq!(state.entries[0].reason.as_deref(), Some("manual"));

        let _ = std::fs::remove_file(path);
    }
}
