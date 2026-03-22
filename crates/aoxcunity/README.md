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
- `safety.rs`
- `store.rs`

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

## Integration Contract

- Verified artifacts are expected to be produced outside the kernel-facing state machine via real signature verification (`SignedVote -> VerifiedVote`, timeout equivalent).
- `ConsensusState` remains the execution-plane admission/finalization holder.
- Constitutional artifacts are composed on top of execution finality instead of being implicitly folded into `ConsensusState`.

## Near-mainnet hardening

- warning-free builds are required before merge,
- kernel-facing paths are expected to remain verified-input-only,
- persistence/recovery contracts should stabilize before full backend implementation.
- monotonic constitutional finality and replay determinism should be test-proven at the kernel layer.

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
