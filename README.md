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
- `ROADMAP.md` — repository execution roadmap.
- `QUANTUM_ROADMAP.md` — quantum-resilience transformation plan.
- `QUANTUM_CHECKLIST.md` — release-gating checklist for crypto-profile migration.

## 4) Operator and Engineering Command Baseline

```bash
make help
make build
make test
make quality
make audit
```

For environment lifecycle operations:

```bash
make runtime-source-check AOXC_NETWORK_KIND=devnet
make runtime-install AOXC_NETWORK_KIND=devnet
make runtime-verify AOXC_NETWORK_KIND=devnet
make runtime-activate AOXC_NETWORK_KIND=devnet
make runtime-status AOXC_NETWORK_KIND=devnet
```

## 5) Decision Discipline

A change is considered high-risk and requires explicit documentation updates when it affects:
- consensus/finality behavior,
- VM execution semantics,
- cryptographic profile behavior,
- persistence or serialization format,
- external API or operator procedures.

No “ready” claim is valid without reproducible commands and retained artifacts.

## 6) License and Liability Context

AOXChain is distributed under the [MIT License](./LICENSE). Repository materials are provided on an **"as is"** basis without warranties or liability assumptions by maintainers or contributors.
