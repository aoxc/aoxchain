// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{RegistryCommand, RegistrySubcommand};
use crate::keyforge::util::{read_json_file, write_json_file};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Canonical AOXC registry entry.
///
/// Security posture:
/// - This structure contains only operator-facing registry metadata.
/// - It is safe for JSON persistence and stdout emission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEntry {
    pub actor_id: String,
    pub status: String,
    pub reason: Option<String>,
}

/// Canonical AOXC registry state.
///
/// Determinism policy:
/// - entries are persisted in actor_id-sorted order,
/// - duplicate actor identifiers are collapsed during normalization.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RegistryState {
    pub entries: Vec<RegistryEntry>,
}

/// Canonical upsert response emitted by the registry CLI surface.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RegistryUpsertOutput {
    registry: String,
    actor_id: String,
    result: &'static str,
    status: String,
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

/// Loads a registry from disk if it exists, otherwise returns an empty registry.
///
/// Validation policy:
/// - the path must not be blank,
/// - decoded registry content is normalized before use.
pub fn load_registry(path: &str) -> Result<RegistryState, String> {
    let normalized_path = normalize_required_text(path, "registry")?;

    if Path::new(&normalized_path).exists() {
        let state: RegistryState = read_json_file(&normalized_path)?;
        normalize_registry_state(state)
    } else {
        Ok(RegistryState::default())
    }
}

/// Normalizes and validates a registry status value.
fn normalized_status(raw: &str) -> Result<String, String> {
    let value = raw.trim().to_ascii_lowercase();

    match value.as_str() {
        "active" | "revoked" | "suspended" => Ok(value),
        _ => Err("REGISTRY_STATUS_INVALID".to_string()),
    }
}

/// Upserts a registry entry and persists the canonical sorted registry state.
///
/// Update policy:
/// - blank actor identifiers are rejected,
/// - status is normalized into the canonical lowercase vocabulary,
/// - blank reasons are removed,
/// - existing actor identifiers are updated in place,
/// - registry ordering is stable and deterministic.
fn upsert_entry(
    registry_path: &str,
    actor_id: &str,
    status: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let normalized_registry_path = normalize_required_text(registry_path, "registry")?;
    let normalized_actor_id = normalize_required_text(actor_id, "actor_id")?;
    let normalized_status = normalized_status(status)?;
    let normalized_reason = normalize_optional_reason(reason);

    let state = load_registry(&normalized_registry_path)?;
    let updated = upsert_registry_state(
        state,
        &normalized_actor_id,
        &normalized_status,
        normalized_reason,
    )?;

    write_json_file(&normalized_registry_path, &updated)?;

    let output = RegistryUpsertOutput {
        registry: normalized_registry_path,
        actor_id: normalized_actor_id,
        result: "updated",
        status: normalized_status,
    };

    println!("{}", serialize_pretty_json(&output)?);

    Ok(())
}

/// Lists the canonical registry state.
///
/// Output policy:
/// - the registry is normalized before emission,
/// - entries are emitted in deterministic actor_id-sorted order.
fn list_entries(registry_path: &str) -> Result<(), String> {
    let state = load_registry(registry_path)?;
    println!("{}", serialize_pretty_json(&state)?);
    Ok(())
}

/// Applies a single upsert operation to an in-memory registry state.
///
/// Determinism guarantees:
/// - resulting entries are always actor_id-sorted,
/// - duplicate actors are not introduced,
/// - existing actor records are replaced canonically.
fn upsert_registry_state(
    mut state: RegistryState,
    actor_id: &str,
    status: &str,
    reason: Option<String>,
) -> Result<RegistryState, String> {
    let normalized_actor_id = normalize_required_text(actor_id, "actor_id")?;
    let normalized_status = normalized_status(status)?;

    state = normalize_registry_state(state)?;

    if let Some(existing) = state
        .entries
        .iter_mut()
        .find(|entry| entry.actor_id == normalized_actor_id)
    {
        existing.status = normalized_status;
        existing.reason = reason;
    } else {
        state.entries.push(RegistryEntry {
            actor_id: normalized_actor_id,
            status: normalized_status,
            reason,
        });
    }

    normalize_registry_state(state)
}

/// Normalizes an optional operator-supplied reason.
///
/// Policy:
/// - blank or whitespace-only values become `None`,
/// - surrounding whitespace is trimmed,
/// - non-empty text is preserved.
fn normalize_optional_reason(reason: Option<&str>) -> Option<String> {
    reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// Validates and normalizes a registry state.
///
/// Validation policy:
/// - every actor_id must be non-blank,
/// - every status must be canonical,
/// - duplicate actor identifiers are collapsed with last-write-wins semantics,
/// - output order is actor_id ascending.
fn normalize_registry_state(state: RegistryState) -> Result<RegistryState, String> {
    let mut normalized_entries: Vec<RegistryEntry> = Vec::with_capacity(state.entries.len());

    for entry in state.entries {
        let normalized_actor_id = normalize_required_text(&entry.actor_id, "actor_id")?;
        let normalized_status = normalized_status(&entry.status)?;
        let normalized_reason = normalize_optional_reason(entry.reason.as_deref());

        if let Some(existing) = normalized_entries
            .iter_mut()
            .find(|candidate| candidate.actor_id == normalized_actor_id)
        {
            existing.status = normalized_status;
            existing.reason = normalized_reason;
        } else {
            normalized_entries.push(RegistryEntry {
                actor_id: normalized_actor_id,
                status: normalized_status,
                reason: normalized_reason,
            });
        }
    }

    normalized_entries.sort_by(|left, right| left.actor_id.cmp(&right.actor_id));

    Ok(RegistryState {
        entries: normalized_entries,
    })
}

/// Serializes an operator-facing value into canonical pretty JSON.
fn serialize_pretty_json<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))
}

/// Enforces non-blank operator-facing text input.
///
/// Policy:
/// - trims leading and trailing whitespace,
/// - rejects whitespace-only values,
/// - returns normalized content.
fn normalize_required_text(value: &str, field: &str) -> Result<String, String> {
    let normalized = value.trim();

    if normalized.is_empty() {
        return Err(format!("INVALID_ARGUMENT: {} must not be blank", field));
    }

    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_file(name: &str) -> String {
        let pid = std::process::id();
        std::env::temp_dir()
            .join(format!("aoxckit-registry-{pid}-{name}.json"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn normalized_status_accepts_canonical_values() {
        assert_eq!(normalized_status("active"), Ok("active".to_string()));
        assert_eq!(normalized_status("REVOKED"), Ok("revoked".to_string()));
        assert_eq!(
            normalized_status(" suspended "),
            Ok("suspended".to_string())
        );
    }

    #[test]
    fn normalized_status_rejects_unknown_values() {
        assert_eq!(
            normalized_status("pending"),
            Err("REGISTRY_STATUS_INVALID".to_string())
        );
    }

    #[test]
    fn normalize_optional_reason_removes_blank_values() {
        assert_eq!(normalize_optional_reason(None), None);
        assert_eq!(normalize_optional_reason(Some("   ")), None);
        assert_eq!(
            normalize_optional_reason(Some(" manual action ")),
            Some("manual action".to_string())
        );
    }

    #[test]
    fn load_registry_returns_default_when_file_is_missing() {
        let path = tmp_file("missing");
        let _ = std::fs::remove_file(&path);

        let state = load_registry(&path).expect("missing registry must resolve to default");
        assert_eq!(state, RegistryState::default());
    }

    #[test]
    fn normalize_registry_state_sorts_entries_and_normalizes_status() {
        let state = RegistryState {
            entries: vec![
                RegistryEntry {
                    actor_id: "actor-b".to_string(),
                    status: "REVOKED".to_string(),
                    reason: Some(" reason ".to_string()),
                },
                RegistryEntry {
                    actor_id: " actor-a ".to_string(),
                    status: "ACTIVE".to_string(),
                    reason: Some("   ".to_string()),
                },
            ],
        };

        let normalized = normalize_registry_state(state).expect("normalization must succeed");

        assert_eq!(normalized.entries.len(), 2);
        assert_eq!(normalized.entries[0].actor_id, "actor-a");
        assert_eq!(normalized.entries[0].status, "active");
        assert_eq!(normalized.entries[0].reason, None);
        assert_eq!(normalized.entries[1].actor_id, "actor-b");
        assert_eq!(normalized.entries[1].status, "revoked");
        assert_eq!(normalized.entries[1].reason.as_deref(), Some("reason"));
    }

    #[test]
    fn normalize_registry_state_collapses_duplicate_actor_ids() {
        let state = RegistryState {
            entries: vec![
                RegistryEntry {
                    actor_id: "actor-1".to_string(),
                    status: "active".to_string(),
                    reason: None,
                },
                RegistryEntry {
                    actor_id: "actor-1".to_string(),
                    status: "revoked".to_string(),
                    reason: Some("manual".to_string()),
                },
            ],
        };

        let normalized = normalize_registry_state(state).expect("normalization must succeed");

        assert_eq!(normalized.entries.len(), 1);
        assert_eq!(normalized.entries[0].actor_id, "actor-1");
        assert_eq!(normalized.entries[0].status, "revoked");
        assert_eq!(normalized.entries[0].reason.as_deref(), Some("manual"));
    }

    #[test]
    fn upsert_registry_state_inserts_new_entry() {
        let state = RegistryState::default();

        let updated =
            upsert_registry_state(state, "actor-1", "active", Some("bootstrap".to_string()))
                .expect("upsert must succeed");

        assert_eq!(updated.entries.len(), 1);
        assert_eq!(updated.entries[0].actor_id, "actor-1");
        assert_eq!(updated.entries[0].status, "active");
        assert_eq!(updated.entries[0].reason.as_deref(), Some("bootstrap"));
    }

    #[test]
    fn upsert_registry_state_updates_existing_entry() {
        let state = RegistryState {
            entries: vec![RegistryEntry {
                actor_id: "actor-1".to_string(),
                status: "active".to_string(),
                reason: None,
            }],
        };

        let updated =
            upsert_registry_state(state, "actor-1", "revoked", Some("manual".to_string()))
                .expect("upsert must succeed");

        assert_eq!(updated.entries.len(), 1);
        assert_eq!(updated.entries[0].status, "revoked");
        assert_eq!(updated.entries[0].reason.as_deref(), Some("manual"));
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

    #[test]
    fn upsert_entry_rejects_blank_actor_id() {
        let path = tmp_file("blank-actor");
        let _ = std::fs::remove_file(&path);

        let result = upsert_entry(&path, "   ", "active", None);

        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: actor_id must not be blank".to_string())
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn list_entries_accepts_existing_registry() {
        let path = tmp_file("list");
        let _ = std::fs::remove_file(&path);

        upsert_entry(&path, "actor-1", "active", Some("bootstrap")).expect("upsert must succeed");

        let result = list_entries(&path);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn handle_dispatches_list_successfully() {
        let path = tmp_file("handle-list");
        let _ = std::fs::remove_file(&path);

        upsert_entry(&path, "actor-1", "active", Some("bootstrap")).expect("upsert must succeed");

        let command = RegistryCommand {
            command: RegistrySubcommand::List {
                registry: path.clone(),
            },
        };

        let result = handle(command);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(path);
    }
}
