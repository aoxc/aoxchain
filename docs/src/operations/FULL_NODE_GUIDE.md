# AOXChain Full Node Installation and Join Guide

This guide provides a deterministic operator path to install, bootstrap, validate, and run an AOXChain full node.

## 1. Preconditions

- Rust toolchain installed (stable channel).
- Git and build-essential toolchain available.
- Dedicated node host with persistent disk and stable network connectivity.
- Operator-defined environment profile (`localnet`, `devnet`, `testnet`, or `mainnet`).

## 2. Build the Operator Binary

```bash
cargo build -p aoxcmd --release
```

Verify:

```bash
./target/release/aoxc --help
./target/release/aoxc version
```

## 3. Initialize Node Identity and Genesis Material

Use a strong password and avoid shell history leakage:

```bash
read -rsp "AOXC password: " AOXC_PASS; echo
./target/release/aoxc key-bootstrap --profile testnet --password "$AOXC_PASS"
./target/release/aoxc genesis-init --profile testnet
unset AOXC_PASS
```

Validate deterministic bundle identity before start:

```bash
./target/release/aoxc genesis-validate --strict
./target/release/aoxc genesis-production-gate
./target/release/aoxc network-identity-gate --enforce --env testnet --format json
```

## 4. Start Node and Join Network

```bash
./target/release/aoxc node start --profile testnet
```

In a separate terminal:

```bash
./target/release/aoxc node status --profile testnet
./target/release/aoxc network status --profile testnet
./target/release/aoxc network verify --profile testnet
```

## 5. Query Chain and RPC Health

```bash
./target/release/aoxc query chain status
./target/release/aoxc query network peers
./target/release/aoxc api status
```

## 6. Re-bootstrap Safety Rule

If reusing an existing node home after a prior bootstrap, remove stale runtime DB before a fresh bootstrap:

```bash
rm -f /path/to/aoxc/home/testnet/runtime/db/main.redb
```

This avoids deterministic parent-hash mismatch failures caused by mixing historical runtime state with a newly initialized genesis.

## 7. Multi-node Formation (Single Host)

Use topology bootstrap for deterministic multi-node labs:

```bash
./target/release/aoxc topology-bootstrap \
  --mode mainchain-4 \
  --password '<strong-password>' \
  --allocation-preset validator-heavy \
  --output-dir /tmp/aoxc-topology-mainchain4
```

The generated output includes per-node RPC, metrics, startup, and query hints.

## 8. Operational Hardening Checklist

- Run `make testnet-readiness-gate` before promotion claims.
- Retain artifacts under `artifacts/` with commit linkage.
- Follow `SECURITY.md` private disclosure flow for suspected vulnerabilities.
- Keep node profile and release-policy manifests hash-consistent across environments.
