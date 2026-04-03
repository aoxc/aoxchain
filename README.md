# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is a deterministic Layer-1 engineering program focused on:

1. kernel-owned consensus and settlement policy,
2. protocol-governed deterministic VM execution,
3. cryptographic agility with staged post-quantum migration controls.

---

## Repository Status (April 3, 2026)

> **Current posture:** active development, testnet-readiness by gate evidence, not a production warranty.
>
> **Critical warning:** AOXChain remains experimental software.  
> No repository state alone should be interpreted as a guarantee of production fitness, uninterrupted operation, legal compliance, or financial suitability.

If you declare readiness, use retained command evidence and explicit gate outputs.

---

## Why AOXChain

AOXChain is designed for environments where deterministic behavior and auditable boundaries are non-negotiable:

- deterministic execution and replay consistency,
- fail-closed validation at ingress and policy boundaries,
- kernel-first interpretation of trust/finality,
- explicit cryptographic profile governance (including PQ transition paths),
- release decisions backed by reproducible artifacts.

---

## Is AOXChain a Quantum-Targeting Chain?

**Yes—by roadmap and architecture intent.**  
AOXChain targets quantum-resilience via profile-versioned cryptography, hybrid migration windows, and rollback-safe governance flows rather than unverifiable “absolute security” claims.

Primary references:

- `QUANTUM_ROADMAP.md`
- `QUANTUM_CHECKLIST.md`
- `WHITEPAPER.md`

---

## Repository Layout

| Path | Purpose |
|---|---|
| `crates/` | Protocol, kernel, VM, network, service, and operator crates. |
| `configs/` | Runtime/network profile definitions and environment bundles. |
| `tests/` | Integration, adversarial, and production-readiness validation suites. |
| `scripts/` | Automation for quality/readiness/audit evidence workflows. |
| `docs/` | mdBook + operator-facing technical and governance documents. |
| `artifacts/` | Generated evidence and release/readiness artifacts. |

---

## Canonical Documents

- `READ.md` — canonical technical reference and execution contract.
- `WHITEPAPER.md` — end-to-end protocol architecture, trust model, and production closure narrative.
- `SCOPE.md` — in-scope/out-of-scope and compatibility posture.
- `ARCHITECTURE.md` — component topology, flow, and dependency direction.
- `SECURITY.md` — private disclosure and security handling model.
- `TESTING.md` — mandatory validation policy and gate criteria.
- `NETWORK_SECURITY_ARCHITECTURE.md` — validator/sentry/RPC trust segmentation baseline.
- `docs/ADVANCED_NODE_ROLE_BLUEPRINT.md` — full multi-role, multi-plane topology and staged activation model.
- `ROADMAP.md` — program roadmap.
- `QUANTUM_ROADMAP.md` + `QUANTUM_CHECKLIST.md` — cryptographic migration execution/gates.
- `docs/PRODUCTION_IMPLEMENTATION_BLUEPRINT.md` — production implementation and closure matrix.
- `docs/PRODUCTION_READINESS_CHECKLIST.md` — auditable production go/no-go checklist and required evidence map.
- `docs/OS_COMPATIBILITY.md` — cross-OS and Docker compatibility contract.

---

## Engineering and Operator Baseline Commands

Quality and readiness:

```bash
make help
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
make quantum-readiness-gate
make quantum-full
```

Runtime lifecycle:

```bash
make runtime-source-check AOXC_NETWORK_KIND=<env>
make runtime-install AOXC_NETWORK_KIND=<env>
make runtime-verify AOXC_NETWORK_KIND=<env>
make runtime-activate AOXC_NETWORK_KIND=<env>
make runtime-status AOXC_NETWORK_KIND=<env>
```

---

## Readiness and Promotion Policy (Short Form)

AOXChain should be treated as ready for a target environment only when required gates pass and evidence is retained.

For testnet go/no-go decisions, use:

- `make testnet-gate`
- `make testnet-readiness-gate`
- `cargo test -p tests`

If any required gate fails, status is **NOT_READY** until remediation and revalidation are complete.

---

## Compatibility and Change Discipline

Breaking changes may be accepted when required for determinism, safety, or architectural integrity, but they must be:

- explicitly declared,
- documented with impact rationale,
- accompanied by relevant validation evidence.

Architecture-sensitive changes (consensus, execution semantics, crypto policy, storage format, API behavior, operator controls) require synchronized updates to canonical docs.

---

## Experimental and Liability Notice

AOXChain is distributed under the [MIT License](./LICENSE) on an **"as is"** basis.  
Maintainers and contributors provide no warranty of correctness, availability, merchantability, fitness for a particular purpose, or regulatory/compliance suitability, except where prohibited by applicable law.

Do not interpret testnet readiness as production guarantee.
