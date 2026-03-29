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
- **Ultra Command Center:** searchable command palette in the header for rapid route and anchor execution.

## Usage

From repository root:

```bash
cargo run -p aoxchub
```

For Dioxus development workflow:

```bash
dx serve --platform desktop
```

## CLI Binary (Advanced Compatibility)

AOXC Hub now exposes a compatibility-oriented CLI layer in the same `aoxchub` binary.

Examples:

```bash
aoxchub --profile real
aoxchub doctor --profile testnet --format json --headless
aoxchub paths --aoxcdata-home /srv/aoxc/.AOXCData
```

The CLI layer is designed for deterministic operator automation while preserving desktop launch behavior when no headless-only workflow is requested.

## Integration Checklist (UI-aligned)

1. Network profile selector (Dev/Testnet/Mainnet) with strict environment isolation.
2. RPC compatibility handshake (version, chain-id, genesis hash, capability checks).
3. Wallet policy and signature boundary enforcement.
4. Observability wiring (logs, health signals, operator diagnostics).
5. Release gate with build checks and smoke validation.

## Notes

- This crate remains under active integration; interface composition is complete, while deep business content and production connectors can be layered in subsequent iterations.
- The repository is MIT-licensed; operational responsibility and deployment risk remain with integrators.

## AOXCData Deep Integration Surface

The Operations route now includes a production-oriented integration matrix for:

- Binary mapping for `aoxc`, `aoxchub`, and `aoxckit` under `~/.AOXCData/bin`.
- Directory coverage for ledger, runtime index, IPFS cache, logs, and operator key locations.
- Make command control plane examples for build, test, release checks, and hub execution.
- CLI command surface examples aligned to real operator workflows.

This keeps AOXC Hub usable as a single visual command reference while deeper backend adapters continue to mature.


## Navigation Baseline (Production-Oriented)

AOXC Hub now uses a two-level menu model so operators can move fast without losing route context:

- **5 route menus:** Dashboard, Wallet, Operations, Overview, Settings.
- **9 quick-anchor menus:** Integration checklist, wallet setup, overview, dashboard metrics, validator matrix, bridge, governance, staking, ecosystem.

This structure keeps top-level navigation stable while preserving deep-link access to operational panels.

## Ultra Command Center

The header now includes an **Ultra Command Center** trigger that opens a searchable command palette.

Capabilities:

- Route-level execution: Landing, Dashboard, Wallet, Operations, Overview, and Settings.
- In-page anchor execution: integration checklist, validator matrix, and governance sections.
- Operator-focused discovery: each command includes category and execution intent text to reduce navigation latency.

This addition is intentionally deterministic and local-first; it does not depend on remote services and remains consistent across Desktop, Dev, Testnet, and Mainnet profiles.
