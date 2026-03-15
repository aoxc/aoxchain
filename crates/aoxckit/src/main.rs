mod keyforge;

use clap::Parser;
use keyforge::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Key(command) => keyforge::cmd_key::handle(command),
        Commands::ActorId(command) => keyforge::cmd_actor_id::handle(command),
        Commands::Cert(command) => keyforge::cmd_cert::handle(command),
        Commands::Passport(command) => keyforge::cmd_passport::handle(command),
        Commands::Keyfile(command) => keyforge::cmd_keyfile::handle(command),
        Commands::Registry(command) => keyforge::cmd_registry::handle(command),
        Commands::Revoke(command) => keyforge::cmd_revoke::handle(command),
        Commands::Quorum(command) => keyforge::cmd_quorum::handle(command),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
