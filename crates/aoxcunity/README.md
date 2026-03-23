# AOXCUNITY

**Documentation Version:** `aoxc.v.0.1.0-testnet.1`
**Cargo-Compatible Version:** `0.1.0-testnet.1`

## Executive Summary
AOXCUNITY contains the consensus kernel that evaluates validator voting behavior, block admissibility, quorum thresholds, finality, pruning, and replay-safe recovery.

## Architectural Overview
This component is part of the AOX Chain production roadmap and is documented as a reviewable subsystem rather than a placeholder package. The goal of this README is to give enough context that an engineer, auditor, or operator can understand why the crate exists, what code families it owns, and where the main security boundaries live.

## Main Code Areas
- `state.rs`: block and vote admission, quorum observation, and finalization logic.
- `kernel.rs`: event-driven transition execution, invariants, timeout handling, and pruning coordination.
- `safety.rs`, `rotation.rs`, `round.rs`, and `fork_choice.rs`: safety policy, validator set rotation, round monotonicity, and branch selection.
- `tests/hack_resilience.rs`, `tests/adversarial_consensus.rs`, and `tests/block_fuzz_latency.rs`: adversarial, deterministic fuzz-style, and consensus regression evidence.

## Security and Audit Focus
This is a high-stakes component. Reviewers must prioritize quorum accounting, stale-vote rejection, equivocation handling, finality monotonicity, and deterministic recovery semantics.

Reviewers should additionally confirm the following before promotion.
- Interfaces remain deterministic and version-aligned with the workspace baseline.
- Inputs are validated before affecting durable state or privileged behavior.
- Tests cover both expected behavior and hostile or malformed scenarios.
- Operational assumptions are mirrored in the corresponding `READ.md` and `VERSION.md` files.

## Integration Notes
This README is intentionally paired with a folder-specific `READ.md` and `VERSION.md`. The README explains the subsystem at a high level, the READ document explains the production audit expectations in more depth, and the VERSION document defines the mandatory release-discipline rules for future changes.

## Release Status
Current subsystem baseline: `aoxc.v.0.1.0-testnet.1` / `0.1.0-testnet.1`.
