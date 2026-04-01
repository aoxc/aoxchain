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
