# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is an experimental, modular Layer-1 engineering workspace focused on deterministic execution, auditable operations, and evidence-governed release readiness.

> **Repository status (March 30, 2026):** Active development. Interfaces, procedures, and compatibility guarantees can change while controls mature.

## 1) Purpose

This repository is used to design and validate:
- deterministic state transition and execution behavior,
- consensus and networking safety boundaries,
- operator-grade runtime controls,
- audit-ready evidence for release and production-closure decisions.

## 2) Quick start (Make-first workflow)

### Prerequisites
- Rust toolchain (`cargo`, `rustfmt`, `clippy`)
- GNU Make
- Bash
- Git
- `sha256sum`, `tar`

### Basic flow
```bash
make help
make build
make test
make quality
```

### Runtime lifecycle flow
```bash
make runtime-source-check
make runtime-install AOXC_NETWORK_KIND=devnet
make runtime-verify
make runtime-activate
make runtime-status
```

### Operator flow
```bash
make ops-help
make ops-prepare
make ops-start
make ops-status
```

### One-command four-node full layout (including snapshots)
```bash
make aoxc-full-4nodes
```

Optional Docker asset generation:
```bash
make aoxc-full-4nodes-docker
```

This flow provisions a hardened multi-node directory tree, copies canonical environment/genesis materials, bootstraps four nodes, executes deterministic rounds, and writes compressed snapshots plus an audit report under the generated root path.

## 3) How the Make system is organized

The root `Makefile` is a single-runtime operator surface. It intentionally avoids environment fan-out and exposes auditable targets for:
- build and packaging,
- runtime installation and activation,
- health and policy verification,
- daemon lifecycle operations,
- evidence and audit trace generation.

Key target groups:
- **Engineering quality:** `build`, `test`, `check`, `fmt`, `clippy`, `quality`, `ci`.
- **Runtime management:** `runtime-install`, `runtime-verify`, `runtime-activate`, `runtime-status`, `runtime-doctor`, `runtime-reset`.
- **Full local system provisioning:** `aoxc-full-4nodes`, `aoxc-full-4nodes-docker`.
- **Operations:** `ops-prepare`, `ops-start`, `ops-stop`, `ops-restart`, `ops-logs`, `ops-flow`.
- **Release/evidence:** `package-*`, `publish-release`, `audit`, `db-*`.

## 4) Identity and key management baseline

AOXChain identity derivation uses a canonical BIP44-style HD path envelope:

- **Purpose:** `44`
- **AOXC coin type:** `2626`
- **Canonical path format:** `m/44/2626/<chain>/<role>/<zone>/<index>`

Important implementation details:
- Canonical persisted path components are constrained to the 31-bit unhardened range (`0 ..= 0x7FFF_FFFF`).
- Hardened behavior is represented as a projection helper where needed, not by storing hardened markers in canonical path text.
- Deterministic entropy derivation is domain-separated and combines master-seed + canonical path fields.
- Node key bundles enforce role-path consistency and fingerprint validation for auditability.

If you are asking whether the system is aligned to `m/44'/2626'`: AOXChain follows the BIP44 field semantics (`44`, `2626`) but stores canonical textual paths in unhardened numeric form (`m/44/2626/...`) as an explicit policy decision.

## 5) Repository map

| Path | Role |
|---|---|
| `crates/` | Rust workspace crates (kernel, execution, networking, RPC, SDK, operator tooling). |
| `configs/` | Environment/runtime profiles and deterministic network definitions. |
| `docs/` | mdBook-backed governance and technical documentation. |
| `models/` | Readiness, risk, and profile models used by checks and audits. |
| `scripts/` | Automation for quality gates and evidence generation. |
| `tests/` | Integration and production-readiness validation suite. |
| `artifacts/` | Generated release-evidence and production-closure bundles. |
| `contracts/` | Contract surface and deployment matrix inputs. |

## 6) Documentation baseline

- `README.md`: repository purpose, setup, and operational entry points.
- `READ.md`: canonical technical definitions and invariants.
- `SCOPE.md`: in-scope and out-of-scope boundaries, compatibility policy.
- `ARCHITECTURE.md`: component boundaries and data/control flow.
- `SECURITY.md`: reporting and security coordination expectations.
- `TESTING.md`: required validation layers and commands.

mdBook entry points:
- `docs/src/README.md`
- `docs/src/SYSTEM_STATUS.md`
- `docs/src/AI_TRAINING_AND_AUDIT_GUIDE.md`

## 7) License and liability context

AOXChain is distributed under the [MIT License](./LICENSE). Repository materials are provided on an "as is" basis without warranties or liability assumptions by maintainers or contributors.

Nothing in this repository constitutes legal, financial, or production-readiness certification.
