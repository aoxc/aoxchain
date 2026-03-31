# AOXCHub

AOXCHub is a localhost-only operator control plane for AOXChain. It presents approved AOXC and Make workflows with immutable command previews, explicit confirmation, and transparent terminal streaming.

## Purpose
- Provide a professional local operator console for non-terminal operators.
- Preserve command truth by showing exact execution semantics before launch.
- Enforce environment-specific controls for MAINNET and TESTNET.

## Contents
- Embedded dark-theme UI served directly from the Rust binary.
- Immutable compile-time command catalog for AOXC and Make actions.
- Binary discovery with source trust metadata and policy enforcement.
- Local process runner with output limits and timeout controls.

## Usage
1. Build and run: `cargo run -p aoxchub` (or `make hub-mainnet` / `make hub-testnet`).
2. Open `http://127.0.0.1:7070`.
3. Select MAINNET or TESTNET from the visible selector.
4. Confirm Root Environment Binding values (profile, config path, AOXC home, Make scope).
5. Select an approved AOXC binary source.
6. Review preview text, confirm execution, and monitor terminal output.

## Product Blueprint
A full operator-surface specification is maintained in `OPERATOR_BLUEPRINT.md`.

## Notes
- The service binds to `127.0.0.1` by default and is not designed for remote exposure.
- AOXCHub is orchestration and observability only; AOXC remains the canonical operational binary.
- Repository policy and license obligations remain governed by the AOXChain MIT license context.

## Capacity Controls
AOXCHub runner behavior can be tuned without code changes through environment variables:

- `AOXCHUB_MAX_CONCURRENT_JOBS` (default: `8`): maximum concurrently executing jobs.
- `AOXCHUB_ACQUIRE_TIMEOUT_MS` (default: `250`): maximum queue wait time for an execution slot.
- `AOXCHUB_MAX_JOB_RECORDS` (default: `512`): in-memory job record retention ceiling.
- `AOXCHUB_COMMAND_TIMEOUT_SECS` (default: `300`): hard timeout for an individual command execution.
- `AOXCHUB_MAX_OUTPUT_BYTES` (default: `524288`): aggregate captured stdout/stderr bytes retained per job.

These controls are designed to enforce deterministic backpressure and bounded memory growth under elevated operator load.
