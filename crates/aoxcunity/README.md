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

That document proposes **AOXC Covenant Consensus (ACC)** built around:

- deterministic kernel transitions,
- legitimacy certificates,
- continuity/timeout certificates,
- finality via a composite **Covenant Seal** rather than a single generic QC.

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
