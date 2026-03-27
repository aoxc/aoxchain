# AOXC Scripts — Production Operations Guide

Scope: `scripts/`

This directory contains release, validation, and runtime automation scripts for AOXC.
The script surface is designed to be production-oriented and compatible with:

- Mainnet
- Testnet
- Devnet

## Key Scripts

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

## Operational Safety Notes

- Scripts resolve `aoxc` binary from `$BIN_PATH`, `$HOME/.aoxc/bin/aoxc`, or `./bin/aoxc`.
- Each environment uses isolated state directory: `./.aoxc-<env>`.
- Each environment logs to: `./logs/network/<env>/runtime.log`.
- `start` is idempotent and avoids duplicate daemons via PID checks.
