<div align="center">
  <a href="https://github.com/aoxc/aoxcore">
    <img src="logos/aoxc_transparent.png" alt="AOXChain Logo" width="180" />
  </a>

# AOXChain
### Experimental Sovereign Coordination Chain
#### AOXC Alpha: Genesis V1

![Status](https://img.shields.io/badge/status-experimental-orange)
![Model](https://img.shields.io/badge/architecture-sovereign--core-purple)
![Stack](https://img.shields.io/badge/stack-rust-orange)
![CLI](https://img.shields.io/badge/tooling-aoxcmd-blue)

</div>

AOXChain is an experimental Rust blockchain workspace built around one core idea:

> **the local chain is the sovereign constitutional core, and remote systems are execution domains.**

This repository should be read as a **new chain project**. It is not positioned as a wrapper around another network, and this README intentionally describes AOXChain on its own terms.

---

## 1. What AOXChain is

AOXChain is designed to own the parts of a system that must remain canonical:

- identity,
- supply,
- governance,
- relay authorization,
- validator/security policy,
- settlement finality,
- treasury and reserves.

Remote domains may execute logic, hold contract adapters, and provide ecosystem-specific integrations, but the **final constitutional authority** stays on AOXChain.

---

## 2. Current architecture in one sentence

- **Local chain:** sovereign constitutional core.
- **Remote chains/domains:** execution, integration, liquidity, and application surfaces.

If you want the machine-readable view, run:

```bash
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
```

---

## 3. Canonical local roots

AOXChain currently models the following local constitutional roots:

1. `identity`
2. `supply`
3. `governance`
4. `relay`
5. `security`
6. `settlement`
7. `treasury`

These can be inspected from the CLI:

```bash
cargo run -p aoxcmd -- sovereign-core
```

---

## 4. Address and key derivation format

AOXChain uses a BIP44-style derivation prefix centered on the AOXC coin type.

### Canonical HD path

```text
m/44/2626/<chain>/<role>/<zone>/<index>
```

Example:

```text
m/44/2626/1/1/2/0
```

Meaning:

- `44` -> BIP44 purpose
- `2626` -> AOXC coin type / chain identity namespace
- `chain` -> chain identifier
- `role` -> actor role
- `zone` -> logical or geographic zone
- `index` -> sequential key index

This path model is implemented in the AOXC identity layer and should be treated as the canonical derivation format for operator and system key material.

---

## 5. Workspace layout

| Layer | Crate(s) | Responsibility |
|---|---|---|
| Protocol | `aoxcore` | identity, protocol primitives, genesis, tx, receipts |
| Consensus | `aoxcunity` | rounds, quorum, vote/finality state |
| Networking | `aoxcnet` | transport, discovery, gossip, sync |
| RPC / Ingress | `aoxcrpc` | HTTP, gRPC, WebSocket, security middleware |
| Execution | `aoxcvm` | multi-lane runtime and compatibility layers |
| Operations | `aoxcmd`, `aoxckit` | bootstrap, runtime ops, manifests, policy commands |

---

## 6. 10-minute operator onboarding

For the fastest operator-oriented setup path, start here:

- [`docs/ONBOARDING_10_MINUTES.md`](docs/ONBOARDING_10_MINUTES.md)
- [`docs/ONCALL_RUNBOOK.md`](docs/ONCALL_RUNBOOK.md)
- [`docs/MAINNET_READINESS_CHECKLIST.md`](docs/MAINNET_READINESS_CHECKLIST.md)
- [`docs/INCIDENT_RESPONSE_DRILL.md`](docs/INCIDENT_RESPONSE_DRILL.md)

Recommended local validation flow:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
```

---

## 7. Mainnet-readiness operator guides

- **On-call / SRE runbook:** [`docs/ONCALL_RUNBOOK.md`](docs/ONCALL_RUNBOOK.md)
- **Incident response drill:** [`docs/INCIDENT_RESPONSE_DRILL.md`](docs/INCIDENT_RESPONSE_DRILL.md)
- **Mainnet checklist:** [`docs/MAINNET_READINESS_CHECKLIST.md`](docs/MAINNET_READINESS_CHECKLIST.md)
- **Readiness / audit context:** [`docs/AUDIT_READINESS_AND_OPERATIONS.md`](docs/AUDIT_READINESS_AND_OPERATIONS.md)

---

## 8. Existing technical references

Additional architecture and operational analysis documents remain under [`docs/`](docs/):

- `REPO_GAP_ANALIZI_TR.md`
- `GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`
- `REAL_CHAIN_RUNBOOK_TR.md`
- `REAL_NETWORK_VALIDATION_RUNBOOK_TR.md`
- `ADVANCED_PRODUCTION_SUGGESTIONS_TR.md`
- `V0_1_0_ALPHA_PRODUCTION_PLAN.md`

---

## 9. Status note

AOXChain is still experimental. The presence of runbooks and checklists in this repository should be read as **preparation material**, not as proof that a production mainnet is already ready for launch.
