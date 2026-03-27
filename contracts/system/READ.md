# AOXC System Contracts — Mainnet/Testnet/Devnet Ready

This folder contains system-level contract artifacts and protocol logic drafts for
production-oriented deployment flows.

## Included Components

- `governance.rs` — governance and administrative control logic.
- `bridge_logic.rs` — bridge flow and settlement integration logic.
- `staking.sol` — staking baseline contract (Solidity side).
- `deployment-matrix.toml` — network-specific deployment and policy profile.

## Environment Compatibility

This folder is maintained for full compatibility with:

- Mainnet (strict governance and safety controls)
- Testnet (pre-production validation and migration rehearsals)
- Devnet (fast iteration with guarded controls)

## Production Controls (Must-Have)

- Explicit timelock windows and quorum policy.
- Explicit bridge guardrails and emergency stop behavior.
- Explicit staking limits and slashing posture by environment.
- Deterministic release notes and deployment matrix verification.

## Reading Order (Always)

1. `deployment-matrix.toml`
2. `governance.rs`
3. `bridge_logic.rs`
4. `staking.sol`

## Release Gate Checklist

- Environment profile selected (mainnet/testnet/devnet).
- Governance timelock and quorum values are non-default and audited.
- Bridge pause/recovery procedure documented.
- Staking reward/slash parameters validated for the target environment.
- Final review completed and approved by protocol + security owners.
