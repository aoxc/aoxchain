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

### `scripts/release/generate_release_evidence.sh`
Release evidence generation workflow.

This script supports release publication assurance and operator-verifiable
release evidence generation.

### `scripts/release_artifact_certify.sh`
Release artifact certification helper.

This script validates and certifies release artifacts according to the
repository’s release assurance expectations.

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
- `scripts/aoxc-q-v0.2.0.start.sh --help`

The `aoxc_full_4nodes.sh` flow provisions a production-oriented local four-node
layout under a dedicated root, copies canonical genesis/config materials,
bootstraps node homes, creates per-node snapshots, and can emit Docker assets
for isolated node services.

The `aoxc-q-v0.2.0.start.sh` flow provisions a seven-node AOXC-Q v0.2.0 style
local testnet layout (no Docker/Podman requirement), prepares per-node account
material, and can run persistent local node loops from generated `start-all.sh`
and `stop-all.sh` control scripts. Per-node bootstrap command logs are written
under `nodes/<node>/run/*.log` for failure diagnosis. Operator password can be
provided with `--password`, `--password-file`, `AOXC_Q_PASSWORD`, or an
interactive prompt (`--prompt-password`). The script now exposes explicit
subcommands (`up`, `bootstrap`, `start`, `stop`, `status`, `logs`) for
operational lifecycle control.

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
