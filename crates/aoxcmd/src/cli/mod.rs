// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, detect_language, localized_unknown_command, print_usage},
    error::AppError,
};
use std::{env, ffi::OsString};

pub(crate) mod audit;
pub(crate) mod bootstrap;
pub(crate) mod db;
pub(crate) mod describe;
pub(crate) mod evidence;
pub(crate) mod ops;

pub(crate) const AOXC_RELEASE_NAME: &str = "AOXC Mainnet-Candidate Operator Plane";
pub(crate) const TESTNET_FIXTURE_MEMBERS: [(&str, &str, u16, u16, u16, &str); 5] = [
    ("atlas", "Atlas Validator", 39001, 19101, 1, "atlas-seed"),
    ("boreal", "Boreal Validator", 39002, 19102, 2, "boreal-seed"),
    ("cypher", "Cypher Validator", 39003, 19103, 3, "cypher-seed"),
    ("delta", "Delta Validator", 39004, 19104, 4, "delta-seed"),
    ("ember", "Ember Validator", 39005, 19105, 5, "ember-seed"),
];

/// Process-scoped guard that installs an optional `AOXC_HOME` override and
/// restores the prior environment state when command dispatch exits.
///
/// Security and correctness rationale:
/// - `AOXC_HOME` is process-global mutable state.
/// - A command-scoped override must never leak into later commands, tests,
///   or unrelated execution paths that share the same process.
/// - Restoration must occur on every exit path, including early returns
///   and command failures.
struct CliHomeOverrideGuard {
    previous_home: Option<OsString>,
    override_applied: bool,
}

impl CliHomeOverrideGuard {
    /// Installs a scoped `AOXC_HOME` override derived from `--home`, when present.
    ///
    /// Behavioral contract:
    /// - Captures the pre-existing `AOXC_HOME` value before mutation.
    /// - Applies the override only when `--home` is present and non-blank.
    /// - Leaves the process environment untouched when no explicit override exists.
    fn install(args: &[String]) -> Self {
        let previous_home = env::var_os("AOXC_HOME");

        match arg_value(args, "--home") {
            Some(home) if !home.trim().is_empty() => {
                env::set_var("AOXC_HOME", home);
                Self {
                    previous_home,
                    override_applied: true,
                }
            }
            _ => Self {
                previous_home,
                override_applied: false,
            },
        }
    }
}

impl Drop for CliHomeOverrideGuard {
    fn drop(&mut self) {
        if !self.override_applied {
            return;
        }

        match self.previous_home.take() {
            Some(previous_home) => env::set_var("AOXC_HOME", previous_home),
            None => env::remove_var("AOXC_HOME"),
        }
    }
}

/// Executes the AOXC operator CLI command surface.
///
/// Operational objectives:
/// - Preserve deterministic command routing from the raw process argument vector.
/// - Detect language preference before usage or unknown-command handling.
/// - Scope `--home` overrides strictly to the current CLI execution.
pub fn run_cli() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();
    let lang = detect_language(&args[1..]);

    // The guard intentionally remains alive for the full dispatch scope so
    // every downstream module resolves a consistent effective AOXC home while
    // still guaranteeing restoration on every exit path.
    let _home_override_guard = CliHomeOverrideGuard::install(&args[1..]);

    if args.len() < 2 {
        print_usage(lang);
        return Ok(());
    }

    match args[1].as_str() {
        "version" | "--version" | "-V" => describe::cmd_version(),
        "help" | "--help" | "-h" => {
            print_usage(lang);
            Ok(())
        }
        "vision" => describe::cmd_vision(),
        "build-manifest" => describe::cmd_build_manifest(),
        "node-connection-policy" => describe::cmd_node_connection_policy(&args[2..]),
        "sovereign-core" => describe::cmd_sovereign_core(),
        "module-architecture" => describe::cmd_module_architecture(),
        "compat-matrix" => describe::cmd_compat_matrix(),
        "port-map" => describe::cmd_port_map(),
        "profile-baseline" => ops::cmd_profile_baseline(&args[2..]),
        "testnet-fixture-init" => bootstrap::cmd_testnet_fixture_init(&args[2..]),
        "load-benchmark" => ops::cmd_load_benchmark(&args[2..]),
        "mainnet-readiness" => ops::cmd_mainnet_readiness(&args[2..]),
        "testnet-readiness" => ops::cmd_testnet_readiness(&args[2..]),
        "full-surface-readiness" => ops::cmd_full_surface_readiness(&args[2..]),
        "level-score" => ops::cmd_level_score(&args[2..]),
        "operator-evidence-record" => evidence::cmd_operator_evidence_record(&args[2..]),
        "operator-evidence-list" => evidence::cmd_operator_evidence_list(&args[2..]),
        "key-bootstrap" => bootstrap::cmd_key_bootstrap(&args[2..]),
        "keys-inspect" => bootstrap::cmd_keys_inspect(&args[2..]),
        "keys-show-fingerprint" => bootstrap::cmd_keys_show_fingerprint(&args[2..]),
        "keys-verify" => bootstrap::cmd_keys_verify(&args[2..]),
        "genesis-init" => bootstrap::cmd_genesis_init(&args[2..]),
        "genesis-validate" => bootstrap::cmd_genesis_validate(&args[2..]),
        "genesis-inspect" => bootstrap::cmd_genesis_inspect(&args[2..]),
        "genesis-hash" => bootstrap::cmd_genesis_hash(&args[2..]),
        "config-init" => bootstrap::cmd_config_init(&args[2..]),
        "config-validate" => bootstrap::cmd_config_validate(&args[2..]),
        "config-print" => bootstrap::cmd_config_print(&args[2..]),
        "production-bootstrap" => bootstrap::cmd_production_bootstrap(&args[2..]),
        "dual-profile-bootstrap" => bootstrap::cmd_dual_profile_bootstrap(&args[2..]),
        "node-bootstrap" => ops::cmd_node_bootstrap(&args[2..]),
        "produce-once" => ops::cmd_produce_once(&args[2..]),
        "node-run" => ops::cmd_node_run(&args[2..]),
        "node-health" => ops::cmd_node_health(&args[2..]),
        "network-smoke" => ops::cmd_network_smoke(&args[2..]),
        "real-network" => ops::cmd_real_network(&args[2..]),
        "storage-smoke" => ops::cmd_storage_smoke(&args[2..]),
        "db-init" => db::cmd_db_init(&args[2..]),
        "db-status" => db::cmd_db_status(&args[2..]),
        "db-put-block" => db::cmd_db_put_block(&args[2..]),
        "db-get-height" => db::cmd_db_get_height(&args[2..]),
        "db-get-hash" => db::cmd_db_get_hash(&args[2..]),
        "db-compact" => db::cmd_db_compact(&args[2..]),
        "economy-init" => ops::cmd_economy_init(&args[2..]),
        "treasury-transfer" => ops::cmd_treasury_transfer(&args[2..]),
        "stake-delegate" => ops::cmd_stake_delegate(&args[2..]),
        "stake-undelegate" => ops::cmd_stake_undelegate(&args[2..]),
        "economy-status" => ops::cmd_economy_status(&args[2..]),
        "runtime-status" => ops::cmd_runtime_status(&args[2..]),
        "diagnostics-doctor" => audit::cmd_diagnostics_doctor(&args[2..]),
        "diagnostics-bundle" => audit::cmd_diagnostics_bundle(&args[2..]),
        "interop-readiness" => audit::cmd_interop_readiness(&args[2..]),
        "interop-gate" => audit::cmd_interop_gate(&args[2..]),
        "production-audit" => audit::cmd_production_audit(&args[2..]),
        other => Err(localized_unknown_command(lang, other)),
    }
}

#[cfg(test)]
mod tests {
    use super::CliHomeOverrideGuard;
    use std::{
        env,
        ffi::OsString,
        sync::{Mutex, MutexGuard, OnceLock},
    };

    /// Serializes tests that mutate process-global environment state.
    ///
    /// Rationale:
    /// - Environment variables are shared across the entire process.
    /// - Tests that mutate `AOXC_HOME` must not race each other.
    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn cli_args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    /// Restores `AOXC_HOME` to its pre-test state.
    fn restore_aoxc_home(previous: Option<OsString>) {
        match previous {
            Some(value) => env::set_var("AOXC_HOME", value),
            None => env::remove_var("AOXC_HOME"),
        }
    }

    /// Returns the current `AOXC_HOME` in an OS-string-safe form.
    fn current_aoxc_home() -> Option<OsString> {
        env::var_os("AOXC_HOME")
    }

    #[test]
    fn home_override_guard_applies_override_for_scope_and_restores_previous_value() {
        let _lock = env_lock();
        let previous = current_aoxc_home();

        env::set_var("AOXC_HOME", "/tmp/aoxc-original-home");

        {
            let _guard =
                CliHomeOverrideGuard::install(&cli_args(&["--home", "/tmp/aoxc-temporary-home"]));

            assert_eq!(
                current_aoxc_home(),
                Some(OsString::from("/tmp/aoxc-temporary-home"))
            );
        }

        assert_eq!(
            current_aoxc_home(),
            Some(OsString::from("/tmp/aoxc-original-home"))
        );

        restore_aoxc_home(previous);
    }

    #[test]
    fn home_override_guard_removes_temporary_value_when_no_previous_value_existed() {
        let _lock = env_lock();
        let previous = current_aoxc_home();

        env::remove_var("AOXC_HOME");

        {
            let _guard =
                CliHomeOverrideGuard::install(&cli_args(&["--home", "/tmp/aoxc-ephemeral-home"]));

            assert_eq!(
                current_aoxc_home(),
                Some(OsString::from("/tmp/aoxc-ephemeral-home"))
            );
        }

        assert_eq!(current_aoxc_home(), None);

        restore_aoxc_home(previous);
    }

    #[test]
    fn home_override_guard_does_not_mutate_environment_when_home_flag_is_absent() {
        let _lock = env_lock();
        let previous = current_aoxc_home();

        env::set_var("AOXC_HOME", "/tmp/aoxc-stable-home");

        {
            let _guard = CliHomeOverrideGuard::install(&cli_args(&["db-status"]));

            assert_eq!(
                current_aoxc_home(),
                Some(OsString::from("/tmp/aoxc-stable-home"))
            );
        }

        assert_eq!(
            current_aoxc_home(),
            Some(OsString::from("/tmp/aoxc-stable-home"))
        );

        restore_aoxc_home(previous);
    }

    #[test]
    fn home_override_guard_ignores_blank_home_values() {
        let _lock = env_lock();
        let previous = current_aoxc_home();

        env::set_var("AOXC_HOME", "/tmp/aoxc-existing-home");
        let expected = current_aoxc_home();

        {
            let _guard = CliHomeOverrideGuard::install(&cli_args(&["--home", "   "]));

            assert_eq!(current_aoxc_home(), expected);
        }

        assert_eq!(current_aoxc_home(), expected);

        restore_aoxc_home(previous);
    }
}
