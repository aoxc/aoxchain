# AOXChain Architecture Model

This document defines the repository architecture using an operating-system-like discipline model: kernel, system services, runtimes, operator environment, and peripheral surfaces.

## 1) Layer model (current normalized view)

### Kernel (consensus-critical deterministic state transition)
**Primary crates:** `aoxcore`, `aoxcunity`.

- **Currently implemented:** deterministic domain models (blocks, transactions, receipts, genesis, mempool), consensus state machine surfaces, quorum/finality/safety modules.  
- **Partially implemented:** full end-to-end multi-node evidence for all adversarial conditions is referenced in planning docs, but complete repository-local proof bundles are not uniformly present.  
- **Target state:** complete consensus replay corpus and recovery proofs across version transitions.

### Runtimes (deterministic execution and lane orchestration)
**Primary crates:** `aoxcexec`, `aoxcvm`, `aoxcenergy`.

- **Currently implemented:** lane policy objects, deterministic hashing/accounting primitives, multi-lane dispatch modules, execution host interfaces, cost/economic primitives.  
- **Partially implemented:** runtime integration evidence across all supported lanes (native/EVM/WASM/external) is not yet complete in one reproducible root-level verification set.  
- **Target state:** strict replay-stable cross-lane conformance suite and formal malformed-input corpus.

### System Services (networking, RPC, configuration, persistence)
**Primary crates:** `aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`, `aoxclibs`, `aoxchal`.

- **Currently implemented:** p2p/gossip/sync modules, HTTP/gRPC/WebSocket RPC modules, data/index stores, typed config validation, shared utility crates.  
- **Partially implemented:** complete partition-healing, snapshot lifecycle, and multi-backend durability evidence are not uniformly documented as release-grade proofs.  
- **Target state:** standardized service-level SLO evidence package (availability, recovery, persistence integrity).

### Operator Environment (control plane)
**Primary crates:** `aoxcmd`, `aoxckit`, `aoxchub`.

- **Currently implemented:** CLI bootstrap/db/runtime/diagnostics commands, key-management toolkit commands, desktop Tauri control UI shell.  
- **Partially implemented:** full command-to-control-path evidence and policy guard verification for every mutating operator action is not yet centrally tracked.  
- **Target state:** immutable audit export pipeline linking every mutating operation to signed evidence artifacts.

### Applications / Adapters / Peripheral
**Primary crates:** `aoxcsdk`, `aoxcontract`, `aoxcai`, `aoxcmob`.

- **Currently implemented:** SDK builders, contract descriptor/manifest model, AI advisory extension plane, mobile gateway/session/security structures.  
- **Partially implemented:** ecosystem-facing compatibility guarantees and stable external versioning contracts remain in-progress.  
- **Target state:** long-lived compatibility policy and ecosystem deprecation governance.

## 2) Kernel boundary

The kernel boundary is the deterministic state-transition authority. In this repository that boundary is centered on `aoxcore` and `aoxcunity`, with execution outputs admitted via deterministic runtime boundaries (`aoxcexec`/`aoxcvm`) rather than operator UI paths.

**Boundary rule:** `aoxchub` (desktop UI) and other operator surfaces are not protocol authority and must not become hidden consensus paths.

## 3) Deterministic vs non-deterministic boundaries

### Deterministic boundary (inside consensus correctness)
- canonical block/transaction/receipt/state models (`aoxcore`),
- consensus voting/finality/safety transitions (`aoxcunity`),
- execution policy resolution and receipt semantics expected to be replay-stable (`aoxcexec`, deterministic subsets of `aoxcvm`),
- canonical serialization/hashing boundaries (`aoxclibs`, selected runtime modules).

### Non-deterministic boundary (outside direct consensus authority)
- p2p transport timing and peer availability (`aoxcnet`),
- RPC ingress patterns and client behavior (`aoxcrpc`),
- operator command timing/human decisions (`aoxcmd`, `aoxchub`, `aoxckit`),
- AI advisory outputs (`aoxcai`),
- external mobile/network adapter behavior (`aoxcmob`).

**Normalization requirement:** non-deterministic inputs must be validated and normalized before influencing deterministic execution.

## 4) Crate map by architecture domain (primary assignment)

| Crate | Primary domain | Secondary concern(s) | Current-state confidence |
|---|---|---|---|
| `aoxcore` | Kernel | runtime interfaces | currently implemented |
| `aoxcunity` | Kernel | persistence interfaces | currently implemented |
| `aoxcexec` | Runtimes | kernel integration | partially implemented |
| `aoxcvm` | Runtimes | system bridge | partially implemented |
| `aoxcenergy` | Runtimes | policy/economics | partially implemented |
| `aoxcnet` | System Services | operator telemetry | partially implemented |
| `aoxcrpc` | System Services | operator ingress | partially implemented |
| `aoxcdata` | System Services | kernel state persistence | partially implemented |
| `aoxconfig` | System Services | operator controls | currently implemented |
| `aoxclibs` | System Services | deterministic utility base | currently implemented |
| `aoxchal` | System Services | deployment optimization | not yet evidenced for critical paths |
| `aoxcmd` | Operator Environment | service orchestration | currently implemented |
| `aoxckit` | Operator Environment | key lifecycle | partially implemented |
| `aoxchub` | Operator Environment | observability UX | partially implemented |
| `aoxcsdk` | Applications/Peripheral | contract integration | currently implemented |
| `aoxcontract` | Applications/Peripheral | runtime metadata feed | currently implemented |
| `aoxcai` | Applications/Peripheral | operator advisory | partially implemented |
| `aoxcmob` | Applications/Peripheral | remote operator adapter | partially implemented |

## 5) Current-state vs target-state summary

### Currently implemented (directly evidenced by repository code layout)
- multi-crate decomposition across kernel, runtime, services, operator plane, and adapters,
- explicit consensus and execution crates,
- explicit operator CLI and desktop surfaces,
- explicit docs/runbooks/readiness artifacts.

### Partially implemented (evidence exists but is incomplete)
- end-to-end replay verification across all runtime lanes,
- comprehensive partition/recovery evidence,
- complete control-plane command-to-evidence mapping,
- complete ecosystem compatibility policy guarantees.

### Target state
- formal release-gated architecture checks that fail build/release on boundary violations,
- durable evidence bundles for deterministic replay, snapshot recovery, and multi-node fault scenarios,
- explicit API and runtime compatibility lifecycle commitments.

### Not yet evidenced
- formal proof that every operator mutating action is always mapped to one audited backend control path,
- repository-wide formal model checks for all consensus invariants,
- uniformly packaged disaster-recovery exercises across storage backends.

## 6) Known gaps

1. A single root architecture truth document did not previously exist; architecture assertions were distributed across many files with mixed maturity language.
2. The root README used “100% readiness” wording that could be read as production-completeness rather than scoped baseline completion.
3. Release criticality tiers were not previously normalized at crate level.
4. Security, execution, and state boundaries were documented in fragments rather than one explicit root model set.
5. License messaging was permissive-MIT oriented and not aligned with stronger reciprocity goals for sovereign protocol components.
