# AOXC Scripts — Production Operations Guide

**Scope:** `scripts/`

This directory contains the operational automation surface for AOXC. It includes release workflows, validation gates, local runtime helpers, environment daemons, and operator-facing orchestration utilities.

The script surface is intended to support production-disciplined workflows across the following primary network profiles:

- `mainnet`
- `testnet`
- `devnet`

Additional repository-supported runtime scopes also include `local-dev`, `localnet`, and `real-chain`, but the primary network daemon layer is centered on `mainnet`, `testnet`, and `devnet`.

## Key Scripts

- `scripts/aoxc_easy.sh`
  - Beginner-friendly operator helper aligned with the Make-driven operations surface.
- `scripts/network_env_daemon.sh`
  - Primary single-environment daemon orchestrator used for `start`, `once`, `status`, `stop`, and log tail flows across `mainnet`, `testnet`, and `devnet`.
- `scripts/network_stack.sh`
  - Dual-environment orchestration layer used for coordinated `testnet + mainnet` lifecycle control.
- `scripts/autonomy_control_plane.sh`
  - Higher-level autonomy and operational coordination surface.
- `scripts/real_chain_daemon.sh`
  - Local real-chain runtime helper used for `run`, `once`, `health`, and `tail` flows.
- `scripts/node_supervisor.sh`
  - Local packaged runtime supervisor used by the local developer support surface.
- `scripts/continuous_producer.sh`
  - Continuous local producer helper for local runtime workflows.
- `scripts/run-local.sh`
  - Local packaged node entrypoint for isolated `local-dev` execution.
- `scripts/quality_gate.sh`
  - Repository quality gate entrypoint for quick, full, and release-oriented checks.
- `scripts/autonomy_sqlite_ctl.py`
  - SQLite-backed operator state and event control helper.
- `scripts/validation/*`
  - Readiness gates, persistent environment checks, multi-host validation, and operational closure scenarios.
- `scripts/release/generate_release_evidence.sh`
  - Release evidence generation workflow associated with publication and release assurance.

## Make Integration

The script surface is wired directly into the repository Makefile and should remain compatible with the following target groups.

### Environment and Path Management

- `make paths`
- `make env-check`
- `make bootstrap-paths`
- `make bootstrap-desktop-paths`
- `make bootstrap-env-paths AOXC_ENV=devnet`
- `make env-print AOXC_ENV=devnet`
- `make env-source-check AOXC_ENV=devnet`
- `make env-install AOXC_ENV=devnet`
- `make env-verify AOXC_ENV=devnet`
- `make env-activate AOXC_ENV=devnet`
- `make env-status AOXC_ENV=devnet`
- `make env-fingerprint AOXC_ENV=devnet`
- `make env-doctor AOXC_ENV=devnet`
- `make env-reinstall AOXC_ENV=devnet`
- `make env-clean AOXC_ENV=devnet`
- `make env-reset AOXC_ENV=devnet`
- `make env-show-active AOXC_ENV=devnet`
- `make env-sync-default AOXC_ENV=devnet`

### Environment-Specific Installation and Activation

- `make env-install-mainnet`
- `make env-install-testnet`
- `make env-install-devnet`
- `make env-install-localnet`
- `make env-verify-mainnet`
- `make env-verify-testnet`
- `make env-verify-devnet`
- `make env-verify-localnet`
- `make env-activate-mainnet`
- `make env-activate-testnet`
- `make env-activate-devnet`
- `make env-activate-localnet`
- `make env-bootstrap-mainnet`
- `make env-bootstrap-testnet`
- `make env-bootstrap-devnet`
- `make env-bootstrap-localnet`

### Local Developer Helpers

- `make dev-bootstrap`
- `make run-local`
- `make supervise-local`
- `make audit-install`
- `make produce-loop`

### Local Real-Chain Workflow

- `make real-chain-prep`
- `make real-chain-run`
- `make real-chain-run-once`
- `make real-chain-health`
- `make real-chain-tail`

### Single-Environment Network Operations

- `make net-mainnet-start`
- `make net-mainnet-once`
- `make net-mainnet-status`
- `make net-mainnet-stop`
- `make net-testnet-start`
- `make net-testnet-once`
- `make net-testnet-status`
- `make net-testnet-stop`
- `make net-devnet-start`
- `make net-devnet-once`
- `make net-devnet-status`
- `make net-devnet-stop`

### Dual-Environment Network Operations

- `make net-dual-start`
- `make net-dual-once`
- `make net-dual-status`
- `make net-dual-stop`
- `make net-dual-restart`

### Operator Convenience Targets

- `make ops-help`
- `make ops-doctor`
- `make ops-auto-prepare`
- `make ops-auto-bootstrap`
- `make ops-start-mainnet`
- `make ops-start-testnet`
- `make ops-start-devnet`
- `make ops-start-dual`
- `make ops-auto-start`
- `make ops-auto-once`
- `make ops-stop-mainnet`
- `make ops-stop-testnet`
- `make ops-stop-devnet`
- `make ops-stop-dual`
- `make ops-status-mainnet`
- `make ops-status-testnet`
- `make ops-status-devnet`
- `make ops-status-dual`
- `make ops-restart-mainnet`
- `make ops-restart-testnet`
- `make ops-restart-devnet`
- `make ops-restart-dual`
- `make ops-logs-mainnet`
- `make ops-logs-testnet`
- `make ops-logs-devnet`
- `make ops-dashboard`
- `make ops-flow-mainnet`
- `make ops-flow-testnet`
- `make ops-flow-devnet`
- `make ops-autonomy-blueprint`

### Quality, Build, and Release Integration

- `make fmt`
- `make check`
- `make test`
- `make clippy`
- `make audit`
- `make quality`
- `make quality-quick`
- `make quality-release`
- `make ci`
- `make build`
- `make build-release`
- `make build-release-all`
- `make build-release-mainnet`
- `make build-release-testnet`
- `make build-release-devnet`
- `make build-release-matrix`
- `make package-bin`
- `make release-binary-list`
- `make package-all-bin`
- `make package-versioned-bin`
- `make package-versioned-archive`
- `make package-network-versioned-bin`
- `make package-desktop-testnet`
- `make publish-release`

### SQLite-Backed Operator State

- `make db-init-sqlite`
- `make db-status-sqlite`
- `make db-event-sqlite`
- `make db-release-sqlite`
- `make db-history-sqlite`

### UI Surfaces

- `make ui-mainnet`
- `make ui-testnet`
- `make ui-devnet`

## Canonical Path Contract

The repository Makefile defines the AOXC runtime path contract around the following canonical roots:

- `AOXC_DATA_ROOT ?= $(HOME)/.AOXCData`
- `AOXC_HOME ?= $(AOXC_DATA_ROOT)/home/default`
- `AOXC_BIN_DIR ?= $(AOXC_DATA_ROOT)/bin`
- `AOXC_BIN_PATH ?= $(AOXC_BIN_DIR)/aoxc`
- `AOXC_RELEASES_DIR ?= $(AOXC_DATA_ROOT)/releases`
- `AOXC_NETWORK_BIN_ROOT ?= $(AOXC_DATA_ROOT)/binary`

Environment-scoped runtime homes include:

- `$(AOXC_DATA_ROOT)/home/local-dev`
- `$(AOXC_DATA_ROOT)/home/real`
- `$(AOXC_DATA_ROOT)/home/mainnet`
- `$(AOXC_DATA_ROOT)/home/testnet`
- `$(AOXC_DATA_ROOT)/home/devnet`
- `$(AOXC_DATA_ROOT)/home/localnet`

Environment-scoped log roots include:

- `$(AOXC_DATA_ROOT)/logs/real-chain`
- `$(AOXC_DATA_ROOT)/logs/mainnet`
- `$(AOXC_DATA_ROOT)/logs/testnet`
- `$(AOXC_DATA_ROOT)/logs/devnet`
- `$(AOXC_DATA_ROOT)/logs/localnet`

A dedicated desktop testnet runtime surface is also defined under:

- `$(AOXC_DATA_ROOT)/desktop/testnet`

## Operational Safety Notes

- Scripts should resolve the `aoxc` binary deterministically, typically using:
  - `$BIN_PATH`
  - `$HOME/.AOXCData/bin/aoxc`
  - `./bin/aoxc`
- Each environment should use an isolated runtime home and isolated log directory.
- Environment activation should remain aligned with the canonical bundle model under `configs/environments/<env>/`.
- `start` flows should be idempotent and must avoid duplicate managed processes.
- `status` flows should report operator-meaningful runtime state rather than only file presence.
- `stop` flows should terminate only the intended managed process set.
- Release, quality, and validation scripts should fail closed on invalid prerequisites or missing dependencies.
- Script interfaces must remain compatible with the repository Makefile contract when refactoring or renaming operational entrypoints.

## Recommended Usage Model

For repository-aligned routine operations, the preferred flow is:

1. `make env-check`
2. `make bootstrap-paths`
3. `make package-all-bin`
4. `make env-activate AOXC_ENV=devnet`
5. `make ops-doctor AOXC_ENV=devnet`
6. `make ops-auto-start AOXC_ENV=devnet`

This sequence matches the repository’s Make-driven operational model and preserves compatibility with environment activation, runtime packaging, and daemon orchestration flows.
