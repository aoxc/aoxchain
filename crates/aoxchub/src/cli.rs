use std::fmt;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "aoxchub",
    version,
    about = "AOXC Hub control binary for desktop and operator automation",
    long_about = "AOXC Hub unifies desktop launch and operator automation workflows.\n\
Supports profile-isolated execution and machine-readable diagnostics."
)]
pub struct Cli {
    /// Runtime profile for compatible environment loading.
    #[arg(long, short = 'p', default_value_t = Profile::Dev)]
    pub profile: Profile,

    /// Optional AOXCData root used to resolve binary and state directories.
    #[arg(long, env = "AOXCDATA_HOME")]
    pub aoxcdata_home: Option<PathBuf>,

    /// Render command output in table or json format.
    #[arg(long, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Skip desktop UI and execute command-mode workflows.
    #[arg(long, default_value_t = false)]
    pub headless: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Prints resolved runtime paths and compatibility settings.
    Paths,
    /// Performs deterministic compatibility checks for operator execution.
    Doctor,
    /// Explicitly launches the desktop UI.
    Launch,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::Json => write!(f, "json"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum Profile {
    Dev,
    Testnet,
    Mainnet,
    Real,
}

impl Profile {
    pub fn config_hint(self) -> &'static str {
        match self {
            Self::Dev => "config/app.development.toml",
            Self::Testnet => "config/app.development.toml",
            Self::Mainnet => "config/app.production.toml",
            Self::Real => "config/app.production.toml",
        }
    }

    pub fn chain_hint(self) -> &'static str {
        match self {
            Self::Dev => "aoxchain-dev",
            Self::Testnet => "aoxchain-testnet",
            Self::Mainnet => "aoxchain-mainnet",
            Self::Real => "aoxchain-mainnet",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dev => write!(f, "dev"),
            Self::Testnet => write!(f, "testnet"),
            Self::Mainnet => write!(f, "mainnet"),
            Self::Real => write!(f, "real"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RuntimeLayout {
    pub profile: String,
    pub chain_id: String,
    pub config_file: String,
    pub aoxc_bin: String,
    pub aoxchub_bin: String,
    pub aoxckit_bin: String,
    pub ledger_db: String,
    pub logs: String,
    pub keys: String,
}

impl RuntimeLayout {
    pub fn from_cli(cli: &Cli) -> Self {
        let base = cli
            .aoxcdata_home
            .clone()
            .unwrap_or_else(|| PathBuf::from("~/.AOXCData"));

        Self {
            profile: cli.profile.to_string(),
            chain_id: cli.profile.chain_hint().to_string(),
            config_file: cli.profile.config_hint().to_string(),
            aoxc_bin: base.join("bin/aoxc").display().to_string(),
            aoxchub_bin: base.join("bin/aoxchub").display().to_string(),
            aoxckit_bin: base.join("bin/aoxckit").display().to_string(),
            ledger_db: base
                .join("home/default/ledger/db/main.redb")
                .display()
                .to_string(),
            logs: base.join("logs").display().to_string(),
            keys: base.join("keys").display().to_string(),
        }
    }

    pub fn render_table(&self) -> String {
        [
            format!("profile      : {}", self.profile),
            format!("chain_id     : {}", self.chain_id),
            format!("config_file  : {}", self.config_file),
            format!("aoxc_bin     : {}", self.aoxc_bin),
            format!("aoxchub_bin  : {}", self.aoxchub_bin),
            format!("aoxckit_bin  : {}", self.aoxckit_bin),
            format!("ledger_db    : {}", self.ledger_db),
            format!("logs         : {}", self.logs),
            format!("keys         : {}", self.keys),
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, Profile};

    #[test]
    fn supports_profile_real_for_compatibility() {
        let cli = Cli::parse_from(["aoxchub", "--profile", "real"]);
        assert_eq!(cli.profile, Profile::Real);
    }

    #[test]
    fn parses_doctor_subcommand() {
        let cli = Cli::parse_from(["aoxchub", "--headless", "doctor"]);
        assert!(matches!(cli.command, Some(Command::Doctor)));
        assert!(cli.headless);
    }
}
