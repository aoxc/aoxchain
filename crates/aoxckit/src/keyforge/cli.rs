// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

/// Canonical AOXC keyforge command-line interface.
///
/// Design objectives:
/// - enforce a strict and explicit operator contract,
/// - reject ambiguous or blank input at parse time whenever possible,
/// - clearly separate public-safe operations from dangerous plaintext flows,
/// - preserve a predictable machine-friendly CLI surface suitable for automation.
#[derive(Debug, Parser)]
#[command(
    name = "aoxc-keyforge",
    version,
    about = "AOXC identity and key management CLI",
    long_about = "AOXC identity and key management CLI with explicit security boundaries for public output, encrypted custody, and operational review.",
    arg_required_else_help = true,
    subcommand_required = true,
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level AOXC keyforge command namespace.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Key generation, inspection, and custody-safe export operations.
    Key(KeyCommand),

    /// Actor identifier generation and inspection workflows.
    ActorId(ActorIdCommand),

    /// Certificate issuance, verification, inspection, and template generation workflows.
    Cert(CertCommand),

    /// Passport inspection workflows.
    Passport(PassportCommand),

    /// Keyfile encryption and decryption workflows.
    Keyfile(KeyfileCommand),

    /// Registry mutation and listing workflows.
    Registry(RegistryCommand),

    /// Revocation workflows.
    Revoke(RevokeCommand),

    /// Quorum evaluation workflows.
    Quorum(QuorumCommand),

    /// Zero-knowledge setup initialization workflows.
    ZkpSetup(ZkpSetupCommand),
}

/// Key namespace root.
///
/// Security model:
/// - public output operations must never emit plaintext private material,
/// - encrypted persistence is the preferred custody path,
/// - dangerous plaintext export must remain explicit and rare.
#[derive(Debug, Args)]
pub struct KeyCommand {
    #[command(subcommand)]
    pub command: KeySubcommand,
}

/// Supported output encodings for operator-visible command responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    PrettyJson,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::PrettyJson
    }
}

/// Canonical key subcommands.
///
/// Important security note:
/// `generate` is intentionally defined as a public-only operation.
/// It must not emit secret key material to stdout.
#[derive(Debug, Subcommand)]
pub enum KeySubcommand {
    /// Generates a fresh keypair and emits only the public operational view.
    ///
    /// Backward compatibility:
    /// - `generate` remains the canonical subcommand name.
    /// - handlers should treat this as a public-safe operation only.
    #[command(name = "generate", visible_alias = "generate-public")]
    GeneratePublic {
        /// Output format used for the public response payload.
        #[arg(long, value_enum, default_value_t = OutputFormat::PrettyJson)]
        format: OutputFormat,
    },

    /// Generates encrypted key custody material and writes it to disk.
    ///
    /// This command is the preferred production-grade path for key generation
    /// because it avoids plaintext secret emission to terminal output.
    #[command(name = "generate-keyfile")]
    GenerateKeyfile {
        /// Destination path for the encrypted keyfile artifact.
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        output: String,

        /// Inline password used to encrypt the keyfile.
        ///
        /// Operational guidance:
        /// prefer `--password-file` in automated or auditable environments
        /// to reduce shell history exposure.
        #[arg(
            long,
            value_parser = non_empty_trimmed_arg,
            conflicts_with = "password_file",
            required_unless_present = "password_file"
        )]
        password: Option<String>,

        /// Path to a file that contains the encryption password.
        #[arg(
            long,
            value_parser = non_empty_trimmed_arg,
            conflicts_with = "password",
            required_unless_present = "password"
        )]
        password_file: Option<String>,

        /// Replaces an existing output file.
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Reads, validates, and reprints a canonical AOXC node key bundle.
    #[command(name = "inspect-bundle")]
    InspectBundle {
        /// Path to the serialized node key bundle.
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        file: String,

        /// Output format used for the inspected bundle payload.
        #[arg(long, value_enum, default_value_t = OutputFormat::PrettyJson)]
        format: OutputFormat,
    },
}

/// Actor identifier namespace root.
#[derive(Debug, Args)]
pub struct ActorIdCommand {
    #[command(subcommand)]
    pub command: ActorIdSubcommand,
}

/// Actor identifier workflows.
#[derive(Debug, Subcommand)]
pub enum ActorIdSubcommand {
    /// Generates an AOXC actor identifier from a public key and role metadata.
    Generate {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        pubkey: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        role: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        zone: String,
    },
}

/// Certificate namespace root.
#[derive(Debug, Args)]
pub struct CertCommand {
    #[command(subcommand)]
    pub command: CertSubcommand,
}

/// Certificate workflows.
#[derive(Debug, Subcommand)]
pub enum CertSubcommand {
    /// Issues a certificate from validated identity metadata.
    Issue {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        chain: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        actor_id: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        role: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        zone: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        pubkey: String,

        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        issued_at: u64,

        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        expires_at: u64,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        issuer: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        output: Option<String>,
    },

    /// Verifies a certificate against the supplied issuer.
    Verify {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        file: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        issuer: String,
    },

    /// Inspects a serialized certificate.
    Inspect {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        file: String,
    },

    /// Generates an mTLS certificate template.
    #[command(name = "generate-mtls")]
    GenerateMtls {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        common_name: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        output: String,
    },
}

/// Passport namespace root.
#[derive(Debug, Args)]
pub struct PassportCommand {
    #[command(subcommand)]
    pub command: PassportSubcommand,
}

/// Passport workflows.
#[derive(Debug, Subcommand)]
pub enum PassportSubcommand {
    /// Inspects a serialized passport document.
    Inspect {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        file: String,
    },
}

/// Shared password source arguments.
///
/// Security posture:
/// - callers must provide exactly one password source,
/// - inline passwords are allowed for compatibility,
/// - password files are preferred for operational hygiene.
#[derive(Debug, Clone, Args)]
#[command(group(
    ArgGroup::new("password_source")
        .args(["password", "password_file"])
        .required(true)
))]
pub struct PasswordSourceArgs {
    /// Inline password supplied directly on the command line.
    #[arg(long, value_parser = non_empty_trimmed_arg)]
    pub password: Option<String>,

    /// Path to a file that contains the password.
    #[arg(long, value_parser = non_empty_trimmed_arg)]
    pub password_file: Option<String>,
}

/// Keyfile namespace root.
#[derive(Debug, Args)]
pub struct KeyfileCommand {
    #[command(subcommand)]
    pub command: KeyfileSubcommand,
}

/// Keyfile workflows.
///
/// Security posture:
/// - encryption is safe-by-default,
/// - decryption into plaintext files is explicitly marked dangerous and gated.
#[derive(Debug, Subcommand)]
pub enum KeyfileSubcommand {
    /// Encrypts plaintext key material into a canonical AOXC keyfile envelope.
    Encrypt {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        input: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        output: String,

        #[command(flatten)]
        password: PasswordSourceArgs,

        /// Replaces an existing output file.
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Decrypts an AOXC keyfile into plaintext output.
    ///
    /// Danger:
    /// this operation writes raw secret material back to disk and therefore
    /// requires an explicit operator acknowledgement.
    Decrypt {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        input: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        output: String,

        #[command(flatten)]
        password: PasswordSourceArgs,

        /// Explicit acknowledgement required before plaintext secret output is allowed.
        #[arg(long, default_value_t = false)]
        allow_plaintext_output: bool,

        /// Replaces an existing output file.
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

/// Registry namespace root.
#[derive(Debug, Args)]
pub struct RegistryCommand {
    #[command(subcommand)]
    pub command: RegistrySubcommand,
}

/// Registry workflows.
#[derive(Debug, Subcommand)]
pub enum RegistrySubcommand {
    /// Inserts or updates a registry entry.
    #[command(name = "upsert-entry")]
    UpsertEntry {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        registry: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        actor_id: String,

        #[arg(long, default_value = "active", value_parser = non_empty_trimmed_arg)]
        status: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        reason: Option<String>,
    },

    /// Lists entries from a registry.
    List {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        registry: String,
    },
}

/// Revocation namespace root.
#[derive(Debug, Args)]
pub struct RevokeCommand {
    #[command(subcommand)]
    pub command: RevokeSubcommand,
}

/// Revocation workflows.
#[derive(Debug, Subcommand)]
pub enum RevokeSubcommand {
    /// Revokes an actor from the supplied registry.
    Actor {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        registry: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        actor_id: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        reason: String,
    },
}

/// Quorum namespace root.
#[derive(Debug, Args)]
pub struct QuorumCommand {
    #[command(subcommand)]
    pub command: QuorumSubcommand,
}

/// Quorum evaluation workflows.
#[derive(Debug, Subcommand)]
pub enum QuorumSubcommand {
    /// Evaluates quorum status against a total membership and approval count.
    Evaluate {
        #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
        total: u16,

        #[arg(long, value_parser = clap::value_parser!(u16).range(0..))]
        approvals: u16,

        #[arg(long, default_value_t = 6667, value_parser = clap::value_parser!(u16).range(1..=10_000))]
        threshold_bps: u16,
    },
}

/// Zero-knowledge setup namespace root.
#[derive(Debug, Args)]
pub struct ZkpSetupCommand {
    #[command(subcommand)]
    pub command: ZkpSetupSubcommand,
}

/// Zero-knowledge setup workflows.
#[derive(Debug, Subcommand)]
pub enum ZkpSetupSubcommand {
    /// Initializes a trusted setup artifact for the requested circuit.
    Init {
        #[arg(long, value_parser = non_empty_trimmed_arg)]
        circuit: String,

        #[arg(long, value_parser = non_empty_trimmed_arg)]
        output: String,

        #[arg(long, default_value_t = 18, value_parser = clap::value_parser!(u8).range(1..=28))]
        powers: u8,
    },
}

/// Rejects blank or whitespace-only command-line values.
///
/// This parser intentionally trims only for validation purposes and returns the
/// original owned value so that downstream handlers can preserve exact input if
/// their operational contract requires it.
fn non_empty_trimmed_arg(value: &str) -> Result<String, String> {
    if value.trim().is_empty() {
        return Err("value must not be blank".to_string());
    }

    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn key_generate_public_parses_with_default_pretty_json_format() {
        let cli = Cli::try_parse_from(["aoxc-keyforge", "key", "generate"])
            .expect("public generate command must parse");

        match cli.command {
            Commands::Key(KeyCommand {
                command: KeySubcommand::GeneratePublic { format },
            }) => {
                assert_eq!(format, OutputFormat::PrettyJson);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn key_generate_keyfile_accepts_password_file_source() {
        let cli = Cli::try_parse_from([
            "aoxc-keyforge",
            "key",
            "generate-keyfile",
            "--output",
            "keys/operator-key.json",
            "--password-file",
            "secrets/keyforge.pass",
        ])
        .expect("generate-keyfile command must parse");

        match cli.command {
            Commands::Key(KeyCommand {
                command:
                    KeySubcommand::GenerateKeyfile {
                        output,
                        password,
                        password_file,
                        force,
                    },
            }) => {
                assert_eq!(output, "keys/operator-key.json");
                assert_eq!(password, None);
                assert_eq!(password_file.as_deref(), Some("secrets/keyforge.pass"));
                assert!(!force);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn key_generate_keyfile_rejects_missing_password_source() {
        let result = Cli::try_parse_from([
            "aoxc-keyforge",
            "key",
            "generate-keyfile",
            "--output",
            "keys/operator-key.json",
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn keyfile_encrypt_requires_exactly_one_password_source() {
        let result = Cli::try_parse_from([
            "aoxc-keyforge",
            "keyfile",
            "encrypt",
            "--input",
            "plain.bin",
            "--output",
            "secret.keyfile.json",
            "--password",
            "Inline#2026!",
            "--password-file",
            "pw.txt",
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn keyfile_decrypt_parses_explicit_plaintext_acknowledgement() {
        let cli = Cli::try_parse_from([
            "aoxc-keyforge",
            "keyfile",
            "decrypt",
            "--input",
            "secret.keyfile.json",
            "--output",
            "plain.bin",
            "--password",
            "Inline#2026!",
            "--allow-plaintext-output",
            "--force",
        ])
        .expect("dangerous decrypt command must parse when explicitly acknowledged");

        match cli.command {
            Commands::Keyfile(KeyfileCommand {
                command:
                    KeyfileSubcommand::Decrypt {
                        input,
                        output,
                        allow_plaintext_output,
                        force,
                        ..
                    },
            }) => {
                assert_eq!(input, "secret.keyfile.json");
                assert_eq!(output, "plain.bin");
                assert!(allow_plaintext_output);
                assert!(force);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn blank_values_are_rejected_at_parse_time() {
        let result = Cli::try_parse_from(["aoxc-keyforge", "cert", "inspect", "--file", "   "]);

        assert!(result.is_err());
    }

    #[test]
    fn quorum_threshold_bps_out_of_range_is_rejected() {
        let result = Cli::try_parse_from([
            "aoxc-keyforge",
            "quorum",
            "evaluate",
            "--total",
            "10",
            "--approvals",
            "7",
            "--threshold-bps",
            "10001",
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn zkp_setup_powers_out_of_range_is_rejected() {
        let result = Cli::try_parse_from([
            "aoxc-keyforge",
            "zkp-setup",
            "init",
            "--circuit",
            "identity-v1",
            "--output",
            "setup.params",
            "--powers",
            "0",
        ]);

        assert!(result.is_err());
    }
}
