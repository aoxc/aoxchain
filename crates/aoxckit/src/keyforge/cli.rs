// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "aoxc-keyforge")]
#[command(about = "AOXC identity and key management CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Key(KeyCommand),
    ActorId(ActorIdCommand),
    Cert(CertCommand),
    Passport(PassportCommand),
    Keyfile(KeyfileCommand),
    Registry(RegistryCommand),
    Revoke(RevokeCommand),
    Quorum(QuorumCommand),
    ZkpSetup(ZkpSetupCommand),
}

#[derive(Debug, Args)]
pub struct KeyCommand {
    #[command(subcommand)]
    pub command: KeySubcommand,
}

#[derive(Debug, Subcommand)]
pub enum KeySubcommand {
    Generate,
    InspectBundle {
        #[arg(long)]
        file: String,
    },
}

#[derive(Debug, Args)]
pub struct ActorIdCommand {
    #[command(subcommand)]
    pub command: ActorIdSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ActorIdSubcommand {
    Generate {
        #[arg(long)]
        pubkey: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        zone: String,
    },
}

#[derive(Debug, Args)]
pub struct CertCommand {
    #[command(subcommand)]
    pub command: CertSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CertSubcommand {
    Issue {
        #[arg(long)]
        chain: String,
        #[arg(long)]
        actor_id: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        zone: String,
        #[arg(long)]
        pubkey: String,
        #[arg(long)]
        issued_at: u64,
        #[arg(long)]
        expires_at: u64,
        #[arg(long)]
        issuer: String,
        #[arg(long)]
        output: Option<String>,
    },
    Verify {
        #[arg(long)]
        file: String,
        #[arg(long)]
        issuer: String,
    },
    Inspect {
        #[arg(long)]
        file: String,
    },
    GenerateMtls {
        #[arg(long)]
        common_name: String,
        #[arg(long)]
        output: String,
    },
}

#[derive(Debug, Args)]
pub struct PassportCommand {
    #[command(subcommand)]
    pub command: PassportSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum PassportSubcommand {
    Inspect {
        #[arg(long)]
        file: String,
    },
}

#[derive(Debug, Args)]
pub struct KeyfileCommand {
    #[command(subcommand)]
    pub command: KeyfileSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum KeyfileSubcommand {
    Encrypt {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        password: String,
    },
    Decrypt {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        password: String,
    },
}

#[derive(Debug, Args)]
pub struct RegistryCommand {
    #[command(subcommand)]
    pub command: RegistrySubcommand,
}

#[derive(Debug, Subcommand)]
pub enum RegistrySubcommand {
    UpsertEntry {
        #[arg(long)]
        registry: String,
        #[arg(long)]
        actor_id: String,
        #[arg(long, default_value = "active")]
        status: String,
        #[arg(long)]
        reason: Option<String>,
    },
    List {
        #[arg(long)]
        registry: String,
    },
}

#[derive(Debug, Args)]
pub struct RevokeCommand {
    #[command(subcommand)]
    pub command: RevokeSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum RevokeSubcommand {
    Actor {
        #[arg(long)]
        registry: String,
        #[arg(long)]
        actor_id: String,
        #[arg(long)]
        reason: String,
    },
}

#[derive(Debug, Args)]
pub struct QuorumCommand {
    #[command(subcommand)]
    pub command: QuorumSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum QuorumSubcommand {
    Evaluate {
        #[arg(long)]
        total: u16,
        #[arg(long)]
        approvals: u16,
        #[arg(long, default_value_t = 6667)]
        threshold_bps: u16,
    },
}

#[derive(Debug, Args)]
pub struct ZkpSetupCommand {
    #[command(subcommand)]
    pub command: ZkpSetupSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ZkpSetupSubcommand {
    Init {
        #[arg(long)]
        circuit: String,
        #[arg(long)]
        output: String,
        #[arg(long, default_value_t = 18)]
        powers: u8,
    },
}
