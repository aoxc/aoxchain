use crate::{
    cli::{AOXC_RELEASE_NAME, TESTNET_FIXTURE_MEMBERS},
    cli_support::{arg_value, emit_serialized, has_flag, output_format, text_envelope},
    config::loader::{init_default, load, load_or_init},
    data_home::{ensure_layout, read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::{bootstrap_operator_key, operator_fingerprint, verify_operator_key},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenesisDocument {
    network_name: String,
    chain_num: u64,
    block_time_secs: u64,
    treasury: u64,
    created_at: String,
    identity_root: String,
    validators: Vec<GenesisValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenesisValidator {
    name: String,
    public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestnetFixtureManifest {
    profile: String,
    release_name: String,
    network_name: String,
    chain_num: u64,
    block_time_secs: u64,
    fund_amount: u128,
    created_at: String,
    prefix_policy: PrefixPolicy,
    validators: Vec<TestnetAccountRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrefixPolicy {
    network_prefix: String,
    key_prefix: String,
    node_home_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestnetAccountRecord {
    name: String,
    label: String,
    validator_index: u16,
    p2p_port: u16,
    rpc_port: u16,
    metrics_port: u16,
    seed_alias: String,
    seed_hex: String,
    account_id: String,
    validator_id: String,
    key_name: String,
    home_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestnetGenesisDocument {
    network_name: String,
    chain_num: u64,
    block_time_secs: u64,
    created_at: String,
    treasury: String,
    validators: Vec<TestnetGenesisValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestnetGenesisValidator {
    name: String,
    validator_id: String,
    public_key: String,
    funded_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixtureNodeSettings {
    profile: String,
    chain_num: u64,
    bind_host: String,
    p2p_port: u16,
    rpc_port: u16,
    metrics_port: u16,
    tx_prefix: String,
    key_name: String,
    account_id: String,
    validator_id: String,
    seed_file: String,
}

fn genesis_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("identity").join("genesis.json"))
}

pub fn cmd_testnet_fixture_init(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    let fixture_dir = fixture_output_dir(&home, args);
    let chain_num = parse_u64_flag(args, "--chain-num", 77)?;
    let block_time_secs = parse_u64_flag(args, "--block-time", 6)?;
    let fund_amount = parse_u128_flag(args, "--fund-amount", 2_500_000_000_000_000_000_000_u128)?;
    let created_at = Utc::now().to_rfc3339();
    let network_name = format!("AOXC Testnet {chain_num}");
    let validators = build_fixture_accounts(&fixture_dir)?;

    create_fixture_layout(&fixture_dir)?;
    write_fixture_accounts(&fixture_dir, &validators)?;
    write_fixture_genesis(
        &fixture_dir,
        &network_name,
        chain_num,
        block_time_secs,
        fund_amount,
        &created_at,
        &validators,
    )?;
    write_fixture_nodes(&fixture_dir, chain_num, &validators)?;
    write_fixture_hosts(&fixture_dir, &validators)?;
    write_fixture_launch_script(&fixture_dir, &validators)?;
    write_fixture_readme(
        &fixture_dir,
        &network_name,
        chain_num,
        block_time_secs,
        fund_amount,
    )?;
    write_fixture_manifest(
        &fixture_dir,
        &network_name,
        chain_num,
        block_time_secs,
        fund_amount,
        &created_at,
        &validators,
    )?;

    let mut details = BTreeMap::new();
    details.insert("fixture_dir".to_string(), fixture_dir.display().to_string());
    details.insert("chain_num".to_string(), chain_num.to_string());
    details.insert("fund_amount".to_string(), fund_amount.to_string());
    details.insert(
        "members".to_string(),
        TESTNET_FIXTURE_MEMBERS.len().to_string(),
    );
    emit_serialized(
        &text_envelope("testnet-fixture-init", "ok", details),
        output_format(args),
    )
}

pub fn cmd_key_bootstrap(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "validator".to_string());
    let name = arg_value(args, "--name").unwrap_or_else(|| "validator-01".to_string());
    let password = arg_value(args, "--password").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --password for key bootstrap",
        )
    })?;
    let material = bootstrap_operator_key(&name, &profile, &password)?;
    emit_serialized(&material, output_format(args))
}

pub fn cmd_keys_show_fingerprint(args: &[String]) -> Result<(), AppError> {
    let fp = operator_fingerprint()?;
    let mut details = BTreeMap::new();
    details.insert("fingerprint".to_string(), fp);
    emit_serialized(
        &text_envelope("keys-show-fingerprint", "ok", details),
        output_format(args),
    )
}

pub fn cmd_keys_verify(args: &[String]) -> Result<(), AppError> {
    let password = arg_value(args, "--password");
    verify_operator_key(password.as_deref())?;
    let mut details = BTreeMap::new();
    details.insert("result".to_string(), "verified".to_string());
    details.insert(
        "decrypt_check".to_string(),
        if password.is_some() {
            "passed".to_string()
        } else {
            "skipped-no-password".to_string()
        },
    );
    emit_serialized(
        &text_envelope("keys-verify", "ok", details),
        output_format(args),
    )
}

pub fn cmd_genesis_init(args: &[String]) -> Result<(), AppError> {
    let chain_num = arg_value(args, "--chain-num")
        .unwrap_or_else(|| "1001".to_string())
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Invalid --chain-num value",
            )
        })?;
    let block_time_secs = arg_value(args, "--block-time")
        .unwrap_or_else(|| "6".to_string())
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Invalid --block-time value",
            )
        })?;
    let treasury = arg_value(args, "--treasury")
        .unwrap_or_else(|| "1000000000000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --treasury value"))?;
    let validator_key = operator_fingerprint().unwrap_or_else(|_| "unbootstrapped".to_string());

    let genesis = GenesisDocument {
        network_name: "AOXC Local Genesis".to_string(),
        chain_num,
        block_time_secs,
        treasury,
        created_at: Utc::now().to_rfc3339(),
        identity_root: format!("aoxc-root-{chain_num}"),
        validators: vec![GenesisValidator {
            name: "local-operator".to_string(),
            public_key: validator_key,
        }],
    };
    let content = serde_json::to_string_pretty(&genesis).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode genesis document",
            e,
        )
    })?;
    write_file(&genesis_path()?, &content)?;
    emit_serialized(&genesis, output_format(args))
}

pub fn cmd_genesis_validate(args: &[String]) -> Result<(), AppError> {
    let genesis = load_genesis()?;
    validate_genesis(&genesis)?;
    let mut details = BTreeMap::new();
    details.insert("chain_num".to_string(), genesis.chain_num.to_string());
    details.insert(
        "validators".to_string(),
        genesis.validators.len().to_string(),
    );
    emit_serialized(
        &text_envelope("genesis-validate", "ok", details),
        output_format(args),
    )
}

pub fn cmd_genesis_inspect(args: &[String]) -> Result<(), AppError> {
    let genesis = load_genesis()?;
    emit_serialized(&genesis, output_format(args))
}

pub fn cmd_genesis_hash(args: &[String]) -> Result<(), AppError> {
    let raw = read_file(&genesis_path()?)?;
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let digest = hex::encode(hasher.finalize());
    let mut details = BTreeMap::new();
    details.insert("sha256".to_string(), digest);
    emit_serialized(
        &text_envelope("genesis-hash", "ok", details),
        output_format(args),
    )
}

pub fn cmd_config_init(args: &[String]) -> Result<(), AppError> {
    let settings = init_default()?;
    emit_serialized(&settings, output_format(args))
}

pub fn cmd_config_validate(args: &[String]) -> Result<(), AppError> {
    let settings = load_or_init()?;
    settings
        .validate()
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;
    let mut details = BTreeMap::new();
    details.insert("profile".to_string(), settings.profile);
    details.insert("result".to_string(), "valid".to_string());
    emit_serialized(
        &text_envelope("config-validate", "ok", details),
        output_format(args),
    )
}

pub fn cmd_config_print(args: &[String]) -> Result<(), AppError> {
    let settings = load()?;
    let printable = if has_flag(args, "--redact") {
        settings.redacted()
    } else {
        settings
    };
    emit_serialized(&printable, output_format(args))
}

pub(crate) fn genesis_ready() -> bool {
    load_genesis().is_ok()
}

fn fixture_output_dir(home: &Path, args: &[String]) -> PathBuf {
    arg_value(args, "--output-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("support").join("deterministic-testnet"))
}

fn parse_u64_flag(args: &[String], flag: &str, default: u64) -> Result<u64, AppError> {
    arg_value(args, flag)
        .unwrap_or_else(|| default.to_string())
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Invalid {flag} value"),
            )
        })
}

fn parse_u128_flag(args: &[String], flag: &str, default: u128) -> Result<u128, AppError> {
    arg_value(args, flag)
        .unwrap_or_else(|| default.to_string())
        .parse::<u128>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Invalid {flag} value"),
            )
        })
}

fn create_fixture_layout(fixture_dir: &Path) -> Result<(), AppError> {
    for relative in ["nodes", "homes"] {
        std::fs::create_dir_all(fixture_dir.join(relative)).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create fixture support directory {}",
                    fixture_dir.join(relative).display()
                ),
                error,
            )
        })?;
    }
    Ok(())
}

fn build_fixture_accounts(fixture_dir: &Path) -> Result<Vec<TestnetAccountRecord>, AppError> {
    TESTNET_FIXTURE_MEMBERS
        .iter()
        .map(
            |(name, label, p2p_port, rpc_port, validator_index, seed_alias)| {
                let metrics_port = rpc_port.checked_add(7).ok_or_else(|| {
                    AppError::new(
                        ErrorCode::UsageInvalidArguments,
                        "Metrics port derivation overflowed",
                    )
                })?;
                let seed_hex = derive_fixture_seed_hex(seed_alias);
                let account_id = format!("testnet-{name}-acct");
                let validator_id = format!("testnet-{name}-validator");
                let key_name = format!("testnet-{name}-key");
                let home_dir = fixture_dir.join("homes").join(name).display().to_string();

                Ok(TestnetAccountRecord {
                    name: (*name).to_string(),
                    label: (*label).to_string(),
                    validator_index: *validator_index,
                    p2p_port: *p2p_port,
                    rpc_port: *rpc_port,
                    metrics_port,
                    seed_alias: (*seed_alias).to_string(),
                    seed_hex,
                    account_id,
                    validator_id,
                    key_name,
                    home_dir,
                })
            },
        )
        .collect()
}

fn derive_fixture_seed_hex(seed_alias: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC/TESTNET/FIXTURE/SEED/V1/");
    hasher.update(seed_alias.as_bytes());
    hex::encode(hasher.finalize())
}

fn write_fixture_accounts(
    fixture_dir: &Path,
    validators: &[TestnetAccountRecord],
) -> Result<(), AppError> {
    let payload = serde_json::to_string_pretty(validators).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode deterministic testnet accounts",
            error,
        )
    })?;
    write_file(&fixture_dir.join("accounts.json"), &payload)?;

    for validator in validators {
        let seed_path = fixture_dir
            .join("homes")
            .join(&validator.name)
            .join("identity")
            .join("test-node-seed.hex");
        write_file(&seed_path, &validator.seed_hex)?;
    }
    Ok(())
}

fn write_fixture_genesis(
    fixture_dir: &Path,
    network_name: &str,
    chain_num: u64,
    block_time_secs: u64,
    fund_amount: u128,
    created_at: &str,
    validators: &[TestnetAccountRecord],
) -> Result<(), AppError> {
    let treasury = validators
        .iter()
        .try_fold(0_u128, |accumulator, _| {
            accumulator.checked_add(fund_amount)
        })
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::GenesisInvalid,
                "Deterministic testnet treasury overflowed while summing validator allocations",
            )
        })?;

    let payload = TestnetGenesisDocument {
        network_name: network_name.to_string(),
        chain_num,
        block_time_secs,
        created_at: created_at.to_string(),
        treasury: treasury.to_string(),
        validators: validators
            .iter()
            .map(|validator| TestnetGenesisValidator {
                name: validator.name.clone(),
                validator_id: validator.validator_id.clone(),
                public_key: validator.seed_hex.clone(),
                funded_amount: fund_amount.to_string(),
            })
            .collect(),
    };
    let encoded = serde_json::to_string_pretty(&payload).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode deterministic testnet genesis document",
            error,
        )
    })?;
    write_file(&fixture_dir.join("genesis.json"), &encoded)
}

fn write_fixture_nodes(
    fixture_dir: &Path,
    chain_num: u64,
    validators: &[TestnetAccountRecord],
) -> Result<(), AppError> {
    for validator in validators {
        let node = FixtureNodeSettings {
            profile: "testnet".to_string(),
            chain_num,
            bind_host: "127.0.0.1".to_string(),
            p2p_port: validator.p2p_port,
            rpc_port: validator.rpc_port,
            metrics_port: validator.metrics_port,
            tx_prefix: format!("TESTNET-{}", validator.name.to_ascii_uppercase()),
            key_name: validator.key_name.clone(),
            account_id: validator.account_id.clone(),
            validator_id: validator.validator_id.clone(),
            seed_file: format!("homes/{}/identity/test-node-seed.hex", validator.name),
        };
        let encoded = render_fixture_node_settings(&node);
        write_file(
            &fixture_dir
                .join("nodes")
                .join(format!("{}.toml", validator.name)),
            &encoded,
        )?;
    }
    Ok(())
}

fn write_fixture_hosts(
    fixture_dir: &Path,
    validators: &[TestnetAccountRecord],
) -> Result<(), AppError> {
    let hosts = validators
        .iter()
        .map(|validator| format!("{} 127.0.0.1:{}", validator.name, validator.p2p_port))
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!("{hosts}\n");
    write_file(&fixture_dir.join("hosts.txt"), &content)
}

fn write_fixture_launch_script(
    fixture_dir: &Path,
    validators: &[TestnetAccountRecord],
) -> Result<(), AppError> {
    let mut script = String::from(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nROOT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nAOXC_BIN=\"${AOXC_BIN:-cargo run -q -p aoxcmd --}\"\nROUNDS=\"${ROUNDS:-4}\"\nSLEEP_MS=\"${SLEEP_MS:-25}\"\n\n",
    );
    for validator in validators {
        script.push_str(&format!(
            "$AOXC_BIN node-run --home \"$ROOT_DIR/homes/{name}\" --rounds \"$ROUNDS\" --sleep-ms \"$SLEEP_MS\" --tx-prefix \"TESTNET-{upper}\" >/tmp/aoxc-{name}-run.json\n",
            name = validator.name,
            upper = validator.name.to_ascii_uppercase()
        ));
    }
    write_file(&fixture_dir.join("launch-testnet.sh"), &script)
}

fn write_fixture_readme(
    fixture_dir: &Path,
    network_name: &str,
    chain_num: u64,
    block_time_secs: u64,
    fund_amount: u128,
) -> Result<(), AppError> {
    let content = format!(
        "# Deterministic AOXC Testnet Fixture\n\n\
Network name: `{network_name}`\n\n\
Chain number: `{chain_num}`\n\n\
Block time: `{block_time_secs}` seconds\n\n\
Fund amount per validator: `{fund_amount}`\n\n\
This fixture is designed to stay operationally close to the production path while preserving explicit `testnet-` naming on identities and key labels.\n"
    );
    write_file(&fixture_dir.join("README.md"), &content)
}

fn write_fixture_manifest(
    fixture_dir: &Path,
    network_name: &str,
    chain_num: u64,
    block_time_secs: u64,
    fund_amount: u128,
    created_at: &str,
    validators: &[TestnetAccountRecord],
) -> Result<(), AppError> {
    let payload = TestnetFixtureManifest {
        profile: "deterministic-testnet".to_string(),
        release_name: AOXC_RELEASE_NAME.to_string(),
        network_name: network_name.to_string(),
        chain_num,
        block_time_secs,
        fund_amount,
        created_at: created_at.to_string(),
        prefix_policy: PrefixPolicy {
            network_prefix: "testnet".to_string(),
            key_prefix: "testnet-".to_string(),
            node_home_prefix: "testnet-home-".to_string(),
        },
        validators: validators.to_vec(),
    };
    let encoded = serde_json::to_string_pretty(&payload).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode deterministic testnet manifest",
            error,
        )
    })?;
    write_file(&fixture_dir.join("manifest.json"), &encoded)
}

fn render_fixture_node_settings(node: &FixtureNodeSettings) -> String {
    format!(
        "profile = \"{profile}\"\nchain_num = {chain_num}\nbind_host = \"{bind_host}\"\np2p_port = {p2p_port}\nrpc_port = {rpc_port}\nmetrics_port = {metrics_port}\ntx_prefix = \"{tx_prefix}\"\nkey_name = \"{key_name}\"\naccount_id = \"{account_id}\"\nvalidator_id = \"{validator_id}\"\nseed_file = \"{seed_file}\"\n",
        profile = node.profile,
        chain_num = node.chain_num,
        bind_host = node.bind_host,
        p2p_port = node.p2p_port,
        rpc_port = node.rpc_port,
        metrics_port = node.metrics_port,
        tx_prefix = node.tx_prefix,
        key_name = node.key_name,
        account_id = node.account_id,
        validator_id = node.validator_id,
        seed_file = node.seed_file,
    )
}

fn load_genesis() -> Result<GenesisDocument, AppError> {
    let raw = read_file(&genesis_path()?).map_err(|_| {
        AppError::new(
            ErrorCode::GenesisInvalid,
            "Genesis document is missing. Execute genesis-init before validation.",
        )
    })?;
    let genesis: GenesisDocument = serde_json::from_str(&raw).map_err(|e| {
        AppError::with_source(
            ErrorCode::GenesisInvalid,
            "Failed to parse genesis document",
            e,
        )
    })?;
    validate_genesis(&genesis)?;
    Ok(genesis)
}

fn validate_genesis(genesis: &GenesisDocument) -> Result<(), AppError> {
    if genesis.chain_num == 0 || genesis.block_time_secs == 0 || genesis.treasury == 0 {
        return Err(AppError::new(
            ErrorCode::GenesisInvalid,
            "Genesis document failed non-zero value validation",
        ));
    }
    if genesis.validators.is_empty() {
        return Err(AppError::new(
            ErrorCode::GenesisInvalid,
            "Genesis document must contain at least one validator",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::cmd_testnet_fixture_init;
    use crate::data_home::read_file;
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(label: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let base = env::temp_dir().join(format!("aoxc-{label}-{nanos}"));
        fs::create_dir_all(&base)?;
        Ok(base)
    }

    #[test]
    fn testnet_fixture_init_writes_main_artifacts() -> Result<(), Box<dyn std::error::Error>> {
        let output_dir = unique_temp_dir("fixture-artifacts")?;
        let args = vec![
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--chain-num".to_string(),
            "177".to_string(),
            "--fund-amount".to_string(),
            "123456".to_string(),
        ];

        cmd_testnet_fixture_init(&args)?;

        let accounts = read_file(&output_dir.join("accounts.json"))?;
        let genesis = read_file(&output_dir.join("genesis.json"))?;
        let manifest = read_file(&output_dir.join("manifest.json"))?;
        let node = read_file(&output_dir.join("nodes").join("atlas.toml"))?;
        let seed = read_file(
            &output_dir
                .join("homes")
                .join("atlas")
                .join("identity")
                .join("test-node-seed.hex"),
        )?;

        assert!(accounts.contains("testnet-atlas-acct"));
        assert!(genesis.contains("\"chain_num\": 177"));
        assert!(manifest.contains("\"network_prefix\": \"testnet\""));
        assert!(node.contains("tx_prefix = \"TESTNET-ATLAS\""));
        assert_eq!(seed.len(), 64);

        fs::remove_dir_all(output_dir)?;
        Ok(())
    }
}
