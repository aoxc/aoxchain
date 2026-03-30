# AOXChain Core Execution Kernel (Sovereign VM)

## 1) Architecture design

AOXChain should run a **small sovereign execution kernel** that owns deterministic transaction lifecycle semantics end-to-end. The kernel normalizes input as a canonical envelope, validates deterministic preconditions, meters resources with a bounded fuel model, executes exactly one selected lane adapter, and applies state changes with transactional commit/revert semantics.

Design principles:

- deterministic by construction (stable ordering, bounded integer arithmetic, explicit errors),
- minimal attack surface (small API and error model),
- replay consistency (same input + same state -> same output),
- compatibility without bloat (EVM/WASM are lane adapters, not core semantics).

## 2) Module layout

The Rust module is implemented in `crates/aoxcvm/src/kernel.rs` with clear boundaries:

- `CanonicalTxEnvelope`, `BlockExecutionContext`: canonical inputs,
- `FuelMeter`, `FuelSchedule`: deterministic accounting,
- `StateJournal`, `JournalOp`: transactional write-ahead recording,
- `KernelError`, `Receipt`, `Event`: deterministic observable output,
- `LaneAdapter`, `LaneRegistry`: lane-based dispatch abstraction,
- `CoreKernel`: orchestrator with commit/revert control plane.

## 3) Trait / interface model

- `HostState`: deterministic key-value host state abstraction.
- `LaneAdapter<S>`: lane executor contract (`execute`) over shared `ExecutionEnv`.
- `ExecutionEnv`: bounded host bridge for state read/write/delete, event emission, and fuel charging.

This keeps core ownership small while allowing isolated extension via compatibility lanes.

## 4) Execution flow

`CoreKernel::execute_tx` follows strict deterministic steps:

1. validate envelope,
2. initialize meter with `gas_limit`,
3. charge intrinsic gas (`tx_base + payload_len * byte_cost`),
4. resolve lane adapter from registry,
5. execute lane with journaled state,
6. on success => commit journal + success receipt,
7. on error => revert journal + failure receipt.

## 5) State journal model

`StateJournal` records ordered operations as:

- `Put { key, value }`
- `Delete { key }`

Writes are buffered until execution finishes. No partial application is visible in host state prior to commit.

## 6) Commit / revert model

- `commit`: apply journal operations sequentially to host state.
- `revert`: drop journal operations (no host mutation).

This model ensures transaction atomicity and deterministic rollback behavior.

## 7) Gas / fuel model

The core uses deterministic integer metering:

- intrinsic: `tx_base`, `byte_cost * payload_len`,
- runtime costs: `state_write_cost`, `state_delete_cost`, `event_base`,
- exhaustion: hard stop with `KernelError::GasExhausted`.

All costs are explicit and bounded in `FuelSchedule` to keep accounting auditable.

## 8) Minimal implementation skeleton

The `kernel.rs` module includes:

- complete struct/trait skeleton for production hardening,
- deterministic error surface,
- lane dispatch registry,
- commit/revert journal behavior,
- unit tests that validate:
  - commit-on-success,
  - revert-on-failure,
  - deterministic out-of-fuel failure.

Compatibility lanes (EVM/WASM) should implement `LaneAdapter` and remain outside the sovereign core semantic surface.

## 9) VM-agnostic + adapter standard (new)

Kernel surface is kept VM-blind and only reasons on:

- canonical tx envelope,
- deterministic receipt,
- state transition commitment,
- deterministic failure,
- finality proof payload.

Execution extension is now modeled with a strict adapter contract (`ExecutionAdapter`):

- `validate()`
- `execute()`
- `query()`
- `export_receipt()`
- `export_state_commitment()`

This means EVM/WASM/Move support can be added as lane adapters without changing kernel semantics.

## 10) Canonical envelope + unified receipts (new)

Canonical envelope now carries a normalized shape for all external lane types:

- `sender`
- `nonce`
- `fee`
- `payload`
- `lane`
- `auth_proof`
- `intent_flags`

Unified receipt output remains lane-independent:

- success/fail
- gas used
- events
- output
- state diff hash
- receipt hash

## 11) Lane capability manifest + compatibility tiers (new)

Each adapter is expected to expose a capability manifest describing:

- account model
- state model
- gas model
- event model
- contract lifecycle
- cross-lane support
- determinism level
- compatibility tier

Compatibility tiers are explicit:

- Tier 1: native/full compatibility
- Tier 2: high compatibility
- Tier 3: adapter compatibility
- Tier 4: settlement-only compatibility

## 12) Cross-lane bus + deterministic sandbox policy (new)

Cross-lane message bus (`CrossLaneBus`) enforces:

- deterministic message keying,
- versioned messages,
- replay protection,
- receipt-linked routing keys.

Host determinism hardening is expressed with `DeterministicSandboxPolicy`:

- no wall clock dependence,
- no random host calls,
- bounded memory/gas/syscalls,
- deterministic IO-only policy.

## 13) Replay + conformance evidence hooks (new)

`CoreKernel::conformance_replay` checks two critical properties:

1. same input -> same receipt,
2. same receipt -> same state commitment.

This is a baseline for malformed-input corpus testing, cross-platform determinism checks, and cross-version replay evidence.
