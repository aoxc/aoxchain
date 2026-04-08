# Contributing to AOXCore

## General Standard
AOXCore should be treated as runtime-critical and compatibility-sensitive code.

Contributions should favor:

- deterministic behavior
- explicit validation
- minimal trusted surfaces
- fail-closed semantics
- audit-friendly implementation style
- clear tests for all compatibility-sensitive changes

## Required Mindset
Before changing code in this crate, evaluate whether the change may affect:

- canonical hashing
- state layout
- transaction semantics
- receipt semantics
- world-state commitments
- block execution assumptions
- upstream integration expectations

If the answer may be yes, the change should be treated as compatibility-sensitive.

## Code Style Expectations
Contributions should avoid:

- dead code
- placeholder branches
- ambiguous validation logic
- silent fallbacks for invariant-sensitive paths
- unnecessary abstraction that hides runtime behavior
- unbounded input handling where bounded handling is appropriate

Comments and documentation should be written in clear, professional English,
especially around security, validation, state, and hashing logic.

## Testing Expectations
Contributors should add or update tests when changing:

- validation rules
- hashing behavior
- canonical serialization assumptions
- deterministic ordering
- state transitions
- error surfaces
- bounds checks

Useful coverage commonly includes:

- successful flows
- edge cases
- malformed input rejection
- overflow / underflow protection
- regression tests for compatibility-sensitive behavior

## Review Expectations
A change is easier to review when it includes:

- a narrow and well-defined purpose
- an explanation of compatibility implications
- tests aligned to the change
- no unrelated cleanup mixed into sensitive logic
- consistent naming and documentation

## Preferred Change Shape
Prefer small, reviewable, deterministic changes over large mixed refactors.

Where possible:

1. isolate the behavioral change
2. update tests
3. document compatibility implications
4. keep runtime-critical code paths explicit

## Final Note
AOXCore is not ordinary application code. It is foundational runtime code.
Changes should be made conservatively and reviewed with long-term maintenance,
security, and deterministic behavior in mind.
