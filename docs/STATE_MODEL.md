# AOXChain State Model

This document defines state classes, authority boundaries, and lifecycle expectations.

## 1) Canonical state

**Definition:** consensus-authoritative chain state derived by deterministic transitions.

- **Primary authority:** kernel (`aoxcore`, `aoxcunity`) with deterministic runtime coupling.
- **Lifecycle:** created/updated by valid ordered execution; immutable history by block height/hash semantics.
- **Current state:** core models and consensus structures exist.
- **Gap:** full formalized state migration/recovery proofs across all upgrade scenarios are partially implemented.

## 2) Ephemeral state

**Definition:** transient in-memory process state (queues, sessions, temporary buffers).

- **Primary authority:** service and operator processes.
- **Lifecycle:** process-bounded; not authoritative for consensus.
- **Risk:** stale/partial data must not be treated as canonical truth.

## 3) Mempool state

**Definition:** pre-consensus pending transaction pool and admission metadata.

- **Primary authority:** kernel admission rules + consensus scheduling policy.
- **Lifecycle:** transactions enter, are evicted, included, or rejected.
- **Current state:** mempool modules are present in `aoxcore`; service/operator ingress can feed mempool pathways.

## 4) Execution cache state

**Definition:** runtime-local execution artifacts used for performance and lane orchestration.

- **Primary authority:** runtime crates (`aoxcexec`, `aoxcvm`) under deterministic constraints.
- **Lifecycle:** cacheable/evictable; must not change canonical outcomes.
- **Rule:** cache miss/hit may change latency, never consensus result.

## 5) Snapshot state

**Definition:** point-in-time exported/imported state images for recovery/bootstrap.

- **Primary authority:** system services (`aoxcdata` and associated operator workflows).
- **Lifecycle:** generated, validated, distributed, restored.
- **Current state:** persistence/sync surfaces exist; full recovery evidence package is partially implemented.

## 6) Archival state

**Definition:** long-horizon retained chain/event/history data for audit and compliance.

- **Primary authority:** data and operator service processes.
- **Lifecycle:** append, verify integrity, compact/index, preserve provenance.
- **Gap:** explicit retention policy with release-bound guarantees is not yet evidenced at root level.

## 7) Governance / identity / configuration state

### Governance and constitutional state
- Located across consensus and policy surfaces (`aoxcunity`, `aoxcore`, policy-related crates).
- Must remain deterministic where governance decisions alter canonical rules.

### Identity/key state
- Managed in identity/operator tool crates (`aoxcore::identity`, `aoxckit`, `aoxcmd::keys`).
- Requires strict custody, rotation, and revocation lifecycle controls.

### Configuration state
- Typed config models in `aoxconfig`; runtime/operator load paths in service and command crates.
- Non-deterministic operator config inputs must be normalized before affecting deterministic paths.

## 8) Authority and lifecycle summary table

| State class | Authority | Deterministic requirement | Lifecycle status |
|---|---|---|---|
| Canonical chain state | Kernel + deterministic runtime | strict | currently implemented |
| Ephemeral process state | Services/operators | no (must stay outside consensus authority) | currently implemented |
| Mempool state | Kernel admission + consensus scheduling | strict for admission rules | partially implemented evidence |
| Execution caches | Runtime | strict outcome equivalence | partially implemented evidence |
| Snapshots | Data/service/operator | deterministic validation required | partially implemented |
| Archive/history | Data/service/operator | integrity deterministic, query nondeterministic timing | partially implemented |
| Governance/identity/config | Kernel + operator controls | mixed by surface; canonical-impact paths must be deterministic | partially implemented |
