# aoxcexec

## Purpose

`aoxcexec` is the **Execution Orchestrator** domain within the AOXChain workspace. It serves as the deterministic bridge between the sovereign consensus layer (`aoxcunity`) and the multi-lane virtual machine environment (`aoxcvm`).

This crate now models a production-oriented execution pipeline that:

- validates authenticated `ExecutionPayload` envelopes with replay protection,
- resolves versioned lane policies via a governance-friendly `LaneRegistry`,
- executes deterministic lane transitions against a concrete state store,
- emits `ExecutionResult`, `WriteSet`, `StateDiff`, and `PostStateCommitment`,
- derives canonical batch roots (`state_root`, `receipt_root`, `transactions_root`, `execution_trace_root`, `block_execution_root`), and
- publishes operator-facing execution summaries with deterministic telemetry counters.

## Core Components

- **`ExecutionPayload`**: Versioned transaction envelope with `chain_id`, `sender`, `nonce`, `signature`, fees, replay domain, and access scope.
- **`LaneRegistry`**: Versioned lane policy registry with checksum verification and activation heights.
- **`ExecutionLane`**: Runtime isolation contract for lane validation, gas estimation, execution, result verification, and state commits.
- **`InMemoryStateStore`**: Deterministic authenticated state backend used for canonical roots and replay tests.
- **`BatchExecutionOutcome`**: Final batch artifact containing receipts, execution results, telemetry summary, and all batch-level commitments.

## Security & Operational Notes

- **Deterministic commitments**: All roots are domain-separated BLAKE3 commitments.
- **Replay protection**: Payloads are rejected on `chain_id`, `replay_domain`, nonce, or signature mismatch.
- **Canonical ordering**: Payloads are normalized by `(nonce, tx_hash)` before execution to guarantee deterministic replay.
- **Policy integrity**: Registry entries are checksum-verified before a lane policy becomes active.
- **State safety**: Invalid transactions never receive state mutation rights; only accepted payloads produce write-sets and state diffs.
- **Operator telemetry**: Batch summaries expose rejected payload counts, nonce violations, lane gas utilization, and active policy versions.

## Local Validation

```bash
cargo check -p aoxcexec
cargo clippy -p aoxcexec --all-targets --all-features -- -D warnings
cargo test -p aoxcexec
```
