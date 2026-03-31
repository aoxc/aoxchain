use crate::{
    binaries, commands,
    domain::{
        BinaryCandidate, BinarySourceKind, CommandProgram, CommandView, EnvironmentBinding,
        HubStateView,
    },
    environments::Environment,
    errors::HubError,
    runner::Runner,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct HubService {
    pub environment: Arc<RwLock<Environment>>,
    pub binaries: Arc<RwLock<Vec<BinaryCandidate>>>,
    pub selected_binary_id: Arc<RwLock<Option<String>>>,
    pub runner: Runner,
}

impl HubService {
    pub fn new() -> Self {
        let bins = binaries::discover();
        let selected = bins.first().map(|b| b.id.clone());
        Self {
            environment: Arc::new(RwLock::new(Environment::Mainnet)),
            binaries: Arc::new(RwLock::new(bins)),
            selected_binary_id: Arc::new(RwLock::new(selected)),
            runner: Runner::new(),
        }
    }

    pub async fn state(&self) -> HubStateView {
        let environment = *self.environment.read().await;
        let bins = self.binaries.read().await.clone();
        let selected = self.selected_binary_id.read().await.clone();
        let commands = commands::CATALOG
            .iter()
            .map(|spec| {
                let preview = self.command_preview(environment, spec.program.clone(), spec.args);
                let allowed = self.is_command_allowed(environment, spec.id);
                let policy_note = policy_note(environment, spec.id, spec.risk.clone(), allowed);
                CommandView {
                    spec: spec.clone(),
                    preview,
                    allowed,
                    policy_note,
                }
            })
            .collect();

        HubStateView {
            environment,
            banner: environment.banner_text(),
            binding: EnvironmentBinding {
                slug: environment.slug(),
                root_config: environment.root_config_path(),
                aoxc_home: environment.aoxc_home(),
                make_scope: environment.make_scope(),
            },
            selected_binary_id: selected,
            binaries: bins,
            commands,
        }
    }

    pub async fn set_environment(&self, env: Environment) {
        *self.environment.write().await = env;
    }

    pub async fn set_binary(&self, id: String) -> Result<(), HubError> {
        let env = *self.environment.read().await;
        let bins = self.binaries.read().await;
        let candidate = bins
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| HubError::Security(String::from("selected binary does not exist")))?;
        if !is_binary_allowed(env, &candidate.kind) {
            return Err(HubError::Security(String::from(
                "selected binary source is forbidden in active environment",
            )));
        }
        *self.selected_binary_id.write().await = Some(id);
        Ok(())
    }

    pub async fn add_custom_binary(&self, path: String) -> Result<(), HubError> {
        let env = *self.environment.read().await;
        if env != Environment::Testnet {
            return Err(HubError::Security(String::from(
                "custom binary path is allowed only on testnet",
            )));
        }
        let candidate = BinaryCandidate {
            id: format!("custom-{}", chrono::Utc::now().timestamp()),
            kind: BinarySourceKind::CustomPath,
            path,
            version: None,
            trust: crate::domain::TrustLevel::Unverified,
            checksum_verified: None,
        };
        self.binaries.write().await.push(candidate);
        Ok(())
    }

    pub fn is_command_allowed(&self, env: Environment, command_id: &str) -> bool {
        let Some(spec) = commands::find(command_id) else {
            return false;
        };

        if env == Environment::Mainnet && command_id == "testnet-start" {
            return false;
        }
        if env == Environment::Testnet && command_id == "mainnet-start" {
            return false;
        }

        if env == Environment::Mainnet
            && matches!(spec.risk, crate::domain::RiskClass::High)
            && !matches!(command_id, "mainnet-start" | "aoxc-node-start" | "aoxc-node-stop")
        {
            return false;
        }

        true
    }

    fn command_preview(
        &self,
        env: Environment,
        program: CommandProgram,
        args: &[&'static str],
    ) -> String {
        let prefix = environment_prefix(env);
        match program {
            CommandProgram::Aoxc => format!("{prefix} aoxc {}", args.join(" ")),
            CommandProgram::Make => format!("{prefix} make {}", args.join(" ")),
        }
    }

    pub async fn execute(&self, command_id: String) -> Result<String, HubError> {
        let env = *self.environment.read().await;
        if !self.is_command_allowed(env, &command_id) {
            return Err(HubError::Security(String::from(
                "command is not available in active environment",
            )));
        }
        let spec = commands::find(&command_id)
            .ok_or_else(|| HubError::UnknownCommand(command_id.clone()))?;
        let (program, args) = match spec.program {
            CommandProgram::Make => (
                String::from("make"),
                spec.args.iter().map(|s| s.to_string()).collect(),
            ),
            CommandProgram::Aoxc => {
                let selected_id = self
                    .selected_binary_id
                    .read()
                    .await
                    .clone()
                    .ok_or_else(|| HubError::Security(String::from("aoxc binary not selected")))?;
                let bins = self.binaries.read().await;
                let bin = bins.iter().find(|b| b.id == selected_id).ok_or_else(|| {
                    HubError::Security(String::from("selected aoxc binary is unavailable"))
                })?;
                if !is_binary_allowed(env, &bin.kind) {
                    return Err(HubError::Security(String::from(
                        "selected binary source violates environment policy",
                    )));
                }
                (
                    bin.path.clone(),
                    spec.args.iter().map(|s| s.to_string()).collect(),
                )
            }
        };

        let id = format!("job-{}", chrono::Utc::now().timestamp_millis());
        self.runner
            .launch(
                id.clone(),
                command_id,
                program,
                args,
                environment_bindings(env),
                String::from("/workspace/aoxchain"),
            )
            .await?;
        Ok(id)
    }
}

impl Default for HubService {
    fn default() -> Self {
        Self::new()
    }
}

fn environment_prefix(env: Environment) -> String {
    format!(
        "AOXC_ENV={} AOXC_HOME={} AOXHUB_CONFIG={}",
        env.slug(),
        env.aoxc_home(),
        env.root_config_path()
    )
}

fn environment_bindings(env: Environment) -> Vec<(String, String)> {
    vec![
        (String::from("AOXC_ENV"), String::from(env.slug())),
        (String::from("AOXC_HOME"), String::from(env.aoxc_home())),
        (
            String::from("AOXHUB_CONFIG"),
            String::from(env.root_config_path()),
        ),
    ]
}


fn policy_note(
    env: Environment,
    command_id: &str,
    risk: crate::domain::RiskClass,
    allowed: bool,
) -> String {
    if !allowed {
        return String::from("Blocked by active environment policy");
    }

    if env == Environment::Mainnet && matches!(risk, crate::domain::RiskClass::High) {
        return format!("Allowed high-risk command in mainnet: {}", command_id);
    }

    String::from("Allowed by active environment policy")
}

pub fn is_binary_allowed(env: Environment, kind: &BinarySourceKind) -> bool {
    match env {
        Environment::Mainnet => matches!(
            kind,
            BinarySourceKind::InstalledRelease
                | BinarySourceKind::VersionedBundle
                | BinarySourceKind::GithubRelease
        ),
        Environment::Testnet => matches!(
            kind,
            BinarySourceKind::InstalledRelease
                | BinarySourceKind::VersionedBundle
                | BinarySourceKind::GithubRelease
                | BinarySourceKind::LocalReleaseBuild
                | BinarySourceKind::CustomPath
        ),
    }
}
