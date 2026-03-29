# AOXC Hub

AOXC Hub is the desktop-facing operational surface for AOXChain.  
This crate currently provides an integrated UI shell that connects navigation, route hosting, and all feature sections in one frame while allowing content depth to be expanded iteratively.

## Purpose

- Provide a single operator interface for wallet onboarding, explorer overview, dashboard metrics, operations monitoring, settings, and ecosystem domains.
- Keep the interface non-empty and presentation-complete while backend integrations are hardened step-by-step.
- Preserve a deterministic structure for Dev, Testnet, and Mainnet rollout preparation.

## Current Contents

- **App shell:** top header, left sidebar, route outlet, and footer.
- **Feature sections:** wallet, overview, dashboard, operations, settings, and domain panels.
- **Embedded integration checklist:** visible in UI as an implementation tracker for system-level readiness.
- **Global styling:** desktop-oriented theme and layout primitives.

## Usage

From repository root:

```bash
cargo run -p aoxchub
```

For Dioxus development workflow:

```bash
dx serve --platform desktop
```

## Integration Checklist (UI-aligned)

1. Network profile selector (Dev/Testnet/Mainnet) with strict environment isolation.
2. RPC compatibility handshake (version, chain-id, genesis hash, capability checks).
3. Wallet policy and signature boundary enforcement.
4. Observability wiring (logs, health signals, operator diagnostics).
5. Release gate with build checks and smoke validation.

## Notes

- This crate remains under active integration; interface composition is complete, while deep business content and production connectors can be layered in subsequent iterations.
- The repository is MIT-licensed; operational responsibility and deployment risk remain with integrators.
