// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcmd::cli::run_cli;
use std::process;

fn main() {
    match run_cli() {
        Ok(()) => process::exit(0),
        Err(error) => {
            eprintln!(
                "AOXCMD_ERROR code={} exit={} message={}",
                error.code(),
                error.exit_code(),
                error
            );
            process::exit(error.exit_code());
        }
    }
}
