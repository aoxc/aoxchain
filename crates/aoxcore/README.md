# AOXCore

## Purpose
AOXCore provides the canonical core data models and deterministic execution
foundations for the AOXC chain runtime.

This crate defines the low-level primitives that shape runtime behavior,
including block-domain types, transaction-domain types, receipts, state
structures, and associated deterministic hashing and transition helpers.

## Responsibility Boundary
This crate is responsible for:

- canonical block data structures
- canonical transaction models and hashing
- canonical receipt models and hashing
- canonical account and world-state structures
- deterministic state-transition foundations
- runtime-facing validation primitives that must remain stable and auditable

This crate is not responsible for:

- wallet UX concerns
- node orchestration
- RPC presentation concerns
- explorer-facing formatting concerns
- infrastructure deployment logic

## Design Priorities
AOXCore is developed under the following engineering priorities:

1. Determinism  
   All critical state and hash derivations must remain deterministic across
   environments and implementations.

2. Auditability  
   Core logic must remain explicit, minimal, and suitable for formal review
   and security assessment.

3. Compatibility Discipline  
   Any change affecting state layout, hashing, validation, or transition
   semantics must be treated as compatibility-sensitive.

4. Fail-Closed Semantics  
   Invalid inputs, malformed states, and unsafe transitions must be rejected
   explicitly rather than tolerated implicitly.

## Expected Change Review Standard
Any change within this crate should be evaluated for:

- state layout impact
- hash output stability impact
- serialization / deserialization impact
- transaction / receipt compatibility impact
- runtime invariant impact
- backward compatibility implications
- testing coverage sufficiency

## Testing Expectations
Changes in this crate should normally be accompanied by tests covering, where
relevant:

- happy-path behavior
- edge conditions
- failure semantics
- invariant preservation
- deterministic hashing stability
- compatibility-sensitive regressions

## Repository Context
This crate is part of the AOXC pre-release codebase and should be treated as
consensus-adjacent runtime infrastructure.
