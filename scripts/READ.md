Examples include:

- readiness validation
- multi-host validation
- production closure validation

Validation scripts may reference repository-defined product profiles or
deployment scenarios, but the operational script layer itself must remain
single-runtime.

---

## Canonical Runtime Model

The script layer assumes one portable AOXC root.

Typical examples:

### Linux / POSIX fallback
```text
$HOME/.aoxc
XDG-aware Linux
$XDG_STATE_HOME/aoxc
$XDG_DATA_HOME/aoxc
Windows
%LOCALAPPDATA%/AOXC
%APPDATA%/AOXC

The canonical single-runtime layout is:

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
Runtime Source Contract

The script layer expects canonical runtime source material to be maintained under:

configs/runtime/

Expected source artifacts include:

manifest.v1.json
genesis.v1.json
genesis.v1.sha256
validators.json
bootnodes.json
certificate.json
profile.toml
release-policy.toml

These materials are copied or materialized into the runtime root by the Make
surface.

Make Integration

The script layer is expected to remain compatible with the single-runtime Make
surface.

Core diagnostics and bootstrap
make help
make paths
make env-check
make bootstrap-paths
make bootstrap-desktop-paths
Build and packaging
make build
make build-release
make build-release-all
make build-release-matrix
make package-bin
make package-all-bin
make package-versioned-bin
make package-versioned-archive
make publish-release
make package-desktop
Runtime lifecycle
make runtime-print
make runtime-refresh-genesis-sha256
make runtime-source-check
make runtime-install
make runtime-verify
make runtime-activate
make runtime-status
make runtime-fingerprint
make runtime-doctor
make runtime-reinstall
make runtime-reset
make runtime-show-active
Database and audit
make db-init
make db-status
make db-event
make db-release
make db-history
make db-health
Operations
make ops-help
make ops-doctor
make ops-prepare
make ops-start
make ops-once
make ops-stop
make ops-status
make ops-restart
make ops-logs
make ops-flow
Quality
make fmt
make check
make test
make clippy
make audit
make quality
make quality-quick
make quality-release
make ci
UI
make ui
Operational Safety Notes
Scripts must resolve the aoxc binary deterministically.
Runtime operations must bind to one canonical runtime root.
Log files, PID files, and receipts must remain inside AOXC-owned paths.
start flows must be idempotent and must avoid duplicate managed processes.
status flows must report meaningful runtime state.
stop flows must terminate only the intended managed process.
Release and validation scripts must fail closed on missing prerequisites.
Script refactors must not reintroduce legacy environment fan-out.
Deprecated SQLite/autonomy surfaces must not be reintroduced.
Recommended Usage Model

The preferred repository-aligned operating sequence is:

make env-check
make bootstrap-paths
make package-all-bin
make runtime-activate
make ops-doctor
make ops-start

This sequence preserves a deterministic single-runtime workflow aligned with the
current AOXC operating model.
