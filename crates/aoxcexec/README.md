# aoxcexec

## Purpose

`aoxcexec` is the **Execution Orchestrator** domain within the AOXChain workspace. It serves as the definitive, deterministic bridge between the sovereign consensus layer (`aoxcunity`) and the multi-lane virtual machine environment (`aoxcvm`).

Instead of executing smart contracts directly, this crate is responsible for receiving block contexts, validating execution payloads, safely routing transactions to their respective execution lanes (e.g., EVM, WASM, SUI Move), enforcing strict block-level gas limits, and generating deterministic execution receipts.

## Core Components

- **`ExecutionOrchestrator`**: The primary trait defining the batch execution interface for the network.
- **`ExecutionContext`**: Provides deterministic block-level context (height, timestamp, max gas) to the execution engine.
- **`ExecutionPayload`**: The standardized transaction wrapper containing lane routing information and gas limits.
- **`ExecutionReceipt`**: The outcome of an execution attempt, recording gas usage, state transitions, and success/failure status.
- **`ExecutionError`**: Strict error handling for invalid payloads, gas depletion, arithmetic overflows, and unavailable lanes.

## Code Scope

- `src/lib.rs` - Core orchestration logic, zero-risk arithmetic models, traits, and payload types.

## Security & Operational Notes

- **Zero-Risk Arithmetic**: All gas calculations, limit tracking, and state transitions **must** use `checked_*` operations to prevent overflows and consensus splits.
- **Graceful Rejection**: The orchestrator must never panic on invalid user input or gas depletion; it must gracefully return an appropriate `ExecutionError` or a failed `ExecutionReceipt`.
- **API Stability**: Changes to the orchestrator interface directly impact the consensus-to-execution pipeline and require rigorous cross-crate validation.
- **Explicit Definitions**: Prefer explicit parameters over implicit defaults in critical execution paths to maintain auditability.

## Local Validation

Before submitting changes to this crate, ensure it passes all static analysis and deterministic tests:

```bash
cargo check -p aoxcexec
cargo clippy -p aoxcexec --all-targets --all-features -- -D warnings
cargo test -p aoxcexec
Related Components
Top-level architecture: ../../README.md

Consensus Layer: ../aoxcunity/README.md

Execution Lanes (VMs): ../aoxcvm/README.md
