# DEVELOPMENT_PLAN.md

> Scope: `crates/aoxcvm`
> Product posture: **Deterministic multi-lane settlement kernel**

## Goal Statement
AOXCVM should evolve into a deterministic multi-lane settlement kernel that allows heterogeneous execution environments to run in native semantics while settling under one canonical security, state, receipt, and finality model.

## Current Baseline (as of this revision)
The crate already has a useful baseline:
- `HostStateView` offers a shared storage/gas/event host surface for lanes.
- `LaneDescriptor` and `LaneRegistry` model pluggable lane execution.
- `DeterministicScheduler` provides deterministic lane ordering.
- `ExecutionReceipt` normalizes minimal execution output.

These are the right primitives, but they remain foundational. The next phases must transform them into a full settlement kernel rather than a multi-runtime container.

## Core Architecture Direction
1. **Deterministic execution contract** for every lane and host path.
2. **Unified settlement boundary** with atomic cross-lane commit semantics.
3. **Capability-gated host authority** per lane class.
4. **Cross-lane interoperability protocol** with replay-safe messaging.

---

## Phase 1 — Transactional host journal and canonical state model
Introduce a host transaction journal that separates execution-time mutation from settlement-time commit.

### Additions
- `host/journal.rs`
  - transaction lifecycle: `begin_transaction`, `checkpoint`, `rollback`, `commit`
  - per-lane write sets, delete sets, event buffers
  - conflict-set capture for merge validation
- host state partitioning
  - lane-local persistent state
  - shared settlement state
  - transient execution state

### Exit criteria
- no lane writes directly to persistent host state during execution,
- failed execution paths are rollback-clean,
- cross-lane commit is atomic at the settlement boundary.

## Phase 2 — Capability-based host security model
Replace ambient host authority with explicit lane capabilities.

### Additions
- `host/capabilities.rs`
  - `HostCapability` enum (`StorageRead`, `StorageWrite`, `EventEmit`, `CrossLaneCall`, `NativeCrypto`, `TokenMint`, `SystemHookAccess`)
  - `CapabilityProfile` attached to lane descriptors
  - enforcement guards in host operations

### Exit criteria
- every host action is capability-gated,
- lane descriptors declare capability profile explicitly,
- unauthorized host actions fail deterministically with auditable errors.

## Phase 3 — Canonical receipt and proof surface
Upgrade receipts from "result blob" to settlement artifacts.

### Additions
- `host/receipt/canonical.rs`
  - canonical status
  - gas used
  - state root delta metadata
  - emitted events
  - cross-lane messages
  - security/compliance flags
  - replay hash and execution trace hash

### Exit criteria
- all lane-specific outputs map into one canonical receipt schema,
- upstream layers can consume settlement output without lane-specific parsing,
- receipts are stable replay anchors.

## Phase 4 — Cross-lane messaging protocol
Add deterministic lane-to-lane intent passing and delayed settlement queues.

### Additions
- `hypervm/message_bus.rs`
  - deterministic message envelope
  - async receipt references
  - delayed settlement queue
  - replay protection and causal ordering rules

### Exit criteria
- lane interactions are explicit protocol events, not implicit host coupling,
- replayed messages are rejected deterministically,
- causal ordering is preserved across nodes.

## Phase 5 — Dependency-aware deterministic scheduler
Advance from static sort ordering to replayable dependency planning.

### Additions
- `hypervm/scheduler/deps.rs`
  - declared read/write sets
  - deterministic dependency graph construction
  - conflict-free parallel batches
  - bounded replayable execution plan and proof metadata

### Exit criteria
- parallel execution occurs only for conflict-free partitions,
- execution plan is deterministic and reproducible on every node,
- scheduler metadata is included in settlement evidence.

## Phase 6 — Determinism compliance contract
Formalize and enforce deterministic execution rules per lane.

### Additions
- forbidden nondeterminism policy (wall-clock time, ambient RNG, floating-point semantics unless canonically constrained),
- host-seeded deterministic RNG option,
- deterministic serialization requirements,
- bounded memory and bounded iteration constraints.

### Exit criteria
- each lane declares compliance against a shared determinism contract,
- determinism violations are surfaced as policy failures,
- replay tests verify identical outputs across environments.

## Phase 7 — Lane governance and classification
Elevate lanes from plugins to governed execution domains.

### Additions
- enrich `LaneDescriptor` with:
  - `trust_level`
  - `capability_profile`
  - `isolation_mode`
  - `state_scope`
  - `settlement_class`
  - `message_policy`
  - `upgrade_policy`
- governance admission/versioning checks for lanes.

### Exit criteria
- lane onboarding and upgrade paths are policy-controlled,
- risk posture is explicit at descriptor level,
- governance changes are auditable and reproducible.

## First 5 implementation priorities
1. Transactional host journal.
2. Capability-gated host security.
3. Canonical cross-lane receipt model.
4. Cross-lane message bus.
5. Dependency-aware deterministic scheduler.

These five changes should be treated as the minimum threshold for AOXCVM to qualify as a settlement kernel.

## Delivery constraints
- Preserve deterministic replay as a non-negotiable invariant.
- Keep host APIs explicit; avoid hidden global side effects.
- Ensure every new control path has tests for success, failure, rollback, and replay.
- Treat receipts as audit and settlement artifacts, not logging byproducts.

## Delivery model (single-pass vs phased)
Building all kernel features in one large pass is technically possible but operationally unsafe for this repository profile.

AOXCVM changes should be delivered in **small deterministic slices** because each slice modifies consensus-sensitive behavior (state transitions, host authority, replay boundaries, and receipts).

Required discipline:
- one invariant family per PR (journal, capability gate, receipt canonicalization, message bus, scheduler graph),
- mandatory replay tests and rollback tests per slice,
- no simultaneous refactor of host state + scheduler + receipt format in a single unrecoverable change set.

Practical answer:
- **Not recommended:** “tüm kodu tek seferde geliştirmek”.
- **Recommended:** phased kernel hardening with acceptance gates at each phase boundary.

## Recommended immediate next task
Implement **Phase 1** with an in-memory `HostJournal` prototype and deterministic tests covering:
- nested checkpoints,
- rollback behavior,
- cross-lane merge conflicts,
- atomic commit success/failure boundaries.
