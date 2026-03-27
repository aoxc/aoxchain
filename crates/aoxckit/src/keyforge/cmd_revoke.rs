use crate::keyforge::cli::{RevokeCommand, RevokeSubcommand};
use crate::keyforge::cmd_registry::load_registry;
use crate::keyforge::util::write_json_file;

pub fn handle(command: RevokeCommand) -> Result<(), String> {
    match command.command {
        RevokeSubcommand::Actor {
            registry,
            actor_id,
            reason,
        } => revoke_actor(&registry, &actor_id, &reason),
    }
}

fn revoke_actor(registry_path: &str, actor_id: &str, reason: &str) -> Result<(), String> {
    if actor_id.trim().is_empty() {
        return Err("REVOKE_ACTOR_ID_EMPTY".to_string());
    }

    if reason.trim().is_empty() {
        return Err("REVOKE_REASON_EMPTY".to_string());
    }

    let mut state = load_registry(registry_path)?;
    let entry = state
        .entries
        .iter_mut()
        .find(|item| item.actor_id == actor_id)
        .ok_or_else(|| "REVOKE_ACTOR_NOT_FOUND".to_string())?;

    entry.status = "revoked".to_string();
    entry.reason = Some(reason.trim().to_string());
    write_json_file(registry_path, &state)?;

    let output = serde_json::json!({
        "registry": registry_path,
        "actor_id": actor_id,
        "status": "revoked",
        "reason": reason.trim(),
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::revoke_actor;
    use crate::keyforge::cmd_registry::{RegistryEntry, RegistryState, load_registry};
    use crate::keyforge::util::write_json_file;

    fn tmp_file(name: &str) -> String {
        let pid = std::process::id();
        std::env::temp_dir()
            .join(format!("aoxckit-revoke-{pid}-{name}.json"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn revoke_actor_updates_status_to_revoked() {
        let path = tmp_file("ok");
        let _ = std::fs::remove_file(&path);
        let state = RegistryState {
            entries: vec![RegistryEntry {
                actor_id: "actor-1".to_string(),
                status: "active".to_string(),
                reason: None,
            }],
        };
        write_json_file(&path, &state).expect("seed registry must succeed");

        revoke_actor(&path, "actor-1", "key compromise").expect("revoke must succeed");

        let updated = load_registry(&path).expect("load must succeed");
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
}
