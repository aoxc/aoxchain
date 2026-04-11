use crate::error::{AppError, ErrorCode};
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeMap, env};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

/// Returns the value immediately following a flag, if present.
///
/// Example:
/// - `--format json` => `Some("json")`
/// - `--format` without a following value => `None`
pub fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

/// Returns true when the provided flag is present in the argument vector.
pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|item| item == flag)
}

/// Resolves the requested operator output format.
///
/// The function defaults to text output because text is the safest interactive
/// operator mode for manual terminal use. Structured machine-facing consumers
/// should explicitly request `json` or `yaml`.
pub fn output_format(args: &[String]) -> OutputFormat {
    match arg_value(args, "--format").as_deref() {
        Some("json") => OutputFormat::Json,
        Some("yaml") => OutputFormat::Yaml,
        _ => OutputFormat::Text,
    }
}

/// Detects the preferred language from the ambient process locale.
///
/// The current CLI surface supports Turkish and English selection semantics.
/// Any non-Turkish locale currently falls back to English.
pub fn detect_language(_args: &[String]) -> &'static str {
    let language = env::var("LANG").unwrap_or_default().to_ascii_lowercase();
    if language.starts_with("tr") {
        "tr"
    } else {
        "en"
    }
}

/// Returns a stable unknown-command application error.
pub fn localized_unknown_command(_lang: &str, command: &str) -> AppError {
    let suggestion = suggest_command(command)
        .map(|value| format!(" Did you mean '{value}'?"))
        .unwrap_or_else(|| " Use 'aoxc --help' to list supported commands.".to_string());

    AppError::new(
        ErrorCode::UsageUnknownCommand,
        format!("Unknown command: {command}.{suggestion}"),
    )
}

/// Returns true when the argument slice requests command help.
pub fn asks_for_help(args: &[String]) -> bool {
    args.first()
        .map(|value| matches!(value.as_str(), "help" | "--help" | "-h"))
        .unwrap_or(false)
}

/// Returns true when a single token requests help.
pub fn is_help_token(value: &str) -> bool {
    matches!(value, "help" | "--help" | "-h")
}

/// Prints help for a group/subcommand pair.
///
/// Returns `true` when a dedicated subcommand help surface exists.
pub fn print_subcommand_usage(group: &str, subcommand: &str) -> bool {
    let body = match (group, subcommand) {
        ("chain", "init") => {
            "USAGE\n  aoxc chain init [--profile <localnet|devnet|validation|testnet|mainnet>]"
        }
        ("chain", "create") => {
            "USAGE\n  aoxc chain create --password <secret> [--profile <testnet|mainnet>]"
        }
        ("chain", "start") => "USAGE\n  aoxc chain start [--continuous|--bounded] [--rounds <n>]",
        ("chain", "status") => "USAGE\n  aoxc chain status",
        ("chain", "doctor") => "USAGE\n  aoxc chain doctor",
        ("chain", "consensus-audit") => "USAGE\n  aoxc chain consensus-audit [--profile <name>]",
        ("chain", "demo") => "USAGE\n  aoxc chain demo",

        ("genesis", "init") => "USAGE\n  aoxc genesis init",
        ("genesis", "add-validator") => {
            "USAGE\n  aoxc genesis add-validator --validator-id <id> --consensus-public-key <hex> --network-public-key <hex>"
        }
        ("genesis", "add-account") => {
            "USAGE\n  aoxc genesis add-account --account-id <id> --balance <amount>"
        }
        ("genesis", "build" | "verify") => {
            "USAGE\n  aoxc genesis build [--strict]\n  aoxc genesis verify [--strict]"
        }
        ("genesis", "finalize" | "seal" | "sign" | "freeze" | "production-gate") => {
            "USAGE\n  aoxc genesis finalize [--profile <localnet|devnet|validation|testnet|mainnet>] [--strict]"
        }
        ("genesis", "inspect") => "USAGE\n  aoxc genesis inspect",
        ("genesis", "template-advanced") => {
            "USAGE\n  aoxc genesis template-advanced [--out <path>]"
        }
        ("genesis", "advanced-system") => {
            "USAGE\n  aoxc genesis advanced-system [--profile <name>] [--out <path>]"
        }
        ("genesis", "security-audit") => "USAGE\n  aoxc genesis security-audit",
        ("genesis", "start") => "USAGE\n  aoxc genesis start [--profile <name>] [--strict]",
        ("genesis", "fingerprint") => "USAGE\n  aoxc genesis fingerprint",

        ("validator", "create") => "USAGE\n  aoxc validator create --password <secret>",
        ("validator", "join" | "register") => {
            "USAGE\n  aoxc validator join --validator-id <id> --password <secret>"
        }
        ("validator", "activate" | "bond") => {
            "USAGE\n  aoxc validator activate --validator-id <id> [--stake <amount>]"
        }
        ("validator", "unbond") => {
            "USAGE\n  aoxc validator unbond --validator-id <id> [--stake <amount>]"
        }
        ("validator", "set-status") => {
            "USAGE\n  aoxc validator set-status --validator-id <id> --status <active|jailed|inactive>"
        }
        ("validator", "commission-set") => {
            "USAGE\n  aoxc validator commission-set --validator-id <id> --commission-bps <0-10000>"
        }
        ("validator", "inspect" | "status") => "USAGE\n  aoxc validator inspect",
        ("validator", "rotate-key") => "USAGE\n  aoxc validator rotate-key --password <secret>",

        ("wallet", "create") => {
            "USAGE\n  aoxc wallet create --name <name> --profile <name> --password <secret>"
        }
        ("wallet", "balance") => "USAGE\n  aoxc wallet balance",

        ("account", "fund") => "USAGE\n  aoxc account fund --to <account-id> --amount <value>",

        ("node", "init") => "USAGE\n  aoxc node init",
        ("node", "join") => "USAGE\n  aoxc node join --seed <multiaddr|ip:port>",
        ("node", "start") => "USAGE\n  aoxc node start [--continuous|--bounded]",
        ("node", "status") => "USAGE\n  aoxc node status",
        ("node", "doctor") => "USAGE\n  aoxc node doctor",

        ("network", "create") => "USAGE\n  aoxc network create --password <secret>",
        ("network", "join" | "peer-add" | "seed-add" | "bootstrap-peer-add") => {
            "USAGE\n  aoxc network join --seed <multiaddr|ip:port>"
        }
        ("network", "start") => "USAGE\n  aoxc network start",
        ("network", "status" | "verify") => "USAGE\n  aoxc network status",
        ("network", "join-check") => "USAGE\n  aoxc network join-check",
        ("network", "identity-gate") => "USAGE\n  aoxc network identity-gate [--enforce]",
        ("network", "doctor") => "USAGE\n  aoxc network doctor",

        ("query", "chain") => "USAGE\n  aoxc query chain [status|block|tx|receipt]",
        ("query", "consensus") => {
            "USAGE\n  aoxc query consensus [status|validators|proposer|round|finality|commits|evidence]"
        }
        ("query", "vm") => {
            "USAGE\n  aoxc query vm [status|call|simulate|storage|contract|code|estimate-gas|trace]"
        }
        ("query", "full") => "USAGE\n  aoxc query full",
        (
            "query",
            "block" | "tx" | "receipt" | "account" | "balance" | "network" | "runtime"
            | "state-root" | "rpc",
        ) => {
            "USAGE\n  aoxc query <block|tx|receipt|account|balance|network|runtime|state-root|rpc>"
        }

        ("tx", "transfer") => "USAGE\n  aoxc tx transfer --to <account-id> --amount <value>",
        ("tx", "stake") => {
            "USAGE\n  aoxc tx stake <delegate|undelegate> --to <validator-id> --amount <value>"
        }

        ("api", "status" | "rpc") => "USAGE\n  aoxc api status",
        ("api", "contract" | "api-contract") => "USAGE\n  aoxc api contract",
        ("api", "smoke" | "curl-smoke") => "USAGE\n  aoxc api smoke",
        ("api", "metrics") => "USAGE\n  aoxc api metrics",
        ("api", "health") => "USAGE\n  aoxc api health",
        ("api", "full") => "USAGE\n  aoxc api full",
        (
            "api",
            "chain" | "consensus" | "vm" | "block" | "tx" | "receipt" | "account" | "balance"
            | "state-root" | "network" | "runtime",
        ) => {
            "USAGE\n  aoxc api <chain|consensus|vm|block|tx|receipt|account|balance|state-root|network|runtime>"
        }

        ("stake", "delegate") => {
            "USAGE\n  aoxc stake delegate --to <validator-id> --amount <value>"
        }
        ("stake", "undelegate") => {
            "USAGE\n  aoxc stake undelegate --to <validator-id> --amount <value>"
        }
        ("stake", "validators" | "rewards") => "USAGE\n  aoxc stake <validators|rewards>",

        ("doctor", "network" | "node" | "runtime") => "USAGE\n  aoxc doctor <network|node|runtime>",

        ("audit", "chain" | "genesis" | "validator-set") => {
            "USAGE\n  aoxc audit <chain|genesis|validator-set>"
        }
        _ => return false,
    };

    println!("{body}");
    true
}

/// Prints concise help for a routed command group.
pub fn print_group_usage(group: &str) {
    let body = match group {
        "chain" => {
            "GROUP: chain\n  init             Initialize local chain configuration\n  create           Bootstrap production profile and keys\n  start            Run node production loop\n  status           Query runtime status\n  doctor           Run diagnostics doctor\n  consensus-audit  Validate consensus profile coherence\n  demo             Run deterministic local demo\n\nEXAMPLES\n  aoxc chain init --profile testnet\n  aoxc chain create --password <secret> --profile mainnet\n  aoxc chain status"
        }
        "genesis" => {
            "GROUP: genesis\n  init              Initialize genesis document\n  add-validator     Add validator entry\n  add-account       Add funded account\n  build|verify      Validate genesis integrity\n  finalize|seal|sign|freeze\n                    Run production gate and freeze semantics\n  inspect           Print genesis document\n  fingerprint       Print deterministic genesis hash\n\nEXAMPLES\n  aoxc genesis init\n  aoxc genesis add-validator --validator-id val1 --consensus-public-key <hex> --network-public-key <hex>\n  aoxc genesis finalize --profile mainnet --strict"
        }
        "validator" => {
            "GROUP: validator\n  create            Create validator keys and identity\n  join|register     Register validator in runtime\n  activate          Activate validator\n  unbond            Unbond validator stake\n  set-status        Update validator status\n  commission-set    Update validator commission\n  inspect|status    Inspect local validator key material\n  rotate-key        Rotate validator keys\n\nEXAMPLES\n  aoxc validator create --password <secret>\n  aoxc validator join --validator-id val1 --password <secret>"
        }
        "wallet" => {
            "GROUP: wallet\n  create            Create operator wallet/account\n  balance           Show runtime economy status\n\nEXAMPLES\n  aoxc wallet create --name validator-01 --profile testnet --password <secret>\n  aoxc wallet balance"
        }
        "account" => {
            "GROUP: account\n  fund              Treasury transfer to an account\n\nEXAMPLES\n  aoxc account fund --to <account-id> --amount 100"
        }
        "node" => {
            "GROUP: node\n  init              Bootstrap local node runtime\n  join              Join network using a seed peer\n  start             Run node loop\n  status            Probe node health\n  doctor            Run diagnostics doctor\n\nEXAMPLES\n  aoxc node join --seed 127.0.0.1:19101\n  aoxc node start --continuous"
        }
        "network" => {
            "GROUP: network\n  create            Create dual profile network artifacts\n  join              Alias for node join\n  start             Run network smoke/demo flow\n  status|verify     Execute network smoke checks\n  join-check        Verify network join preconditions\n  identity-gate     Enforce network identity policy\n  doctor            Run diagnostics doctor\n\nEXAMPLES\n  aoxc network status\n  aoxc network identity-gate --enforce"
        }
        "role" => {
            "GROUP: role\n  list              List role model\n  status            Render role model status\n  activate-core7    Activate core7 profile\n\nEXAMPLES\n  aoxc role list\n  aoxc role activate-core7"
        }
        "query" => {
            "GROUP: query\n  chain             Chain status and block/tx queries\n  consensus         Consensus diagnostics\n  vm                VM diagnostics and contract inspection\n  full              Aggregate account/tx/network projection\n  block|tx|receipt|account|balance\n                    Direct single-surface queries\n  network           Network status/peer/full projection\n  runtime           Runtime status and snapshots\n  state-root        Query state root\n  rpc               Query RPC status\n\nEXAMPLES\n  aoxc query chain status\n  aoxc query vm trace"
        }
        "api" => {
            "GROUP: api\n  status|rpc        RPC status\n  contract          API contract descriptor\n  smoke             RPC curl smoke validation\n  metrics           Metrics projection\n  health            Runtime health surface\n  full              Aggregate projection\n  chain|consensus|vm|network|runtime\n                    Routed query aliases\n\nEXAMPLES\n  aoxc api status\n  aoxc api full --account-id <id>"
        }
        "tx" => {
            "GROUP: tx\n  transfer          Treasury transfer alias\n  stake delegate    Stake delegation flow\n  stake undelegate  Stake undelegation flow\n\nEXAMPLES\n  aoxc tx transfer --to <account-id> --amount 10\n  aoxc tx stake delegate --to validator-01 --amount 100"
        }
        "stake" => {
            "GROUP: stake\n  delegate          Delegate stake to validator\n  undelegate        Undelegate stake\n  validators        Runtime validator staking overview\n  rewards           Runtime reward overview\n\nEXAMPLES\n  aoxc stake delegate --to validator-01 --amount 250"
        }
        "doctor" => {
            "GROUP: doctor\n  node|network|runtime  Run diagnostics doctor by domain\n\nEXAMPLES\n  aoxc doctor network"
        }
        "audit" => {
            "GROUP: audit\n  chain|genesis|validator-set  Run production audit view\n\nEXAMPLES\n  aoxc audit chain"
        }
        _ => "No dedicated help exists for this group yet. Use 'aoxc --help' for the full surface.",
    };

    println!("{body}");
}

fn suggest_command(command: &str) -> Option<&'static str> {
    const CANDIDATES: &[&str] = &[
        "chain",
        "genesis",
        "validator",
        "wallet",
        "account",
        "node",
        "network",
        "role",
        "api",
        "query",
        "tx",
        "stake",
        "doctor",
        "audit",
        "version",
        "help",
    ];

    CANDIDATES
        .iter()
        .map(|candidate| (*candidate, levenshtein(command, candidate)))
        .filter(|(_, distance)| *distance <= 3)
        .min_by_key(|(_, distance)| *distance)
        .map(|(candidate, _)| candidate)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let mut costs: Vec<usize> = (0..=b_chars.len()).collect();

    for (i, a_char) in a.chars().enumerate() {
        let mut diagonal = i;
        costs[0] = i + 1;

        for (j, b_char) in b_chars.iter().enumerate() {
            let previous = costs[j + 1];
            let substitution = diagonal + usize::from(a_char != *b_char);
            let insertion = costs[j + 1] + 1;
            let deletion = costs[j] + 1;

            costs[j + 1] = substitution.min(insertion).min(deletion);
            diagonal = previous;
        }
    }

    *costs.last().unwrap_or(&0)
}

/// Prints the top-level AOXC operator help surface.
///
/// This output is intentionally ASCII-safe and icon-free in order to preserve
/// consistent rendering across SSH sessions, CI terminals, container logs,
/// and restricted console environments.
pub fn print_usage(_lang: &str) {
    println!(
        "\
AOXCMD - Operator Command Plane
Deterministic bootstrap, audit-oriented UX, hardened local flows

USAGE
  aoxc <command> [flags]
  aoxc <group> <subcommand> [flags]

GUIDED GROUPS
  chain init|create|start|status|doctor|demo
  genesis init|add-validator|add-account|build|verify|finalize|seal|sign|freeze|inspect|template-advanced|advanced-system|security-audit|fingerprint|production-gate|start
  validator create|join|register|activate|bond|unbond|inspect|status|rotate-key
  wallet create|balance
  account fund
  node init|join|start|status|doctor
  network create|join|peer-add|seed-add|bootstrap-peer-add|start|status|verify|identity-gate|doctor
  role list|status|activate-core7
  api [status|contract|smoke|metrics|health|full|chain|consensus|vm|block|tx|receipt|account|balance|state-root|network|runtime]
  query chain|consensus|vm|full|block|tx|receipt|account|balance|network|runtime|state-root|rpc
  tx transfer|stake delegate|stake undelegate
  stake delegate|undelegate|validators|rewards
  doctor [node|network|runtime]
  audit [chain|genesis|validator-set]

DESCRIBE
  version
  vision
  build-manifest
  node-connection-policy
  sovereign-core
  module-architecture
  compat-matrix
  quantum-blueprint
  quantum-posture
  port-map
  api-contract
  profile-baseline [--enforce]

BOOTSTRAP
  key-bootstrap
  keys-inspect
  keys-show-fingerprint
  keys-verify [--password <value>]
  address-create --name <validator> --profile <validation|testnet|mainnet|devnet|localnet> --password <value> [--account-id-mode <secure|legacy|dual>] [--account-id-bytes <16-32>] [--account-salt <value>]
  genesis-init
  genesis-add-account --account-id <id> --balance <amount> [--role <treasury|validator|system|user|governance|forge|quorum|seal|archive|sentinel|relay|pocket>]
  genesis-add-validator --validator-id <id> --consensus-public-key <hex> --network-public-key <hex> [--consensus-fingerprint <hex>] [--network-fingerprint <hex>] [--bootnode-address <host:port>] [--balance <amount>] [--display-name <label>]
  genesis-validate [--strict]
  genesis-finalize [--profile <localnet|devnet|validation|testnet|mainnet>] [--strict]
  genesis-seal [--profile <localnet|devnet|validation|testnet|mainnet>] [--strict]
  genesis-sign [--profile <localnet|devnet|validation|testnet|mainnet>] [--strict]
  genesis-freeze [--profile <localnet|devnet|validation|testnet|mainnet>] [--strict]
  genesis-inspect
  genesis-hash
  genesis-advanced-system [--profile <localnet|devnet|validation|testnet|mainnet>] [--out <path>] [--no-enforce]
  genesis-start [--profile <localnet|devnet|validation|testnet|mainnet>] [--no-init] [--bootstrap-key-if-missing --password <value> --operator-name <name>] [--strict] [--production-gate] [--dry-run|--skip-node-run] [--family-id <u32>] [--family-name <text>] [--family-code <text>] [--chain-id <u64>] [--chain-name <text>] [--network-class <text>] [--network-serial <text>] [--network-id <text>] [--genesis-epoch <u64>] [--block-time-ms <u64>] [--validator-quorum-policy <text>] [--consensus-identity-profile <text>] [--epoch-length-blocks <u64>] [--pacemaker-base-timeout-ms <u64>] [--pacemaker-max-timeout-ms <u64>] [--reconfiguration-finality-lag-blocks <u64>] [--native-symbol <text>] [--native-decimals <u8>] [--treasury-account-id <text>] [--treasury-amount <decimal>] [--enforce-pq-consensus] [--enforce-block-validation-rules] [--expected-genesis-sha256 <hex>] [--expected-validators-sha256 <hex>] [--expected-bootnodes-sha256 <hex>] [--expected-certificate-sha256 <hex>] [--rounds <n>] [--interval-secs <2..600>] [--tx-prefix <value>] [--continuous|--bounded] [--log-level <info|debug>] [--no-live-log] [--no-rpc-serve]
  config-init [--profile <validation|testnet|mainnet>] [--bind-host <host>] [--json-logs]
  config-validate
  config-print
  production-bootstrap --password <value> [--profile <testnet|mainnet>] [--name <validator>] [--bind-host <host>] [--produce-once-tx <value>] [--skip-produce-once]
  topology-bootstrap --mode <single|mainchain-4|devnet-4> --password <value> [--output-dir <path>] [--name-prefix <validator>] [--profile <localnet|devnet|validation|testnet|mainnet>]
  dual-profile-bootstrap --password <value> [--output-dir <path>] [--name-prefix <validator>]

NODE AND ECONOMY
  node-bootstrap
  node join --seed <multiaddr|ip:port> [--peer <id>] [--chain-id <id>] [--genesis <path>] [--profile <testnet|mainnet|...>] [--home <path>] [--trust-root <path|fingerprint>] [--allow-sync-from <peer-id>]
  produce-once
  node-run [--rounds <n>] [--continuous] [--bounded] [--interval-secs <2..600>] [--tx-prefix <value>] [--log-level <info|debug>] [--no-live-log] [--no-rpc-serve] [--no-auto-discovery] [--genesis-fingerprint <hex>] [--bootstrap-limit <1..128>] [--quantum-only] [--include-rpc]
  node-health
  economy-init
  treasury-transfer
  validator join --validator-id <id> [--name <display-name>] [--profile <validation|testnet|mainnet>] --password <value>
  validator activate --validator-id <id> [--stake <amount>]
  validator bond --validator-id <id> [--stake <amount>]
  validator unbond --validator-id <id> [--stake <amount>]
  stake-delegate
  stake-undelegate
  economy-status
  faucet-status [--account-id <id>]
  faucet-history --account-id <id>
  faucet-balance
  faucet-enable
  faucet-disable
  faucet-config-show
  faucet-audit
  faucet-config [--enable|--disable] [--max-claim-amount <n>] [--cooldown-secs <n>] [--daily-limit-per-account <n>] [--daily-global-limit <n>] [--min-reserve-balance <n>] [--ban-account <id>] [--unban-account <id>] [--allow-account <id>] [--disallow-account <id>]
  faucet-claim --account-id <id> [--amount <n>] [--auto-init] [--force]
  faucet-reset [--keep-config]
  runtime-status
  runtime-snapshot [--action <snapshot|list|prune|restore-latest>] [--keep <n>] [--runtime-root <path>] [--snapshot-dir <path>]
  runtime-snapshot-list [--runtime-root <path>] [--snapshot-dir <path>]
  runtime-snapshot-prune [--keep <n>] [--runtime-root <path>] [--snapshot-dir <path>]
  runtime-restore-latest [--runtime-root <path>] [--snapshot-dir <path>]
  chain-status
  consensus-status
  consensus-validators
  consensus-proposer
  consensus-round
  consensus-finality
  consensus-commits
  consensus-evidence
  vm-status
  query chain status
  query block --height <latest|n>
  query tx --hash <tx-hash>
  query receipt --hash <tx-hash>
  query account --id <account-id>
  query balance --id <account-id>
  query consensus status
  query consensus validators
  query consensus proposer
  query consensus round
  query consensus finality
  query consensus commits
  query consensus evidence
  query vm status
  query vm call
  query vm simulate
  query vm storage
  query vm contract
  query vm code
  query vm estimate-gas
  query vm trace
  query full [--account-id <id>] [--tx-hash <hash>]
  query network status
  query network peers [--no-auto-discovery] [--genesis-fingerprint <hex>] [--bootstrap-limit <1..128>] [--quantum-only] [--include-rpc] [--known-bootnode <node-id>] [--known-bootnode-file <path>] [--bootnodes-file <path>] [--bootnodes-sha256 <hex>] [--certificate-file <path>] [--certificate-sha256 <hex>] [--strict-bootnode-id] [--strict-security] [--require-official-peers] [--require-bootnodes-sha256] [--deny-private-peers] [--min-peer-count <1..256>]
  query network full [--no-auto-discovery] [--genesis-fingerprint <hex>] [--bootstrap-limit <1..128>] [--quantum-only] [--include-rpc] [--known-bootnode <node-id>] [--known-bootnode-file <path>] [--bootnodes-file <path>] [--bootnodes-sha256 <hex>] [--certificate-file <path>] [--certificate-sha256 <hex>] [--strict-bootnode-id] [--strict-security] [--require-official-peers] [--require-bootnodes-sha256] [--deny-private-peers] [--min-peer-count <1..256>]
  query runtime [status|snapshot] [--action <snapshot|list|prune|restore-latest>] [--keep <n>] [--runtime-root <path>] [--snapshot-dir <path>]
  query state-root
  query rpc
  api status
  api contract
  api smoke
  api metrics
  api health
  api full [--account-id <id>] [--tx-hash <hash>]
  api network full
  api runtime [status|snapshot] [--action <snapshot|list|prune|restore-latest>] [--keep <n>] [--runtime-root <path>] [--snapshot-dir <path>]
  block-get --height <latest|n>
  tx-get --hash <tx-hash>
  tx-receipt --hash <tx-hash>
  account-get --id <account-id>
  balance-get --id <account-id>
  peer-list [--no-auto-discovery] [--genesis-fingerprint <hex>] [--bootstrap-limit <1..128>] [--quantum-only] [--include-rpc] [--known-bootnode <node-id>] [--known-bootnode-file <path>] [--bootnodes-file <path>] [--bootnodes-sha256 <hex>] [--certificate-file <path>] [--certificate-sha256 <hex>] [--strict-bootnode-id] [--strict-security] [--require-official-peers] [--require-bootnodes-sha256] [--deny-private-peers] [--min-peer-count <1..256>]
  network-status
  state-root [--height <n>]
  metrics
  rpc-status
  rpc-curl-smoke
  rpc-serve [--host <ip>] [--rpc-port <n>] [--metrics-port <n>]

VALIDATION AND AUDIT
  load-benchmark
  storage-smoke
  network-smoke
  real-network
  db-init [--backend <sqlite|redb>]
  db-status [--backend <sqlite|redb>]
  db-put-block --block-file <path> [--backend <sqlite|redb>]
  db-get-height --height <n> [--backend <sqlite|redb>]
  db-get-hash --hash <hex64> [--backend <sqlite|redb>]
  db-compact [--backend <sqlite|redb>]
  diagnostics-doctor
  diagnostics-bundle
  interop-readiness
  interop-gate
  production-audit
  mainnet-readiness [--enforce] [--write-report <path>]
  testnet-readiness [--enforce] [--write-report <path>]
  full-surface-readiness [--enforce] [--write-report <path>]
  network-identity-gate [--full] [--env <name>] [--enforce]
  level-score [--enforce]
  operator-evidence-record --action <name> --reason <text> [--subject <id>] [--profile <name>] [--command <text>]
  operator-evidence-list [--limit <n>] [--category <name>]

GLOBAL FLAGS
  --home <path>               Override AOXC home directory.
  --format <text|json|yaml>   Select operator output format.
  --redact                    Redact sensitive fields where supported.

OUTPUT MODES
  text  - operator-facing curated terminal layout
  json  - machine-readable automation contract
  yaml  - human review and runbook snapshot format"
    );
}

/// Emits a serializable value according to the requested output format.
///
/// Output contract:
/// - Text: curated terminal layout intended for direct operator use.
/// - JSON: strict machine-facing structured output.
/// - YAML: review-oriented snapshot output.
///
/// The text renderer intentionally differs from JSON output. This preserves
/// a clean separation between human-facing terminal ergonomics and automation
/// contracts consumed by tooling.
pub fn emit_serialized<T: Serialize>(value: &T, format: OutputFormat) -> Result<(), AppError> {
    match format {
        OutputFormat::Text => {
            let value = serde_json::to_value(value).map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to convert text output into an intermediate value",
                    error,
                )
            })?;

            let rendered = render_text_value(&value);
            println!("{rendered}");
        }
        OutputFormat::Json => {
            let text = serde_json::to_string_pretty(value).map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to encode JSON output",
                    error,
                )
            })?;
            println!("{text}");
        }
        OutputFormat::Yaml => {
            let text = serde_yaml::to_string(value).map_err(|error| {
                AppError::with_source(
                    ErrorCode::OutputEncodingFailed,
                    "Failed to encode YAML output",
                    error,
                )
            })?;
            print!("{text}");
        }
    }

    Ok(())
}

/// Builds a stable text envelope for compact operator status commands.
///
/// The timestamp is emitted in RFC 3339 UTC format to preserve deterministic
/// interoperability across shells, CI collectors, and audit pipelines.
pub fn text_envelope(
    command: &str,
    status: &str,
    details: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut envelope = BTreeMap::new();
    envelope.insert("command".to_string(), command.to_string());
    envelope.insert("status".to_string(), status.to_string());
    envelope.insert("timestamp".to_string(), Utc::now().to_rfc3339());
    envelope.extend(details);
    envelope
}

/// Renders a serializable value into a deterministic operator-facing text view.
fn render_text_value(value: &Value) -> String {
    let mut output = String::new();
    render_text_value_into(value, 0, None, &mut output);

    while output.ends_with('\n') {
        output.pop();
    }

    output
}

/// Recursively renders a JSON value into a compact, readable text layout.
///
/// Formatting policy:
/// - Objects are printed as `key: value`.
/// - Arrays are printed as list items.
/// - Nested structures are indented with spaces only.
/// - The renderer remains ASCII-safe and icon-free.
fn render_text_value_into(value: &Value, indent: usize, key: Option<&str>, output: &mut String) {
    let padding = " ".repeat(indent);

    match value {
        Value::Object(map) => {
            if let Some(key) = key {
                output.push_str(&padding);
                output.push_str(key);
                output.push_str(":\n");
            }

            for (child_key, child_value) in map {
                match child_value {
                    Value::Object(_) | Value::Array(_) => {
                        render_text_value_into(child_value, indent + 2, Some(child_key), output);
                    }
                    _ => {
                        output.push_str(&" ".repeat(indent + 2));
                        output.push_str(child_key);
                        output.push_str(": ");
                        output.push_str(&scalar_to_text(child_value));
                        output.push('\n');
                    }
                }
            }
        }
        Value::Array(items) => {
            if let Some(key) = key {
                output.push_str(&padding);
                output.push_str(key);
                output.push_str(":\n");
            }

            for item in items {
                match item {
                    Value::Object(_) | Value::Array(_) => {
                        output.push_str(&" ".repeat(indent + 2));
                        output.push_str("-\n");
                        render_text_value_into(item, indent + 4, None, output);
                    }
                    _ => {
                        output.push_str(&" ".repeat(indent + 2));
                        output.push_str("- ");
                        output.push_str(&scalar_to_text(item));
                        output.push('\n');
                    }
                }
            }
        }
        _ => {
            if let Some(key) = key {
                output.push_str(&padding);
                output.push_str(key);
                output.push_str(": ");
                output.push_str(&scalar_to_text(value));
                output.push('\n');
            } else {
                output.push_str(&padding);
                output.push_str(&scalar_to_text(value));
                output.push('\n');
            }
        }
    }
}

/// Converts a scalar JSON value into a terminal-safe textual representation.
fn scalar_to_text(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{asks_for_help, localized_unknown_command};

    #[test]
    fn asks_for_help_detects_primary_forms() {
        assert!(asks_for_help(&["--help".to_string()]));
        assert!(asks_for_help(&["-h".to_string()]));
        assert!(asks_for_help(&["help".to_string()]));
        assert!(!asks_for_help(&["status".to_string()]));
    }

    #[test]
    fn unknown_command_error_includes_suggestion_when_close_match_exists() {
        let error = localized_unknown_command("en", "genessis");
        let message = error.to_string();
        assert!(message.contains("Did you mean 'genesis'?"));
    }
}
