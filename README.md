# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is a deterministic Layer-1 engineering program focused on production-grade operations, cryptographic agility, and evidence-backed readiness.

## Program Direction

AOXChain follows a two-stage strategy:

1. **Production-Grade Testnet** — build and operate testnet with mainnet discipline.
2. **PQ-Resilient Mainnet** — activate mainnet only after cryptographic, operational, and rollback controls are proven.

This repository intentionally avoids unverifiable claims such as “unbreakable.”

## Current Posture

- Active engineering and validation program.
- Readiness claims are valid only with reproducible evidence.
- Repository contents are provided under MIT on an “as is” basis.

## Repository Layout

| Path | Purpose |
|---|---|
| `crates/` | Protocol, kernel, VM, network, service, and operator crates. |
| `configs/` | Runtime and network profile definitions. |
| `tests/` | Integration and adversarial validation suites. |
| `scripts/` | Automation and evidence workflows. |
| `docs/` | Technical and operational documentation surfaces. |
| `artifacts/` | Generated evidence and release/readiness artifacts. |

## Canonical Documents

- `READ.md` — repository-level technical contract and invariants.
- `ROADMAP.md` — current strategic roadmap and phase gates.
- `SCOPE.md` — in-scope/out-of-scope and compatibility posture.
- `ARCHITECTURE.md` — component boundaries and dependency direction.
- `SECURITY.md` — security posture and disclosure model.
- `TESTING.md` — validation policy and readiness gates.
- `NETWORK_SECURITY_ARCHITECTURE.md` — node, RPC, and host hardening model.
- `WHITEPAPER.md` — protocol-level architecture narrative.

## Baseline Commands

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

## Compatibility and Change Discipline

Breaking changes may be accepted for determinism, safety, or architectural integrity. Compatibility-impacting changes must include explicit rationale, migration guidance when required, and synchronized documentation updates.

## License

AOXChain is distributed under the [MIT License](./LICENSE), provided **"as is"** without warranties except where prohibited by applicable law.
