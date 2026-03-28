// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

mod keyforge;

use clap::Parser;
use keyforge::cli::{Cli, Commands};
use std::process;

/// Entrypoint for the AOXC keyforge operator command plane.
///
/// Operational objectives:
/// - preserve a narrow and deterministic process boundary,
/// - dispatch exactly one validated CLI command,
/// - emit failures exclusively through stderr,
/// - keep stderr output shell-safe, log-safe, and single-line,
/// - terminate with a stable non-zero exit code on command failure.
fn main() {
    process::exit(run());
}

/// Executes the AOXC keyforge CLI and returns a process exit code.
///
/// Exit policy:
/// - `0` indicates successful command completion,
/// - `1` indicates a command-layer failure already rendered to stderr.
fn run() -> i32 {
    let cli = Cli::parse();

    match dispatch(cli) {
        Ok(()) => 0,
        Err(error) => {
            emit_stderr_error(&error);
            1
        }
    }
}

/// Dispatches the parsed CLI command into the appropriate keyforge handler.
///
/// Design goals:
/// - keep command routing explicit,
/// - preserve module-level ownership of business logic,
/// - avoid embedding handler logic directly inside the entrypoint.
fn dispatch(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Key(command) => keyforge::cmd_key::handle(command),
        Commands::ActorId(command) => keyforge::cmd_actor_id::handle(command),
        Commands::Cert(command) => keyforge::cmd_cert::handle(command),
        Commands::Passport(command) => keyforge::cmd_passport::handle(command),
        Commands::Keyfile(command) => keyforge::cmd_keyfile::handle(command),
        Commands::Registry(command) => keyforge::cmd_registry::handle(command),
        Commands::Revoke(command) => keyforge::cmd_revoke::handle(command),
        Commands::Quorum(command) => keyforge::cmd_quorum::handle(command),
        Commands::ZkpSetup(command) => keyforge::cmd_zkp_setup::handle(command),
    }
}

/// Emits a single-line stderr error record for operator and automation use.
///
/// Error rendering policy:
/// - control characters are removed,
/// - multiline messages are collapsed into a single line,
/// - double quotes are normalized to reduce shell/log parsing ambiguity.
fn emit_stderr_error(error: &str) {
    let sanitized = sanitize_for_stderr(error);
    eprintln!("AOXCKIT_ERROR exit=1 message=\"{}\"", sanitized);
}

/// Normalizes stderr-bound text into a shell-safe, single-line representation.
///
/// Rationale:
/// - prevents multiline log injection,
/// - reduces terminal control-character abuse,
/// - preserves a stable human-readable and machine-friendly error contract.
fn sanitize_for_stderr(message: &str) -> String {
    message
        .chars()
        .map(|ch| match ch {
            '\n' | '\r' | '\t' => ' ',
            '"' => '\'',
            _ if ch.is_control() => ' ',
            _ => ch,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_for_stderr_replaces_control_characters_and_quotes() {
        let sanitized = sanitize_for_stderr("bad\tinput\nwith\r\"quotes\"");

        assert_eq!(sanitized, "bad input with 'quotes'");
    }

    #[test]
    fn sanitize_for_stderr_collapses_repeated_whitespace() {
        let sanitized = sanitize_for_stderr("one   two\t\tthree\n\nfour");

        assert_eq!(sanitized, "one two three four");
    }

    #[test]
    fn sanitize_for_stderr_removes_non_printable_control_characters() {
        let input = format!("alpha{}beta{}gamma", '\u{0007}', '\u{001B}');
        let sanitized = sanitize_for_stderr(&input);

        assert_eq!(sanitized, "alpha beta gamma");
    }
}
