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
    fn bind_checked_rejects_unknown_model() {
        let mut registry = ModelRegistry::new();

        let err = registry
            .bind_checked(AiTask::ValidatorAdmission, "missing")
            .expect_err("unknown model must fail");

        assert_eq!(err, AiError::ModelNotFound("missing".to_owned()));
    }
}
