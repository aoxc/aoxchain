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

### `scripts/release/generate_release_evidence.sh`
Release evidence generation workflow.

This script supports release publication assurance and operator-verifiable
release evidence generation.

### `scripts/release_artifact_certify.sh`
Release artifact certification helper.

This script validates and certifies release artifacts according to the
repository’s release assurance expectations.

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

The `aoxc_full_4nodes.sh` flow provisions a production-oriented local four-node
layout under a dedicated root, copies canonical genesis/config materials,
bootstraps node homes, creates per-node snapshots, and can emit Docker assets
for isolated node services.

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
