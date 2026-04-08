# Scope: `crates/kernel/aoxcore`

## Mission
Define the canonical runtime data models and deterministic transition
foundations used by AOXC core execution flows.

## In Scope
The following concerns belong to this crate:

- block-domain core structures
- transaction-domain core structures
- receipt-domain core structures
- account and world-state models
- deterministic hashing and commitment helpers
- validation primitives directly tied to runtime correctness
- low-level transition helpers that influence chain behavior

## Out of Scope
The following concerns should generally remain outside this crate unless there
is a strong architectural reason to include them:

- user interface logic
- wallet application behavior
- RPC response formatting
- indexer-specific presentation layers
- infrastructure automation
- deployment orchestration
- operational scripting unrelated to runtime semantics

## Engineering Rule
Any change inside this crate must be reviewed as potentially runtime-sensitive.

Particular caution is required for changes affecting:

- state structure shape
- serialized layout
- hash derivation
- deterministic ordering
- validation rules
- transition invariants
- error semantics relied upon by upstream components

## Compatibility Rule
Changes that alter any canonical output or persisted structure must be treated
as compatibility-relevant and evaluated accordingly.

Examples include:

- account field changes
- transaction signing payload changes
- receipt hashing changes
- state-root changes
- transaction pool ordering changes
- new validation behavior that changes accepted / rejected inputs

## Testing Rule
A modification in this crate is not complete unless its correctness and
compatibility impact are reflected in tests where applicable.

At minimum, consider:

- deterministic behavior
- invariant preservation
- malformed input rejection
- overflow / underflow handling
- regression coverage for changed semantics

## Review Standard
When in doubt, prefer:

- explicitness over convenience
- deterministic behavior over implicit behavior
- smaller trusted surfaces over broader abstractions
- fail-closed handling over permissive recovery
