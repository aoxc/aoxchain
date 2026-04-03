# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is a deterministic Layer-1 engineering program focused on three primary outcomes:

1. a kernel-led chain with explicit trust and validation boundaries,
2. a special-purpose deterministic VM owned by the protocol,
3. a crypto-agile migration path to post-quantum security profiles.

> **Repository status (April 2, 2026):** Active development. Production-readiness is gated by evidence, not by intent.

## 1) Strategic Goals

### Goal A — Deterministic kernel correctness
- Preserve deterministic state transitions under all supported environments.
- Enforce fail-closed admission and settlement rules.
- Keep consensus-critical behavior isolated from non-deterministic surfaces.

### Goal B — AOXChain-owned VM
- Build and maintain a protocol-governed VM execution surface.
- Enforce deterministic gas/metering and bounded resource behavior.
- Provide explicit VM admission, opcode policy, and syscall validation controls.

### Goal C — Quantum-resilient protocol evolution
- Introduce versioned cryptographic profiles for signatures, key exchange, and validation flows.
- Support hybrid migration (classical + post-quantum) before legacy deprecation.
- Require auditable artifact evidence for every crypto-profile transition.

## 2) Primary Repository Surfaces

| Path | Purpose |
|---|---|
| `crates/` | Protocol/kernel/VM/network/service implementation crates. |
| `configs/` | Runtime and network profile definitions. |
| `scripts/` | Validation, audit, and evidence automation. |
| `tests/` | Integration, adversarial, and readiness validation suites. |
| `artifacts/` | Generated readiness and release evidence outputs. |
| `docs/` | mdBook and operator-facing documentation surfaces. |

## 3) Canonical Governance and Technical Documents

- `READ.md` — canonical technical reference and execution contract.
- `SCOPE.md` — in-scope/out-of-scope and compatibility posture.
- `ARCHITECTURE.md` — component boundaries, dependency direction, trust surfaces.
- `SECURITY.md` — reporting and security handling expectations.
- `TESTING.md` — validation policy and evidence requirements.
- `NETWORK_SECURITY_ARCHITECTURE.md` — network/RPC trust segmentation, DDoS controls, and kernel hardening baseline.
- `ROADMAP.md` — repository execution roadmap.
- `QUANTUM_ROADMAP.md` — quantum-resilience transformation plan.
- `QUANTUM_CHECKLIST.md` — release-gating checklist for crypto-profile migration.
- `docs/PRODUCTION_IMPLEMENTATION_BLUEPRINT.md` — complete production delivery gate matrix for kernel, VM, API/CLI, and crypto-agility execution.

## 4) Operator and Engineering Command Baseline

```bash
make help
make build
make test
make quality
make audit
make quantum-readiness-gate
make quantum-full
```

For environment lifecycle operations:

```bash
make runtime-source-check AOXC_NETWORK_KIND=devnet
make runtime-install AOXC_NETWORK_KIND=devnet
make runtime-verify AOXC_NETWORK_KIND=devnet
make runtime-activate AOXC_NETWORK_KIND=devnet
make runtime-status AOXC_NETWORK_KIND=devnet
```

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
- **Operations:** `ops-prepare`, `ops-start`, `ops-stop`, `ops-restart`, `ops-logs`, `ops-flow`.
- **Guided workflows:** `demo`, `localnet`, `devnet`, `testnet`, `doctor`, `audit-chain`.
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
- `QUANTUM_ROADMAP.md`: phased post-quantum transformation program (block structure, VM, network, governance, AI-assisted operations).
- `QUANTUM_CHECKLIST.md`: release-gating checklist for quantum-resilience execution and evidence.

mdBook entry points:
- `docs/src/README.md`
- `docs/src/SYSTEM_STATUS.md`
- `docs/src/AI_TRAINING_AND_AUDIT_GUIDE.md`

A change is considered high-risk and requires explicit documentation updates when it affects:
- consensus/finality behavior,
- VM execution semantics,
- cryptographic profile behavior,
- persistence or serialization format,
- external API or operator procedures.

No “ready” claim is valid without reproducible commands and retained artifacts.

## 6) License and Liability Context

AOXChain is distributed under the [MIT License](./LICENSE). Repository materials are provided on an **"as is"** basis without warranties or liability assumptions by maintainers or contributors.
