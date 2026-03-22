# aoxcunity

## Purpose

`aoxcunity` is responsible for the **consensus engine** domain within the AOXChain workspace.

## Code Scope

- `quorum.rs`
- `vote.rs`
- `fork_choice.rs`
- `rotation.rs`
- `proposer.rs`
- `seal.rs`
- `state.rs`
- `constitutional.rs`
- `kernel.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcunity && cargo test -p aoxcunity
```


## AOXC-native Consensus Direction

For an AOXC-specific consensus design that goes beyond standard BFT clones, see
[`AOXC_COVENANT_CONSENSUS.md`](./AOXC_COVENANT_CONSENSUS.md).

For the current implementation gap analysis and the deterministic-engine roadmap, see
[`../../docs/AOXCUNITY_ENGINE_ROADMAP_TR.md`](../../docs/AOXCUNITY_ENGINE_ROADMAP_TR.md).

That document proposes **AOXC Covenant Consensus (ACC)** built around:

- deterministic kernel transitions,
- legitimacy certificates,
- continuity/timeout certificates,
- finality via a composite **Covenant Seal** rather than a single generic QC.

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)

## Current Status Clarification

`aoxcunity` already contains solid consensus primitives and finality-oriented building blocks,
but `kernel.rs` currently exposes the transition contract more clearly than a fully centralized
state-machine orchestrator. Until a single deterministic `ConsensusEvent -> TransitionResult`
engine owns round/lock/vote/fork-choice/finality evolution end to end, this crate should be
described as a **deterministic consensus scaffold** rather than a fully production-ready
consensus subsystem.
