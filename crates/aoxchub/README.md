# AOXCHUB

AOXCHUB is the desktop and web operator interface for AOXChain. It provides a
single control surface for consensus visibility, execution-lane monitoring,
treasury workflows, node operations, governance intents, and security posture
inspection.

## Purpose

- Provide production-oriented operational visibility from authoritative chain
  and telemetry integrations.
- Keep custody and protocol-kernel boundaries explicit: AOXCHUB is a control
  plane, not a consensus kernel or signing engine.
- Offer profile-aware behavior across mainnet, devnet, and testnet routing.

## Contents

- `src/views/operations.rs`: routed operator screens (overview, consensus,
  staking, telemetry, governance, settings).
- `src/services/`: read models and profile-aware integration adapters consumed
  by the UI.
- `src/components/` and `assets/`: reusable UI primitives and desktop styling.
- `SECURITY.md`, `TESTING.md`, `SCOPE.md`, `ARCHITECTURE.md`: governance and
  engineering boundaries for production operation.

## Usage

### Local desktop development

```bash
dx serve --platform desktop
```

### Cargo execution

```bash
cargo run -p aoxchub
```

### Feature selection

- `desktop` (default): desktop runtime.
- `web`: browser runtime.
- `mobile`: mobile runtime.
- `server`: server/fullstack feature set.

## Notes

- AOXCHUB does not execute privileged chain-kernel logic.
- Wallet approvals and governance actions remain policy-gated through external
  signer and approval systems.
- This repository is distributed under the MIT License; no warranty is
  provided.
