use crate::environments::Environment;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum TrustLevel {
    Trusted,
    Verified,
    Experimental,
    Unverified,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum BinarySourceKind {
    InstalledRelease,
    VersionedBundle,
    GithubRelease,
    LocalReleaseBuild,
    CustomPath,
}

#[derive(Clone, Debug, Serialize)]
pub struct BinaryCandidate {
    pub id: String,
    pub kind: BinarySourceKind,
    pub path: String,
    pub version: Option<String>,
    pub trust: TrustLevel,
    pub checksum_verified: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub enum RiskClass {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Serialize)]
pub enum CommandProgram {
    Aoxc,
    Make,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandSpec {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub risk: RiskClass,
    pub program: CommandProgram,
    pub args: &'static [&'static str],
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandView {
    pub spec: CommandSpec,
    pub preview: String,
    pub allowed: bool,
    pub policy_note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct JobStatus {
    pub id: String,
    pub command_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub output: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct EnvironmentBinding {
    pub slug: &'static str,
    pub root_config: &'static str,
    pub aoxc_home: &'static str,
    pub make_scope: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct HubStateView {
    pub environment: Environment,
    pub banner: &'static str,
    pub binding: EnvironmentBinding,
    pub selected_binary_id: Option<String>,
    pub binaries: Vec<BinaryCandidate>,
    pub commands: Vec<CommandView>,
}
