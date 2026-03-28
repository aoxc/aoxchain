# AOXC Scripts — Production Operations Guide

Scope: `scripts/`

This directory contains release, validation, and runtime automation scripts for AOXC.
The script surface is designed to be production-oriented and compatible with:

- Mainnet
- Testnet
- Devnet

## Key Scripts

- `scripts/aoxc_easy.sh`
  - Beginner-friendly operations CLI (`doctor/start/stop/status/restart/logs/menu`).
- `scripts/network_env_daemon.sh`
  - Unified orchestrator for `start|once|status|stop` across `mainnet|testnet|devnet`.
- `scripts/real_chain_daemon.sh`
  - Legacy loop daemon for local real-chain simulation.
- `scripts/validation/*`
  - Readiness gates, multi-host checks, and production-closure scenarios.
- `scripts/release/generate_release_evidence.sh`
  - Release evidence generation workflow.

## Make Integration

Use Make targets for environment operations:

- `make net-mainnet-start`, `make net-mainnet-once`, `make net-mainnet-status`, `make net-mainnet-stop`
- `make net-testnet-start`, `make net-testnet-once`, `make net-testnet-status`, `make net-testnet-stop`
- `make net-devnet-start`, `make net-devnet-once`, `make net-devnet-status`, `make net-devnet-stop`
- `make ops-help`, `make ops-doctor`
- `make ops-start-mainnet|ops-start-testnet|ops-start-devnet`
- `make ops-status-mainnet|ops-status-testnet|ops-status-devnet`
- `make ops-stop-mainnet|ops-stop-testnet|ops-stop-devnet`
- `make ops-restart-mainnet|ops-restart-testnet|ops-restart-devnet`
- `make ops-logs-mainnet|ops-logs-testnet|ops-logs-devnet`

## Operational Safety Notes

- Scripts resolve `aoxc` binary from `$BIN_PATH`, `$HOME/.AOXCData/bin/aoxc`, or `./bin/aoxc`.
- Each environment uses isolated state directory: `$HOME/.AOXCData/home/<env>` (or `$AOXC_HOME_DIR`).
- Each environment logs to: `$HOME/.AOXCData/logs/network/<env>/runtime.log` (or `$LOG_DIR`).
- `start` is idempotent and avoids duplicate daemons via PID checks.
