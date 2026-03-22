# AOXC Covenant Consensus (ACC)

> Technical expansion: **AOXC Constitutional Consensus**.
> Narrative/branding name: **AOXC Covenant Consensus**.

## Goal

This document proposes an **AOXC-native consensus family** that is intentionally different from
classic Nakamoto longest-chain designs and from straightforward HotStuff/Tendermint clones.

The target is **not** to promise "99% bug-free" security. No honest protocol design can make that
claim. The target is instead:

- deterministic kernel behavior,
- explicit safety invariants,
- evidence-oriented fault handling,
- bounded-memory operation,
- constitutional legitimacy in addition to stake or validator weight,
- strong crash-recovery semantics,
- high-throughput optimistic path with slower but safer degraded modes.

## Core Idea

ACC separates consensus into **three coupled planes** instead of one monolithic vote loop.

1. **Execution Plane**
   - proposes canonical blocks,
   - carries deterministic AOXC execution payloads,
   - remains pure and replayable.

2. **Legitimacy Plane**
   - proves that the proposer and validator set are constitutionally valid for the current epoch,
   - binds governance/identity constraints to the block,
   - prevents "stake-only" authority from being the sole finality source.

3. **Continuity Plane**
   - tracks time, delay, recovery, and partition healing,
   - emits timeout and continuity certificates,
   - prevents liveness recovery from weakening safety rules.

A block is not merely "voted on". It must satisfy a **triple condition**:

- execution-valid,
- legitimacy-justified,
- continuity-safe.

This makes AOXC distinct from most existing chains, which typically merge all authority into a
single validator-weighted vote stream.

## Novel Primitive: Covenant Seal

The unique finality artifact in ACC is the **Covenant Seal**.

A Covenant Seal is only produced when all of the following exist for the same block:

- **Execution Quorum Certificate (EQC)**
  - weighted validator quorum over prepare/commit path.
- **Legitimacy Certificate (LC)**
  - proof that the active proposer and validator transition are constitutionally valid.
- **Continuity Certificate (CC)**
  - proof that the round was either on the optimistic fast path or safely recovered through timeout.

Finality is therefore:

`Finality = EQC + LC + CC -> CovenantSeal`

This is intentionally different from single-certificate BFT finality.

## Why This Is Different

ACC differs from mainstream designs in several ways:

- **not longest-chain PoW/PoS**: head selection is not derived from accumulated chain weight alone,
- **not plain HotStuff**: finality does not rely on a single chained QC abstraction,
- **not plain Tendermint**: legitimacy and continuity are first-class, protocol-visible objects,
- **not Avalanche-style repeated sampling**: safety depends on deterministic evidence artifacts,
- **not simple dual-quorum governance**: legitimacy is attached to proposer/epoch transition, not
  just off-chain governance metadata.

## Kernel Model

The kernel should stay pure and deterministic.

```rust
pub enum ConsensusEvent {
    ProposalReceived(ProposalEnvelope),
    PrepareVoteVerified(VerifiedVote),
    CommitVoteVerified(VerifiedVote),
    TimeoutVoteVerified(VerifiedTimeoutVote),
    LegitimacyProofObserved(LegitimacyProof),
    RecoveryTick(RecoveryTick),
    SnapshotRestored(KernelSnapshot),
}

pub struct TransitionResult {
    pub emitted_messages: Vec<KernelMessage>,
    pub finalized: Option<CovenantSeal>,
    pub evidence: Vec<ConsensusEvidence>,
    pub metrics: TransitionMetrics,
}

pub trait ConsensusKernel {
    fn apply(&mut self, event: ConsensusEvent) -> TransitionResult;
}
```

The important rule is that the kernel never directly performs network or storage I/O.

## Protocol Flow

### Phase 0 — Epoch Legitimacy

Before a proposer is accepted for round `r`, the proposal must carry a legitimacy basis:

- active epoch id,
- validator-set digest,
- transition certificate from previous epoch,
- proposer entitlement proof,
- constitutional policy hash.

This yields a **Legitimacy Certificate (LC)** candidate.

### Phase 1 — Deterministic Proposal

The proposer submits a canonical block with:

- parent block hash,
- justified parent seal,
- legitimacy basis,
- execution root,
- evidence root,
- timeout root,
- deterministic ordering metadata.

### Phase 2 — Prepare

Validators issue `Prepare` only if all safety predicates hold:

- parent is known,
- legitimacy basis is valid for the active epoch,
- proposal does not violate local lock,
- block extends the current safe branch,
- proposer entitlement is valid,
- round is inside the accepted continuity window.

### Phase 3 — Commit

Validators issue `Commit` only if:

- prepare quorum is justified,
- the proposal does not violate monotonic lock advancement,
- no stronger conflicting justification exists,
- continuity state permits commit progression.

The result is **EQC**.

### Phase 4 — Continuity

If the optimistic path fails, validators emit timeout votes.

Timeout votes are aggregated into **CC** and allow recovery without clearing safety locks.

### Phase 5 — Covenant Finality

A block finalizes only when:

- EQC is present,
- LC is present,
- CC or optimistic continuity proof is present,
- branch ancestry matches the latest finalized Covenant Seal.

Then the block receives a **Covenant Seal**.

## Safety Rules

### 1. Monotonic Lock Rule

A validator may only move its lock to a block justified by a strictly stronger safe justification.

### 2. Finalized Ancestry Rule

No event may revive a branch that is not descendant from the latest finalized Covenant Seal.

### 3. Legitimacy Non-Skipping Rule

A proposer cannot skip epoch legitimacy transition requirements even when it has enough stake.

### 4. Timeout Non-Reset Rule

Timeout recovery may advance liveness but cannot erase safety locks.

### 5. Evidence Persistence Rule

Equivocation and invalid-transition evidence must survive crash recovery and replay.

## Data Structures

### Proposal Envelope

```rust
pub struct ProposalEnvelope {
    pub block_hash: [u8; 32],
    pub parent_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub execution_root: [u8; 32],
    pub legitimacy_root: [u8; 32],
    pub evidence_root: [u8; 32],
    pub timeout_root: [u8; 32],
    pub proposer: [u8; 32],
    pub parent_covenant: Option<[u8; 32]>,
}
```

### Legitimacy Proof

```rust
pub struct LegitimacyProof {
    pub epoch: u64,
    pub validator_set_hash: [u8; 32],
    pub transition_certificate_hash: [u8; 32],
    pub proposer_right_hash: [u8; 32],
    pub constitutional_policy_hash: [u8; 32],
}
```

### Continuity Certificate

```rust
pub struct ContinuityCertificate {
    pub height: u64,
    pub round: u64,
    pub timeout_round: u64,
    pub signer_root: [u8; 32],
    pub observed_power: u64,
}
```

### Covenant Seal

```rust
pub struct CovenantSeal {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub eqc_hash: [u8; 32],
    pub legitimacy_hash: [u8; 32],
    pub continuity_hash: [u8; 32],
    pub seal_hash: [u8; 32],
}
```

## Performance Strategy

ACC should still be fast in the optimistic path.

### Fast Path

- indexed block store (`HashMap<BlockHash, BlockRecord>`),
- indexed vote buckets by `(height, round, kind, block_hash)`,
- incremental power accounting,
- cached active-validator views,
- branch-local pruning after finalization,
- append-only WAL plus periodic snapshots.

### Slow Safe Path

When partitions or legitimacy transitions occur:

- enable continuity-certificate collection,
- narrow acceptable proposer set,
- require explicit transition justification,
- increase evidence retention window,
- keep deterministic replay of all recovery events.

## Storage Model

ACC should not treat in-memory state as authority.

Required storage surfaces:

- `consensus_journal.log` for append-only events,
- `kernel_snapshot.bin` for checkpoint recovery,
- `evidence_store.bin` for equivocation and invalid-legitimacy proofs,
- `epoch_store.bin` for validator transition certificates,
- `finality_store.bin` for latest Covenant Seals.

Recovery rule:

1. load snapshot,
2. replay journal deterministically,
3. rehydrate evidence and latest Covenant Seal,
4. recompute caches only from persisted state.

## Suggested AOXC-Specific Differentiators

If AOXC truly wants to be different from every existing chain, the most defensible differentiators are:

### A. Constitutional Quorum

A block may require both:

- weighted validator quorum, and
- legitimacy quorum over constitutionally required validator classes or zones.

This is stronger than ordinary stake-only finality.

### B. Role-Segmented Validator Authority

Validators can have distinct operational authority lanes:

- execution validators,
- continuity validators,
- constitutional validators,
- observer-auditors.

A seal can require a subset of these lanes depending on epoch mode.

### C. Emergency Safety Mode

Under partition or detected governance anomaly, AOXC can move from:

- `OptimisticMode` -> `RecoveryMode` -> `ConstitutionalSafeMode`

without changing deterministic kernel semantics.

### D. Evidence-First Punishment

Instead of vague slash hooks, all faults become canonical evidence objects first:

- proposal equivocation,
- vote equivocation,
- invalid legitimacy claim,
- invalid timeout certificate,
- illegal epoch transition.

## What To Build First

### P0

- `VerifiedVote` / `VerifiedTimeoutVote` pipeline,
- persistent consensus journal,
- kernel snapshot and replay,
- bounded vote/evidence retention,
- explicit lock and justify state.

### P1

- legitimacy proof objects,
- continuity certificate objects,
- epoch transition certificates,
- model-based tests for safety invariants,
- partition/heal simulations.

### P2

- multi-lane constitutional quorum,
- emergency safety mode,
- aggregated signatures,
- proposer reputation scoring that does not affect determinism,
- benchmark suite for optimistic vs degraded mode.

## Minimum Invariants To Test

- no two conflicting Covenant Seals can exist at the same finalized height,
- finalized head is monotonic,
- timeout recovery never breaks the lock rule,
- legitimacy transition cannot be skipped,
- stale votes cannot revive non-finalized branches,
- replayed journal reaches the same final state as live execution,
- signer order does not affect seal hash,
- pruning never removes the active finalized branch.

## Honest Recommendation

If you want something **really different** yet still defensible:

- keep the kernel deterministic and small,
- make legitimacy explicit instead of implicit,
- treat continuity/timeouts as first-class certificates,
- finalize with a Covenant Seal instead of one generic quorum proof,
- never claim absolute security percentages.

That combination can make AOXC recognizably its own system without becoming un-auditable.


## Integration Contract

- `ConsensusState` owns execution-plane block/vote admission and execution-finality quorum evaluation.
- A verified-admission layer is expected to produce `VerifiedVote`, `VerifiedTimeoutVote`, legitimacy artifacts, and continuity artifacts before kernel consumption.
- `ExecutionCertificate` is built from execution quorum evidence.
- `ConstitutionalSeal` is composed only after execution, legitimacy, and continuity artifacts independently validate and bind to the same block/epoch boundary.
- This phase intentionally does not implement full persistence, pacemaker, or validator-transition engines.


## Near-Mainnet Kernel Contracts

- `LockState` and `JustificationRef` define conservative lock advancement boundaries.
- `ConsensusJournal`, `SnapshotStore`, `EvidenceStore`, and `FinalityStore` define replay-safe persistence contracts without forcing a storage backend.
- Recovery remains typed and deterministic: snapshot load -> journal replay -> evidence restore -> finalized seal restore.
