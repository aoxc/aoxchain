# Deterministic Five-Node Testnet Fixture

This fixture is intended for deterministic development, demo, and CI validation. It models a multi-node AOX Chain deployment with stable validator identities, a reproducible genesis file, and explicit node-specific configuration files.

## Scope

The fixture generated under `configs/deterministic-testnet/` is designed to stay operationally close to the main deployment flow while remaining clearly marked as a test network.

Generated assets include:

- `accounts.json`: deterministic validator inventory and funding plan.
- `genesis.json`: reproducible genesis payload for the fixture network.
- `nodes/*.toml`: node-specific networking plans.
- `homes/<node>/identity/test-node-seed.hex`: deterministic local fixture seed material.
- `launch-testnet.sh`: helper script that boots the fixture nodes in sequence.

## Fixed validator set

The fixture currently models five stable validators:

- atlas
- boreal
- cypher
- delta
- ember

## Generation command

```bash
cargo run -q -p aoxcmd -- testnet-fixture-init \
  --output-dir configs/deterministic-testnet \
  --chain-num 77 \
  --fund-amount 2500000000000000000000
```

## Launch command

```bash
bash configs/deterministic-testnet/launch-testnet.sh
```

## Testnet parity policy

This fixture should follow the same execution, validation, and operational flow expected from the main network wherever practical. The intentional differences must remain explicit:

- the network is labeled as testnet-only,
- fixture keys and seeds must never be reused for mainnet custody,
- public-facing identifiers may carry testnet-specific naming to prevent operator confusion,
- economic value and trust assumptions remain non-production.

## Security notice

The deterministic seeds used by this fixture are not production secrets. They are suitable only for local development, CI, demos, and controlled interoperability rehearsals.
