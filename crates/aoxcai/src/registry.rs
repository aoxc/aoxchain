use crate::{error::AiError, manifest::ModelManifest, model::AiTask};
use std::{collections::HashMap, fs, path::Path};

/// Stores validated manifests and deterministic task bindings.
#[derive(Debug, Default)]
pub struct ModelRegistry {
    manifests: HashMap<String, ModelManifest>,
    bindings: HashMap<AiTask, String>,
}

impl ModelRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_dir(mut self, dir: impl AsRef<Path>) -> Result<Self, AiError> {
        let dir_ref = dir.as_ref();

        let entries = fs::read_dir(dir_ref).map_err(|err| AiError::Io {
            path: dir_ref.display().to_string(),
            reason: err.to_string(),
        })?;

        for entry in entries {
            let entry = entry.map_err(|err| AiError::Io {
                path: dir_ref.display().to_string(),
                reason: err.to_string(),
            })?;
            let path = entry.path();

            let is_yaml = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
                .unwrap_or(false);

            if is_yaml {
                let manifest = ModelManifest::from_yaml_file(&path)?;
                if manifest.is_enabled() {
                    self.register(manifest)?;
                }
            }
        }

        Ok(self)
    }

    pub fn register(&mut self, manifest: ModelManifest) -> Result<(), AiError> {
        manifest.validate()?;

        let id = manifest.id().to_owned();

        if self.manifests.contains_key(&id) {
            return Err(AiError::ManifestValidation(format!(
                "duplicate manifest id '{}'",
                id
            )));
        }

        for task in &manifest.spec.bindings.default_for_tasks {
            if let Some(existing) = self.bindings.get(task) {
                return Err(AiError::ManifestValidation(format!(
                    "duplicate default binding for task '{task:?}' between '{}' and '{}'",
                    existing, id
                )));
            }
            self.bindings.insert(*task, id.clone());
        }

        self.manifests.insert(id, manifest);
        Ok(())
    }

    pub fn resolve_for_task(&self, task: AiTask) -> Result<&ModelManifest, AiError> {
        let model_id = self
            .bindings
            .get(&task)
            .ok_or_else(|| AiError::BindingNotFound(format!("{task:?}")))?;

        self.manifests
            .get(model_id)
            .ok_or_else(|| AiError::ModelNotFound(model_id.clone()))
    }

    pub fn get(&self, id: &str) -> Result<&ModelManifest, AiError> {
        self.manifests
            .get(id)
            .ok_or_else(|| AiError::ModelNotFound(id.to_owned()))
    }

    pub fn bind_checked(
        &mut self,
        task: AiTask,
        model_id: impl Into<String>,
    ) -> Result<(), AiError> {
        let model_id = model_id.into();

        if !self.manifests.contains_key(&model_id) {
            return Err(AiError::ModelNotFound(model_id));
        }

        self.bindings.insert(task, model_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::AiTask, test_support::base_manifest};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn register_and_get_return_manifest_by_id() {
        let mut registry = ModelRegistry::new();
        let manifest = base_manifest();
        let id = manifest.metadata.id.clone();

        registry
            .register(manifest)
            .expect("registration must succeed");
        let resolved = registry.get(&id).expect("manifest must be found");
        assert_eq!(resolved.metadata.id, id);
    }

    #[test]
    fn duplicate_default_bindings_are_rejected() {
        let mut registry = ModelRegistry::new();
        let manifest_a = base_manifest();

        let mut manifest_b = base_manifest();
        manifest_b.metadata.id = "model-b".to_owned();

        registry
            .register(manifest_a)
            .expect("first manifest registration must succeed");

        let err = registry
            .register(manifest_b)
            .expect_err("duplicate binding must fail");

        match err {
            AiError::ManifestValidation(message) => {
                assert!(message.contains("duplicate default binding"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn bind_checked_overrides_existing_binding_when_model_exists() {
        let mut registry = ModelRegistry::new();
        let manifest_a = base_manifest();
        let mut manifest_b = base_manifest();
        manifest_b.metadata.id = "test-model-b".to_owned();
        manifest_b.spec.bindings.default_for_tasks.clear();

        registry
            .register(manifest_a)
            .expect("first manifest must register");
        registry
            .register(manifest_b)
            .expect("second manifest must register");

        registry
            .bind_checked(AiTask::ValidatorAdmission, "test-model-b")
            .expect("binding override must succeed");

        let resolved = registry
            .resolve_for_task(AiTask::ValidatorAdmission)
            .expect("binding must resolve");
        assert_eq!(resolved.metadata.id, "test-model-b");
    }

    #[test]
    fn load_dir_registers_enabled_yaml_manifests_only() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aoxcai-registry-test-{unique}"));
        fs::create_dir_all(&dir).expect("temp dir must be created");

        let enabled_manifest = base_manifest();
        let enabled_yaml =
            serde_yaml::to_string(&enabled_manifest).expect("manifest must serialize");
        fs::write(dir.join("enabled.yaml"), enabled_yaml).expect("enabled manifest must write");

        let mut disabled_manifest = base_manifest();
        disabled_manifest.metadata.id = "disabled-model".to_owned();
        disabled_manifest.spec.enabled = false;
        disabled_manifest.spec.bindings.default_for_tasks.clear();
        let disabled_yaml =
            serde_yaml::to_string(&disabled_manifest).expect("manifest must serialize");
        fs::write(dir.join("disabled.yaml"), disabled_yaml).expect("disabled manifest must write");

        fs::write(dir.join("notes.txt"), "ignore me").expect("note file must write");

        let registry = ModelRegistry::new()
            .load_dir(&dir)
            .expect("directory load must succeed");
        let resolved = registry
            .resolve_for_task(AiTask::ValidatorAdmission)
            .expect("enabled manifest binding must resolve");
        assert_eq!(resolved.metadata.id, enabled_manifest.metadata.id);
        assert_eq!(
            registry
                .get("disabled-model")
                .expect_err("disabled manifest must not register"),
            AiError::ModelNotFound("disabled-model".to_owned())
        );

        fs::remove_dir_all(&dir).expect("temp dir must be removed");
    }

    #[test]
    fn bind_checked_rejects_unknown_model() {
        let mut registry = ModelRegistry::new();

        let err = registry
            .bind_checked(AiTask::ValidatorAdmission, "missing")
            .expect_err("unknown model must fail");

        assert_eq!(err, AiError::ModelNotFound("missing".to_owned()));
    }
}
