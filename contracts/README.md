# AOXC Contracts — Production Compatibility Guide

This directory contains AOXC contract and protocol-logic surfaces that must remain
compatible across:

- **Mainnet**
- **Testnet**
- **Devnet**

## Scope

- System-level governance contract logic
- Bridge and interoperability logic
- Staking contract examples and policy baselines

## Structure

- `contracts/system/governance.rs`
- `contracts/system/bridge_logic.rs`
- `contracts/system/staking.sol`
- `contracts/system/READ.md`
- `contracts/system/deployment-matrix.toml`

## Production Readiness Rules

A contracts release is considered 100% production-ready when:

1. Mainnet/Testnet/Devnet deployment profile is defined.
2. Security controls (timelock, quorum, validator policy) are explicit.
3. Upgrade policy and emergency pause semantics are documented.
4. Bridge validation assumptions are environment-scoped.
5. Staking parameters are aligned with target environment policy.

## Recommended Review Workflow

1. Read `contracts/system/READ.md`.
2. Validate environment controls in `contracts/system/deployment-matrix.toml`.
3. Review source files for policy alignment.
4. Run chain-specific deployment checks before release.
