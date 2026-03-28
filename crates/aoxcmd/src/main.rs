// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcmd::cli::run_cli;
use std::process;

/// Entrypoint for the AOXC operator command plane.
///
/// Security and operational objectives:
/// - Preserve a minimal and deterministic process boundary.
/// - Emit operator-visible failures exclusively through stderr.
/// - Enforce a single-line failure contract suitable for shells, CI, and log shippers.
/// - Exit strictly with the application-defined process status code.
fn main() {
    if let Err(error) = run_cli() {
        let message = sanitize_for_stderr(&error.to_string());

        eprintln!(
            "AOXCMD_ERROR code={} exit={} message=\"{}\"",
            error.code(),
            error.exit_code(),
            message
        );

        process::exit(error.exit_code());
    }
}

/// Normalizes stderr-bound error text into a shell-safe, single-line form.
///
/// Rationale:
/// - Prevents multiline log injection into terminal sessions and CI collectors.
/// - Removes control characters that may degrade readability or parsing reliability.
/// - Preserves semantic meaning while enforcing a stable stderr contract.
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
