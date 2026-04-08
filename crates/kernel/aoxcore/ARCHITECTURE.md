# AOXCore Architecture

`aoxcore` is the kernel-domain crate that defines canonical protocol types and deterministic boundaries used by upper layers.

## Architectural Role

AOXCore is not an execution-engine crate. Its primary role is to define kernel truth:

- canonical runtime domain objects (block/transaction/receipt/state),
- canonical protocol envelopes and interoperability metadata,
- deterministic trust-boundary interfaces for proof, finality, and settlement classification.

Execution integration remains downstream of these definitions.

## Module Responsibility Map

- `asset`, `block`, `transaction`, `receipts`, `mempool`, `contract`, `genesis`, `native_token`:
  deterministic runtime primitives and canonical state-transition inputs.
- `identity`:
  deterministic identity, signer, and authority-adjacent primitives.
- `protocol`:
  canonical protocol envelope model and kernel interoperability boundary types.

## Interoperability-Native Kernel Surfaces

Within `protocol`, the kernel explicitly owns:

- chain profile identity and compatibility class declarations,
- canonical cross-chain message routing keys,
- proof-type classification and verifier dispatch boundary contracts,
- finality classification input vocabulary,
- policy-evaluation input and outcome types,
- authority-domain / universal identity mapping boundaries,
- replay-protection domain separation semantics.

These are typed in kernel space so higher layers consume a stable, audit-friendly protocol vocabulary.

## Boundary Rules

- Kernel types must remain deterministic and serialization-stable where marked canonical.
- Verification, finality, and settlement decisions must be represented as explicit typed outcomes.
- Service and execution layers may implement behavior behind traits, but must not redefine kernel semantics.
- External I/O, relayer behavior, and cryptographic implementation details remain outside AOXCore.

## Quantum-Native Evolution Contract

The `protocol::quantum::QuantumKernelProfile` surface is the kernel’s upgrade-safe
contract for post-quantum readiness:

- strict defaults are PQ-only and fail closed (`legacy_signature_support = false`),
- profile validation enforces canonical, deterministic policy constraints,
- profile upgrades are versioned and must preserve acceptance of the active default signature
  to avoid architecture rewrites across runtime components.
- profile admission now includes `admit_quantum_transaction`, binding profile policy
  to `transaction::quantum::QuantumTransaction` validation at kernel scope.

`transaction::quantum::QuantumTransaction` also exposes a canonical signing-message builder
for deterministic external signing flows, preserving message-shape stability across
components that generate signatures out of process.
It now also publishes deterministic `intent_id` and `tx_id` derivation so block assembly,
pooling, and telemetry can use a stable quantum transaction identity contract.
`transaction::pool::QuantumTransactionPool` consumes this contract and enforces
profile-bound admission (`QuantumKernelProfile`) plus sender/nonce lane protection
for PQ transactions at kernel scope.
The pool now also supports bounded `select_for_block`/`drain_for_block` assembly
and converts admitted PQ transactions into canonical block `Task` objects.
Selection order is deterministic (`tx_id`-ordered) to keep block proposal inputs
stable across nodes under identical pool state.

This model allows cryptographic agility over time while keeping kernel data-model
boundaries stable for downstream services.
