use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliLanguage {
    En,
    Tr,
    Es,
    De,
}

impl CliLanguage {
    fn from_code(input: &str) -> Self {
        match input.trim().to_ascii_lowercase().as_str() {
            "tr" | "tr-tr" | "turkish" | "türkçe" => Self::Tr,
            "es" | "es-es" | "spanish" | "español" => Self::Es,
            "de" | "de-de" | "german" | "deutsch" => Self::De,
            _ => Self::En,
        }
    }
}

pub(crate) fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find_map(|window| {
        if window[0] == key {
            Some(window[1].clone())
        } else {
            None
        }
    })
}

pub(crate) fn arg_bool_value(args: &[String], key: &str) -> Option<bool> {
    arg_value(args, key).map(|raw| {
        let normalized = raw.trim().to_ascii_lowercase();
        matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
    })
}

pub(crate) fn arg_flag(args: &[String], key: &str) -> bool {
    args.iter().any(|arg| arg == key)
}

pub(crate) fn detect_language(args: &[String]) -> CliLanguage {
    if let Some(explicit) = arg_value(args, "--lang") {
        return CliLanguage::from_code(&explicit);
    }

    let from_env = env::var("AOXC_LANG").unwrap_or_else(|_| "en".to_string());
    CliLanguage::from_code(&from_env)
}

pub(crate) fn localized_unknown_command(lang: CliLanguage, command: &str) -> String {
    match lang {
        CliLanguage::Tr => format!("bilinmeyen komut: {command}"),
        CliLanguage::Es => format!("comando desconocido: {command}"),
        CliLanguage::De => format!("unbekannter befehl: {command}"),
        CliLanguage::En => format!("unknown command: {command}"),
    }
}

pub(crate) fn print_usage(lang: CliLanguage) {
    println!("{}", usage_text(lang));
}

pub(crate) fn usage_text(lang: CliLanguage) -> &'static str {
    match lang {
        CliLanguage::Tr => {
            "AOXC Komut Yüzeyi

Komutlar:
  vision
  build-manifest
  node-connection-policy [--enforce-official]
  sovereign-core
  module-architecture
  compat-matrix
  port-map
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>] [--native-symbol <SYMBOL>] [--native-decimals <u8>] [--settlement-network <name>] [--xlayer-token <0x...>] [--xlayer-main-contract <0x...>] [--xlayer-multisig <0x...>] [--equivalence-mode <text>]
  node-bootstrap
  produce-once [--tx <payload>]
  node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
  network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  network-smoke [--timeout-ms <u64>] [--payload <text>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (veya AOXC_LANG ortam değişkeni)
  --home <dir> (varsayılan: $HOME/.AOXC-Data, veya AOXC_HOME)
"
        }
        CliLanguage::Es => {
            "Superficie de Comandos AOXC

Comandos:
  vision
  build-manifest
  node-connection-policy [--enforce-official]
  sovereign-core
  module-architecture
  compat-matrix
  port-map
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>] [--native-symbol <SYMBOL>] [--native-decimals <u8>] [--settlement-network <name>] [--xlayer-token <0x...>] [--xlayer-main-contract <0x...>] [--xlayer-multisig <0x...>] [--equivalence-mode <text>]
  node-bootstrap
  produce-once [--tx <payload>]
  node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
  network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  network-smoke [--timeout-ms <u64>] [--payload <text>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (o variable AOXC_LANG)
  --home <dir> (por defecto: $HOME/.AOXC-Data, o AOXC_HOME)
"
        }
        CliLanguage::De => {
            "AOXC Kommandooberfläche

Befehle:
  vision
  build-manifest
  node-connection-policy [--enforce-official]
  sovereign-core
  module-architecture
  compat-matrix
  port-map
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>] [--native-symbol <SYMBOL>] [--native-decimals <u8>] [--settlement-network <name>] [--xlayer-token <0x...>] [--xlayer-main-contract <0x...>] [--xlayer-multisig <0x...>] [--equivalence-mode <text>]
  node-bootstrap
  produce-once [--tx <payload>]
  node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
  network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  network-smoke [--timeout-ms <u64>] [--payload <text>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (oder AOXC_LANG Umgebungsvariable)
  --home <dir> (Standard: $HOME/.AOXC-Data oder AOXC_HOME)
"
        }
        CliLanguage::En => {
            "AOXC Command Surface

Commands:
  vision
  build-manifest
  node-connection-policy [--enforce-official]
  sovereign-core

  module-architecture
  compat-matrix
  port-map
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>] [--native-symbol <SYMBOL>] [--native-decimals <u8>] [--settlement-network <name>] [--xlayer-token <0x...>] [--xlayer-main-contract <0x...>] [--xlayer-multisig <0x...>] [--equivalence-mode <text>]
  node-bootstrap
  produce-once [--tx <payload>]
  node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
  network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
  network-smoke [--timeout-ms <u64>] [--payload <text>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (or AOXC_LANG environment variable)
  --home <dir> (default: $HOME/.AOXC-Data, or AOXC_HOME)
"
        }
    }
}
