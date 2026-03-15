use aoxcmd::keys::{KeyBootstrapRequest, KeyManager, KeyPaths};
use aoxcmd::node::engine::produce_single_block;
use aoxcmd::node::state;

use aoxcnet::gossip::consensus_gossip::GossipEngine;
use aoxcore::identity::ca::CertificateAuthority;
use aoxcunity::messages::ConsensusMessage;
use aoxcunity::vote::{Vote, VoteKind};

use std::env;
use std::process;

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("AOXCMD_ERROR: {error}");
        process::exit(1);
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        "key-bootstrap" => cmd_key_bootstrap(&args[2..]),
        "node-bootstrap" => cmd_node_bootstrap(),
        "produce-once" => cmd_produce_once(&args[2..]),
        "network-smoke" => cmd_network_smoke(),
        other => Err(format!("unknown command: {other}")),
    }
}

fn cmd_key_bootstrap(args: &[String]) -> Result<(), String> {
    let base_dir = arg_value(args, "--base-dir").unwrap_or_else(|| "AOXC_DATA/keys".to_string());
    let name = arg_value(args, "--name").unwrap_or_else(|| "node".to_string());
    let chain = arg_value(args, "--chain").unwrap_or_else(|| "AOXC-MAIN".to_string());
    let role = arg_value(args, "--role").unwrap_or_else(|| "validator".to_string());
    let zone = arg_value(args, "--zone").unwrap_or_else(|| "core".to_string());
    let issuer = arg_value(args, "--issuer").unwrap_or_else(|| "AOXC-ROOT-CA".to_string());
    let password = arg_value(args, "--password")
        .ok_or_else(|| "--password is required for key-bootstrap".to_string())?;

    let validity_secs: u64 = arg_value(args, "--validity-secs")
        .unwrap_or_else(|| "31536000".to_string())
        .parse()
        .map_err(|_| "--validity-secs must be a valid u64".to_string())?;

    let paths = KeyPaths::new(base_dir, &name);
    let request = KeyBootstrapRequest::new(chain, role, zone, password, validity_secs);
    let manager = KeyManager::new(paths, request);
    let ca = CertificateAuthority::new(issuer);

    let material = manager
        .load_or_create(&ca)
        .map_err(|error| format!("key bootstrap failed [{}]: {}", error.code(), error))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&material.summary())
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_node_bootstrap() -> Result<(), String> {
    let node = state::setup().map_err(|error| error.to_string())?;

    let output = serde_json::json!({
        "mempool_max_txs": node.mempool.config().max_txs,
        "mempool_max_tx_size": node.mempool.config().max_tx_size,
        "validator_count": node.rotation.validators().len(),
        "quorum": {
            "numerator": node.consensus.quorum.numerator,
            "denominator": node.consensus.quorum.denominator
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_produce_once(args: &[String]) -> Result<(), String> {
    let tx = arg_value(args, "--tx").unwrap_or_else(|| "AOXC_DEMO_TX".to_string());

    let mut node = state::setup().map_err(|error| error.to_string())?;
    let outcome = produce_single_block(&mut node, vec![tx.into_bytes()])?;

    let output = serde_json::json!({
        "height": outcome.block.header.height,
        "hash": hex::encode(outcome.block.hash),
        "parent": hex::encode(outcome.block.header.parent_hash),
        "finalized": outcome.seal.is_some(),
        "seal": outcome.seal,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_network_smoke() -> Result<(), String> {
    let mut gossip = GossipEngine::new();

    let vote = Vote {
        voter: [7u8; 32],
        block_hash: [9u8; 32],
        height: 1,
        round: 0,
        kind: VoteKind::Prepare,
    };

    gossip.broadcast(ConsensusMessage::Vote(vote));
    let inbound = gossip.receive();

    let output = serde_json::json!({
        "transport": "stub",
        "broadcast": "ok",
        "inbound_message": inbound,
        "note": "None is expected until p2p transport integration is implemented"
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find_map(|window| {
        if window[0] == key {
            Some(window[1].clone())
        } else {
            None
        }
    })
}

fn print_usage() {
    println!(
        "AOXC Command Surface\n\nCommands:\n  key-bootstrap --password <secret> [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]\n  node-bootstrap\n  produce-once [--tx <payload>]\n  network-smoke\n  help\n"
    );
}
