# AOXC Scripts — Single-Runtime Operations Guide

**Scope:** `scripts/`

This directory contains the operational automation surface for AOXC under the
single-runtime operating model.

The script layer is intentionally aligned with the following architectural
decisions:

- One portable AOXC root per host
- One canonical runtime surface per host
- No multi-script environment fan-out in the script contract
- No `mainnet` / `testnet` / `devnet` orchestration in this layer
- No SQLite-backed autonomy control surface
- No Python-based operator-memory helpers
- `redb` is the canonical embedded database backend
- Repository configuration and canonical runtime source material determine the
  effective runtime profile outside this script layer

---

## Script Surface

### `scripts/runtime_daemon.sh`
Primary runtime lifecycle controller.

Supported flows:

- `start`
- `once`
- `status`
- `stop`
- `restart`
- `tail`
- `install-service`

This script is the authoritative operational lifecycle surface for bootstrap,
managed execution, PID tracking, runtime logging, and basic smoke execution.
It can also install a persistent systemd service unit for automatic restart and
boot-time startup.

### Persistent service operation

To run AOXC as a persistent service (auto-restart enabled):

```bash
./scripts/runtime_daemon.sh install-service
systemctl --user start aoxc-runtime.service
```

For reboot persistence in user scope, enable linger once:

```bash
loginctl enable-linger "$USER"
```

Optional environment variables:

- `AOXC_SYSTEMD_SCOPE=user|system` (default: `user`)
- `AOXC_SYSTEMD_SERVICE_NAME=<service-name>` (default: `aoxc-runtime`)

### `scripts/lib/runtime_common.sh`
Shared hardening library used by wrapper scripts.

Provides:

- canonical repository root resolution,
- fail-closed input validation helpers for numeric/string/key fields,
- standardized make-target execution with consistent logging,
- reusable executable/runtime contract checks.

Wrapper scripts (`chain_create.sh`, `wallet_seed.sh`, `validator_bootstrap.sh`,
`network_start.sh`, `network_stop.sh`, `preflight_check.sh`, `runtime_recover.sh`,
`finality_smoke.sh`, `transfer_smoke.sh`, and `chain_demo.sh`) depend on this
surface to keep argument validation and runtime invocation behavior consistent.

### `scripts/run_runtime.sh`
Thin smoke-flow entrypoint.

This script delegates directly to `runtime_daemon.sh once` and exists only to
provide the smallest meaningful packaged runtime execution path.

### `scripts/quality_gate.sh`
Canonical repository quality gate entrypoint.

Intended quality flows include:

- quick
- full
- release

### `scripts/validate_environment_bundle.py`
Cross-environment identity and checksum compatibility validator for the
single-system runtime model.

This validator checks mainnet/testnet/devnet bundles for:

- Identity consistency (`environment`, `network_id`, `chain_id`)
- `genesis.v1.sha256` correctness
- AOXHub profile alignment with canonical bundle roots

### `scripts/validation/persistent_testnet_gate.sh`
Persistent testnet readiness gate used by `make testnet-gate` and the `make testnet` workflow.

This gate validates:

- presence of the full testnet identity/configuration artifact set,
- cross-environment bundle consistency checks,
- testnet runtime-source integrity through the Make runtime surface,
- `network-metadata.json` identity alignment with `manifest.v1.json`.

### `scripts/testnet_fullnode_release_start.sh`
Release-aligned real testnet full-node launcher.

This script supports:

- explicit `--home` full-node runtime path,
- release-path publication under `/aoxchain/releases/v<workspace-version>/bin/aoxc` (configurable),
- locked release build for `aoxc` when missing or forced,
- strict workspace/version-policy parity checks before launch,
- optional bootstrap skip and direct `node start` execution.

### `scripts/testnet_production_orchestrator.sh`
Production-oriented testnet topology planner and orchestrator.

This script supports:

- fail-closed testnet topology sizing (`validators >= 3`, `bootnodes >= 2`),
- production-oriented default topology generation (validators, sentries, bootnodes, RPC, archive),
- deterministic node inventory output (`plans/nodes.csv`),
- deterministic command generation (`plans/commands.sh`) for bootstrap and start,
- strict execute-mode password policy (`>=16` chars, mixed classes, no whitespace),
- non-persistent command script secrets (uses `AOXC_TESTNET_PASSWORD` at execution time),
- optional local execution mode (`--execute`) when an AOXC binary is available.

### `scripts/validation/os_compatibility_gate.sh`
Cross-platform compatibility gate used by `make os-compat-gate`.

This gate validates:

- required host/container compatibility surfaces (`Dockerfile`, `docker-compose.yaml`, `Makefile`, docs),
- platform contract markers for Linux/macOS/Windows in the Makefile,
- explicit documentation coverage for Linux/NixOS/macOS/Windows/Docker.

The gate also emits `artifacts/os-compat/summary.json` for review traceability.

### `scripts/validation/aoxcvm_production_closure_gate.sh`
AOXCVM full production closure gate used by `make aoxcvm-production-closure-gate`.

This gate enforces all closure classes together:

- test (`scripts/validation/aoxcvm_phase3_gate.sh`),
- audit (`cargo audit`),
- rehearsal (`scripts/validation/os_compatibility_gate.sh`),
- evidence (`artifacts/aoxcvm-phase3/evidence-bundle/artifacts-manifest.json` presence).

The gate is fail-closed: overall PASS is reported only when every class passes.
It emits `artifacts/aoxcvm-phase3/production-closure-summary.json` for traceable review evidence.

### `scripts/validation/cargo_deny_gate.sh`
Dependency policy gate used by `make cargo-deny-gate`.

This gate:

- verifies `cargo-deny` is available on the operator host,
- executes `cargo deny check` against repository policy (`deny.toml`),
- fails with a clear installation message when `cargo-deny` is missing.

### `scripts/release/generate_release_evidence.sh`
Release evidence generation workflow.

This script supports release publication assurance and operator-verifiable
release evidence generation.

### `scripts/release_artifact_certify.sh`
Release artifact certification helper.

This script validates and certifies release artifacts according to the
repository’s release assurance expectations.

### `scripts/release/secure_binary_bundle.sh`
Secure binary bundling workflow with defense-in-depth hashing and signing.

This script creates release bundles that include:

- complete binary set (`aoxc`, `aoxchub`, `aoxckit`),
- `SHA256SUMS`,
- `SHA3-512SUMS`,
- `BLAKE3SUMS` (`unavailable` marker when `b3sum` is not installed),
- `manifest.secure.json` with git commit and certificate fingerprint,
- detached signatures for manifest and every checksum surface.

Required inputs:

- `RELEASE_SIGNING_KEY` (private key),
- `RELEASE_SIGNING_CERT` (X.509 certificate).

### `scripts/release/verify_secure_binary_bundle.sh`
Secure bundle verifier for signed multi-hash bundles.

This script verifies:

- detached signatures for secure manifest and checksum files,
- manifest-to-binary hash parity for SHA-256 and SHA3-512,
- BLAKE3 parity when BLAKE3 hashes are present.

### `scripts/release/github_binary_install.sh`
GitHub release binary download and installation workflow.

This script supports:

- downloading release archives from `releases/download/v<version>/`,
- checksum verification before extraction,
- optional certificate-based signature verification for checksums,
- full binary installation (`aoxc`, `aoxchub`, `aoxckit`) into a target directory.

Default install and download behavior is fail-closed and root-path oriented:

- install path defaults to `<AOXC_ROOT>/bin/current`,
- download cache defaults to `<AOXC_ROOT>/downloads/github/v<version>/<platform>`.

### Operator orchestration wrappers
The following scripts provide high-level workflow wrappers and intentionally
defer critical validation logic to `aoxc` and `make` surfaces:

- `scripts/chain_demo.sh`
- `scripts/chain_create.sh`
- `scripts/network_start.sh`
- `scripts/network_stop.sh`
- `scripts/validator_bootstrap.sh`
- `scripts/wallet_seed.sh`
- `scripts/preflight_check.sh`
- `scripts/finality_smoke.sh`
- `scripts/transfer_smoke.sh`
- `scripts/runtime_recover.sh`

`scripts/wallet_seed.sh` enforces fail-closed input validation before calling
`make chain-add-account`:

- account IDs must match `^[A-Za-z0-9_.:-]{3,64}$`,
- roles are normalized to lowercase and restricted to canonical genesis roles,
- balance must be non-zero by default (`AOXC_ALLOW_ZERO_BALANCE=1` explicitly
  opts into zero-balance seeding for controlled bootstrap workflows).

---

## Canonical Runtime Model

The script layer assumes one portable AOXC root.

Typical examples:

### Linux / POSIX fallback
```text
$HOME/.aoxc
```

### XDG-aware Linux
```text
$XDG_STATE_HOME/aoxc
$XDG_DATA_HOME/aoxc
```

### Windows
```text
%LOCALAPPDATA%/AOXC
%APPDATA%/AOXC
```

The canonical single-runtime layout is:

```text
<AOXC_ROOT>/
  bin/
    current/
    versioned/
  releases/
  logs/
  runtime/
    identity/
    config/
    state/
    operator/
    db/
  audit/
    operator-events.jsonl
    release-events.jsonl
    db-status.latest.json
    runtime-install.receipt
    runtime-health.latest.txt
  artifacts/
  cache/
  tmp/
  active-profile
```

Unless an explicit install path is provided, binary installation flows should
use `<AOXC_ROOT>/bin/current` as the canonical destination.

---

## GitHub Binary Distribution Paths

For GitHub-hosted release consumption, use a dedicated path contract:

```text
<AOXC_ROOT>/
  downloads/
    github/
      v<version>/
        <platform>/
          aoxc-v<version>-<platform>-portable.tar.gz
          SHA256SUMS
          SHA256SUMS.sig (optional but recommended)
          extract/
```

Recommended installation destination:

```text
<AOXC_ROOT>/bin/current/
  aoxc
  aoxchub
  aoxckit
```

This separation keeps downloaded artifacts, extraction scratch data, and active
runtime binaries distinct for operational clarity and auditability.

---

## Runtime Source Contract

The script layer expects canonical runtime source material to be maintained under:

```text
configs/environments/<network-kind>/
```

`<network-kind>` is selected through `AOXC_NETWORK_KIND` (default: `mainnet`).
This preserves one runtime lifecycle code path while allowing the genesis
identity (`environment`) to declare `mainnet`, `testnet`, or `devnet`.

Expected source artifacts include:

- `manifest.v1.json`
- `genesis.v1.json`
- `genesis.v1.sha256`
- `validators.json`
- `bootnodes.json`
- `certificate.json`
- `profile.toml`
- `release-policy.toml`

These materials are copied or materialized into the runtime root by the Make
surface.

---

## Make Integration

The script layer is expected to remain compatible with the single-runtime Make
surface.

### Core diagnostics and bootstrap
- `make help`
- `make paths`
- `make env-check`
- `make bootstrap-paths`
- `make bootstrap-desktop-paths`

### Build and packaging
- `make build`
- `make build-release`
- `make build-release-all`
- `make build-release-matrix`
- `make package-bin`
- `make package-all-bin`
- `make package-versioned-bin`
- `make package-versioned-archive`
- `make publish-release`
- `make package-desktop`

### Runtime lifecycle
- `make runtime-print`
- `make runtime-refresh-genesis-sha256`
- `make runtime-source-check`
- `make runtime-install`
- `make runtime-verify`
- `make runtime-activate`
- `make runtime-status`
- `make runtime-fingerprint`
- `make runtime-doctor`
- `make runtime-reinstall`
- `make runtime-reset`
- `make runtime-show-active`
- `make runtime-snapshot`
- `make runtime-snapshot-list`
- `make runtime-snapshot-prune`
- `make runtime-restore-latest`

Runtime snapshot surfaces are implemented by `scripts/runtime_snapshot_guard.sh`.
These commands create timestamped runtime snapshot archives and can restore the
latest snapshot into the canonical runtime root.

### Database and audit
- `make db-init`
- `make db-status`
- `make db-event`
- `make db-release`
- `make db-history`
- `make db-health`

### Operations
- `make ops-help`
- `make ops-doctor`
- `make ops-prepare`
- `make ops-start`
- `make ops-once`
- `make ops-stop`
- `make ops-status`
- `make ops-restart`
- `make ops-logs`
- `make ops-flow`

### Full multi-node local system
- `make aoxc-full-4nodes`
- `make aoxc-full-4nodes-docker`
- `scripts/aoxc_full_4nodes.sh --help`
- `scripts/aoxc-rolling.start.sh --help`

The `aoxc_full_4nodes.sh` flow provisions a production-oriented local four-node
layout under a dedicated root, copies canonical genesis/config materials,
bootstraps node homes, creates per-node snapshots, and can emit Docker assets
for isolated node services.

The `aoxc-rolling.start.sh` flow provisions a seven-node rolling devnet style
local testnet layout (no Docker/Podman requirement), prepares per-node account
material, and can run persistent local node loops from generated `start-all.sh`
and `stop-all.sh` control scripts.

### Quality
- `make fmt`
- `make check`
- `make test`
- `make clippy`
- `make audit`
- `make quality`
- `make quality-quick`
- `make quality-release`
- `make ci`

### UI
- `make ui`

---

## Operational Safety Notes

- Scripts must resolve the `aoxc` binary deterministically.
- Runtime operations must bind to one canonical runtime root.
- Log files, PID files, and receipts must remain inside AOXC-owned paths.
- `start` flows must be idempotent and must avoid duplicate managed processes.
- `status` flows must report meaningful runtime state.
- `stop` flows must terminate only the intended managed process.
- Release and validation scripts must fail closed on missing prerequisites.
- Script refactors must not reintroduce legacy environment fan-out.
- Deprecated SQLite/autonomy surfaces must not be reintroduced.

---

## Recommended Usage Model

The preferred repository-aligned operating sequence is:

1. `make env-check`
2. `make bootstrap-paths`
3. `make package-all-bin`
4. `make runtime-activate`
5. `make ops-doctor`
6. `make ops-start`

This sequence preserves a deterministic single-runtime workflow aligned with the
current AOXC operating model.
