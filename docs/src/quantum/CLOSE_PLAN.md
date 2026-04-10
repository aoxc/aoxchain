# Quantum Near-Closure Plan (Core)

## Objective

Move AOXChain from policy-declared quantum readiness to operationally enforceable near-closure under deterministic and fail-closed controls.

This is a near-closure objective, not an absolute-security claim.

## Entry Conditions

The near-closure process starts only when all conditions hold:

1. kernel profile authority is consensus-visible;
2. cryptographic profile identifiers are versioned and validated;
3. baseline testnet gates are green for the active release line.

## Acceptance Rule

A release candidate may claim quantum-full near-closure only if:

- all closure tracks in `CLOSE_TRACKS.md` pass,
- all evidence requirements in `EVIDENCE_PACKAGE.md` are complete,
- cutover and rollback procedures in `CUTOVER_RUNBOOK.md` are rehearsed,
- and all artifacts are linked to the candidate commit.

## Non-Goals

This plan does not assert:

- unconditional cryptographic permanence,
- legal/regulatory guarantees,
- immunity to future cryptanalytic advances.
