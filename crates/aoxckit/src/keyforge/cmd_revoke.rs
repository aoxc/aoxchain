// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{RevokeCommand, RevokeSubcommand};
use crate::keyforge::cmd_registry::{load_registry, RegistryState};
use crate::keyforge::util::write_json_file;
use serde::Serialize;

/// Canonical revoke response emitted by the AOXC revoke CLI surface.
///
/// Security posture:
/// - This payload is public-only.
/// - It contains no secret-bearing material.
/// - It is stable and machine-readable for shells, CI, and audit tooling.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RevokeActorOutput {
    registry: String,
    actor_id: String,
    status: &'static str,
    reason: String,
}

pub fn handle(command: RevokeCommand) -> Result<(), String> {
    match command.command {
        RevokeSubcommand::Actor {
            registry,
            actor_id,
            reason,
        } => revoke_actor(&registry, &actor_id, &reason),
    }
}

/// Revokes an actor inside the supplied registry and persists the updated state.
///
/// Validation policy:
/// - registry path must not be blank,
/// - actor_id must not be blank,
/// - reason must not be blank,
/// - the target actor must already exist in the registry.
fn revoke_actor(registry_path: &str, actor_id: &str, reason: &str) -> Result<(), String> {
    let normalized_registry_path = normalize_required_text(registry_path, "registry")?;
    let normalized_actor_id = normalize_required_text(actor_id, "actor_id")?;
    let normalized_reason = normalize_required_text(reason, "reason")?;

    let state = load_registry(&normalized_registry_path)?;
    let updated_state = revoke_actor_in_state(state, &normalized_actor_id, &normalized_reason)?;

    write_json_file(&normalized_registry_path, &updated_state)?;

    let output = RevokeActorOutput {
        registry: normalized_registry_path,
        actor_id: normalized_actor_id,
        status: "revoked",
        reason: normalized_reason,
    };

    println!("{}", serialize_pretty_json(&output)?);

    Ok(())
}

/// Applies revocation to an in-memory registry state.
///
/// Behavioral contract:
/// - the target actor must exist,
/// - the actor status is set to `revoked`,
/// - the supplied reason is stored as canonical trimmed text,
/// - resulting entries remain sorted by actor_id for deterministic persistence.
fn revoke_actor_in_state(
    mut state: RegistryState,
    actor_id: &str,
    reason: &str,
) -> Result<RegistryState, String> {
    let normalized_actor_id = normalize_required_text(actor_id, "actor_id")?;
    let normalized_reason = normalize_required_text(reason, "reason")?;

    let entry = state
        .entries
        .iter_mut()
        .find(|item| item.actor_id == normalized_actor_id)
        .ok_or_else(|| "REVOKE_ACTOR_NOT_FOUND".to_string())?;

    entry.status = "revoked".to_string();
    entry.reason = Some(normalized_reason);

    state
        .entries
        .sort_by(|left, right| left.actor_id.cmp(&right.actor_id));

    Ok(state)
}

/// Serializes an operator-facing payload into canonical pretty JSON.
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
        return Err(format!(
            "INVALID_ARGUMENT: {} must not be blank",
            field
        ));
    }

    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyforge::cmd_registry::{load_registry, RegistryEntry};
    use crate::keyforge::util::write_json_file;

    fn tmp_file(name: &str) -> String {
        let pid = std::process::id();
        std::env::temp_dir()
            .join(format!("aoxckit-revoke-{pid}-{name}.json"))
            .to_string_lossy()
            .into_owned()
    }

    fn seeded_registry_state() -> RegistryState {
        RegistryState {
            entries: vec![
                RegistryEntry {
                    actor_id: "actor-2".to_string(),
                    status: "active".to_string(),
                    reason: None,
                },
                RegistryEntry {
                    actor_id: "actor-1".to_string(),
                    status: "active".to_string(),
                    reason: None,
                },
            ],
        }
    }

    #[test]
    fn revoke_actor_in_state_updates_status_to_revoked() {
        let state = RegistryState {
            entries: vec![RegistryEntry {
                actor_id: "actor-1".to_string(),
                status: "active".to_string(),
                reason: None,
            }],
        };

        let updated =
            revoke_actor_in_state(state, "actor-1", "key compromise").expect("revoke must succeed");

        assert_eq!(updated.entries.len(), 1);
        assert_eq!(updated.entries[0].actor_id, "actor-1");
        assert_eq!(updated.entries[0].status, "revoked");
        assert_eq!(updated.entries[0].reason.as_deref(), Some("key compromise"));
    }

    #[test]
    fn revoke_actor_in_state_trims_reason_and_preserves_deterministic_sort_order() {
        let state = seeded_registry_state();

        let updated = revoke_actor_in_state(state, "actor-2", "  manual action  ")
            .expect("revoke must succeed");

        assert_eq!(updated.entries[0].actor_id, "actor-1");
        assert_eq!(updated.entries[1].actor_id, "actor-2");
        assert_eq!(updated.entries[1].status, "revoked");
        assert_eq!(updated.entries[1].reason.as_deref(), Some("manual action"));
    }

    #[test]
    fn revoke_actor_in_state_fails_when_actor_missing() {
        let state = RegistryState::default();

        let error = revoke_actor_in_state(state, "missing", "reason")
            .expect_err("revoke must fail for missing actor");

        assert_eq!(error, "REVOKE_ACTOR_NOT_FOUND");
    }

    #[test]
    fn revoke_actor_in_state_rejects_blank_actor_id() {
        let state = RegistryState::default();

        let error = revoke_actor_in_state(state, "   ", "reason")
            .expect_err("blank actor_id must be rejected");

        assert_eq!(
            error,
            "INVALID_ARGUMENT: actor_id must not be blank".to_string()
        );
    }

    #[test]
    fn revoke_actor_in_state_rejects_blank_reason() {
        let state = seeded_registry_state();

        let error = revoke_actor_in_state(state, "actor-1", "   ")
            .expect_err("blank reason must be rejected");

        assert_eq!(
            error,
            "INVALID_ARGUMENT: reason must not be blank".to_string()
        );
    }

    #[test]
    fn revoke_actor_updates_status_to_revoked() {
        let path = tmp_file("ok");
        let _ = std::fs::remove_file(&path);

        write_json_file(&path, &seeded_registry_state()).expect("seed registry must succeed");

        revoke_actor(&path, "actor-1", "key compromise").expect("revoke must succeed");

        let updated = load_registry(&path).expect("load must succeed");

        assert_eq!(updated.entries.len(), 2);
        assert_eq!(updated.entries[0].actor_id, "actor-1");
        assert_eq!(updated.entries[0].status, "revoked");
        assert_eq!(updated.entries[0].reason.as_deref(), Some("key compromise"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revoke_actor_fails_when_actor_missing() {
        let path = tmp_file("missing");
        let _ = std::fs::remove_file(&path);

        write_json_file(&path, &RegistryState::default()).expect("seed must succeed");

        let error = revoke_actor(&path, "missing", "reason")
            .expect_err("revoke must fail for missing actor");

        assert_eq!(error, "REVOKE_ACTOR_NOT_FOUND");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revoke_actor_rejects_blank_reason() {
        let path = tmp_file("blank-reason");
        let _ = std::fs::remove_file(&path);

        write_json_file(&path, &seeded_registry_state()).expect("seed must succeed");

        let error = revoke_actor(&path, "actor-1", "   ")
            .expect_err("blank reason must be rejected");

        assert_eq!(
            error,
            "INVALID_ARGUMENT: reason must not be blank".to_string()
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn handle_dispatches_actor_revoke_successfully() {
        let path = tmp_file("handle");
        let _ = std::fs::remove_file(&path);

        write_json_file(&path, &seeded_registry_state()).expect("seed must succeed");

        let command = RevokeCommand {
            command: RevokeSubcommand::Actor {
                registry: path.clone(),
                actor_id: "actor-1".to_string(),
                reason: "key compromise".to_string(),
            },
        };

        let result = handle(command);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(path);
    }
}
