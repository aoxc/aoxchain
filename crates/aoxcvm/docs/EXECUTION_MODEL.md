# EXECUTION MODEL

AOXCVM yürütmesi üç ilke ile tanımlanır:

- Determinism-first
- Policy-before-mutation
- Overlay-before-commit

## Phases

1. Validate transaction envelope.
2. Verify authorization envelope.
3. Resolve package and protocol compatibility.
4. Execute instruction stream on bounded machine state.
5. Accumulate writes/deletes in state overlay.
6. Enforce capability and syscall policy again on produced diff.
7. Commit only if all checks pass.

## Hard rule

Unverified bytecode or unauthorized mutation can never enter canonical state.
