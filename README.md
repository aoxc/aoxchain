# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is a deterministic Layer-1 engineering program focused on production-grade operation, cryptographic agility, and evidence-backed readiness.

## Program Direction

AOXChain follows a two-stage strategy:

1. **Production-Grade Testnet** — operate testnet with mainnet discipline.
2. **PQ-Resilient Mainnet** — activate mainnet only after cryptographic and operational controls are proven.

The repository intentionally avoids unverifiable claims.

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
- `SCOPE.md` — in-scope/out-of-scope and compatibility posture.
- `ARCHITECTURE.md` — component boundaries and dependency direction.
- `SECURITY.md` — security posture and disclosure model.
- `TESTING.md` — validation policy and readiness gates.
- `ROADMAP.md` — strategic roadmap and phase gates.
- `docs/NAMING_VERSIONING_SIMPLIFICATION_PLAN.md` — naming/versioning baseline and migration policy.
- `docs/GENESIS_IDENTITY_CHECKLIST.md` — genesis and environment identity consistency checklist.

## Identity and Versioning Quick Reference

Use this vocabulary consistently across code, docs, and operations:

| Term | Meaning | Authority |
|---|---|---|
| **Brand** | Product/system name (`AOXChain`) | repository documentation |
| **Ticker** | Native asset symbol (`AOXC`) | protocol/economic docs |
| **Release line** | Human-facing release stream label (for example `AOXC-QTR-V1`) | release notes + tags |
| **Workspace version** | Build/package/release metadata version | `configs/version-policy.toml` |
| **Chain ID** | Deterministic machine identity (numeric) | `configs/registry/network-registry.toml` |
| **Network ID** | Human-readable network identity string | `configs/registry/network-registry.toml` |
| **Crypto profile** | Consensus-visible cryptography mode/version | topology and profile policy |

Rules:

1. Do not use release-line labels as `chain_id` or `network_id`.
2. Do not derive protocol truth from Git tags alone.
3. Keep machine identity policy in repository-controlled, reviewable files.

## Baseline Engineering Commands

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

## Container Runtime (Docker + Podman)

AOXChain container surfaces are maintained to run on both Docker and Podman.

### Build image

```bash
docker build -t aoxchain-node:local .
# or
podman build -t aoxchain-node:local .
```

### Run a single node

```bash
docker run --rm -p 26656:26656 -p 8545:8545 aoxchain-node:local
# or
podman run --rm -p 26656:26656 -p 8545:8545 aoxchain-node:local
```

### Run local multi-node topology

```bash
docker compose up --build
# or
podman compose up --build
```

Additional Podman notes are documented in `PODMAN.md`.

## Compatibility and Change Discipline

Compatibility-impacting changes must include explicit rationale, migration guidance when required, and synchronized documentation updates.

## License

AOXChain is distributed under the [MIT License](./LICENSE), provided **"as is"** without warranties except where prohibited by applicable law.
