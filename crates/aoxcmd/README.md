# AOXCMD

AOXCMD is the audit-oriented operator command plane for AOXChain-style local node bootstrap, identity material handling, ledger initialization, diagnostics, and readiness reporting.

## Design posture

This package is designed for:

- deterministic local bootstrap,
- reproducible operator workflows,
- explicit filesystem layout,
- machine-readable audit output,
- conservative failure handling.

## Default home directory

Unless overridden by `--home` or `AOXC_HOME`, AOXCMD uses:

```text
$HOME/.aoxc-data
```

## Core command groups

### Describe / policy / manifest

```bash
cargo run -- version
cargo run -- vision
cargo run -- build-manifest
cargo run -- node-connection-policy
cargo run -- sovereign-core
cargo run -- module-architecture
cargo run -- compat-matrix
cargo run -- port-map
```

### Bootstrap / identity / genesis

```bash
cargo run -- key-bootstrap --name validator-01 --password "Example#2026!"
cargo run -- keys-verify --password "Example#2026!"
cargo run -- genesis-init --chain-num 1001 --treasury 1000000000000
cargo run -- genesis-validate
cargo run -- genesis-hash
cargo run -- keys-show-fingerprint
```

### Node / runtime / economy

```bash
cargo run -- node-bootstrap
cargo run -- produce-once --tx boot-sequence-1
cargo run -- node-run --rounds 5 --tx-prefix AOXC
cargo run -- node-health
cargo run -- economy-init
cargo run -- treasury-transfer --to ops --amount 1000
cargo run -- economy-status
cargo run -- runtime-status
```

### Diagnostics / audit

```bash
cargo run -- diagnostics-doctor
cargo run -- diagnostics-bundle
cargo run -- interop-readiness
cargo run -- interop-gate --enforce-official
cargo run -- production-audit
cargo run -- mainnet-readiness
```

### Config

```bash
cargo run -- config-init
cargo run -- config-validate
cargo run -- config-print --redact
```

## Filesystem layout

The command plane provisions the following structure:

```text
~/.aoxc-data/
  config/
    settings.json
  identity/
    genesis.json
    node_identity.json
  keys/
    operator_key.json
  ledger/
    ledger.json
  runtime/
    node_state.json
  telemetry/
    metrics.json
  reports/
  support/
```

## Output modes

Most commands support:

```text
--format text
--format json
--format yaml
```

## Key custody note

`key-bootstrap` now stores operator secret material inside an encrypted seed envelope.
Use password-backed verification when you want a deeper integrity check:

```bash
cargo run -- keys-verify --password "Example#2026!"
```

## Operational note

No software package should be treated as “guaranteed error-free.” This package is written with audit-oriented discipline, but all production promotion decisions should still be gated by review, integration testing, and environment-specific verification.
