# AOXChain — Canonical Technical Definition

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="220" />
</p>

> **Status:** Engineering Mainnet Program  
> **System Class:** Deterministic L1 + Multi-VM Execution + Operator-Grade Control Plane  
> **Scope:** This document is the canonical, repository-level technical definition of AOXChain.

---

## 1. System identity and mission

AOXChain is a modular blockchain architecture designed around deterministic state transitions, auditable execution, and production-grade operations. The system separates consensus-critical logic from execution routing, network services, and operator tooling in order to preserve safety boundaries and simplify verifiable evolution.

Primary objectives:

1. **Safety-preserving finality** under explicit consensus rules.
2. **Deterministic replay** for identical canonical inputs.
3. **Operational measurability** via reproducible runbooks and evidence trails.
4. **Controlled protocol evolution** through versioned policy and activation discipline.

---

## 2. Layered architecture (single source of truth)

### 2.1 Kernel (consensus-critical domain)

- `crates/aoxcore`: canonical primitives for blocks, transactions, receipts, identity objects, and state-transition structure.
- `crates/aoxcunity`: consensus/finality/voting/quorum/safety mechanisms.

**Invariant K-1:** Only kernel decisions are authorized to mutate canonical chain state.

### 2.2 Execution plane (VM and lane orchestration)

- `crates/aoxcexec`: deterministic execution policies, lane envelope handling, and accounting constraints.
- `crates/aoxcvm`: multi-lane dispatch and compatibility paths (native, EVM, WASM, and other lanes).
- `crates/aoxcenergy`: gas and economic metering rules.

**Invariant E-1:** Canonical input equality must imply canonical output equality.

### 2.3 System services

- `crates/aoxcnet`: P2P, gossip, discovery, synchronization, resilience workflows.
- `crates/aoxcrpc`: external API surfaces (HTTP, gRPC, WebSocket).
- `crates/aoxcdata`: persistence/indexing/state-storage facilities.
- `crates/aoxconfig`: type-safe configuration and validation mechanisms.

### 2.4 Operator plane

- `crates/aoxcmd`: node and lifecycle operations via CLI.
- `crates/aoxckit`: keying and cryptographic operational toolchain.
- `crates/aoxchub`: desktop control-plane interface.
- `scripts/`: operational automation, readiness gates, and release evidence workflows.

**Invariant O-1:** Operator and UX surfaces are control/observability interfaces, not consensus authorities.

---

## 3. Deterministic execution contract

AOXChain follows a fail-closed deterministic contract:

1. Non-deterministic inputs cannot affect kernel state unless normalized.
2. Malformed or policy-invalid payloads are rejected before state mutation.
3. Policy changes are versioned and activation-scoped.
4. Release claims require commit-bound evidence artifacts.
5. Ambiguous states default to fail-closed behavior.

This contract is the baseline for replay stability, auditability, and environment parity.

---

## 4. Environment model and network parity

The environment surface under `configs/` defines reproducible network contexts:

- `localnet`
- `devnet`
- `testnet`
- `validation`
- `mainnet`
- `sovereign` templates

Each environment carries versioned identity and policy material (`genesis`, `validators`, `profile`, `release-policy`, and auxiliary metadata where relevant).

**Environment principle:** Cross-environment differences must be explicit, reviewable, and documented; hidden assumptions are disallowed.

---

## 5. Mainnet-readiness quality gates

A release candidate is not considered mainnet-eligible unless the following are satisfied:

1. Deterministic replay validation across supported execution lanes.
2. Consensus and network resilience scenarios are executed and documented.
3. Snapshot/restore integrity is demonstrated.
4. API/config compatibility impact is explicitly stated.
5. Security and operations runbooks are current.
6. Release artifact chain is complete (hashes, SBOM, provenance, signatures, and audit output where required).

**Maturity statement:** AOXChain does not claim “absolute defect-free security.” It claims measurable assurance based on explicit gates and evidence.

---

## 6. Security and key lifecycle principles

1. Key lifecycle operations (generation, storage, rotation, revocation) must remain auditable.
2. Consensus-critical changes require explicit risk annotation in review flow.
3. Incident response is runbook-driven and evidence-linked.
4. Operator action → evidence mapping must remain intact across tooling.

---

## 7. Engineering and documentation governance

- Prefer minimal, testable, clearly scoped changes.
- Update code and documentation together.
- State backward-compatibility implications explicitly.
- Subdirectory `READ.md`/`README.md` files define *what a scope does*; they do not define roadmap authority.

---

## 8. Repository navigation

- Top-level portal: `README.md`
- Architecture reference: `docs/ARCHITECTURE.md`
- Execution model: `docs/EXECUTION_MODEL.md`
- State model: `docs/STATE_MODEL.md`
- Security model: `docs/SECURITY_MODEL.md`
- System invariants: `docs/SYSTEM_INVARIANTS.md`
- Environment catalog: `configs/environments/`
- Operations scripts: `scripts/`

---

## 9. Compliance note

This repository is an engineering codebase under active development. Statements in this document are technical definitions and process commitments, not legal or investment representations.
