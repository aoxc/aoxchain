# aoxclibs

## Purpose

`aoxclibs` provides the **deterministic utilities and common shared primitives** for the AOXChain workspace. 

To prevent code duplication and enforce network-wide consistency, this crate houses zero-risk helper functions. It contains strictly **no business or domain logic**. Every function provided here is designed to be panic-free, fully deterministic, and safe for use across consensus, execution, and data layers.

## Core Components

- **`time.rs`**: Safe, drift-resistant time utilities enforcing UNIX timestamp standards (UTC) without implicit timezone mutations. Crucial for consensus-critical path synchronization.
- **`encoding.rs`**: Zero-panic payload formatters. Enforces **uppercase hexadecimal standards** for cryptographic keys and hashes to ensure cross-node state root determinism.
- **`types.rs`**: Common network-wide error types (`LibError`) and shared aliases used across the sovereign core.

## Code Scope

- `src/lib.rs` - Main entry point and module exports.
- `src/time.rs` - System time and UNIX epoch abstractions.
- `src/encoding.rs` - Hexadecimal and byte-array transformation logic.
- `src/types.rs` - Shared library-level error definitions.

## Security & Operational Notes

- **Zero Panic Policy**: Functions in this crate must never `unwrap()`, `expect()`, or `panic!()`. All failures (e.g., malformed hex strings, system clock errors) must be returned gracefully via the `Result` type.
- **No Domain Logic**: This crate is strictly a utility layer. It must not import `aoxcore` or any higher-level domain module. It remains at the absolute bottom of the dependency tree.
- **Deterministic Formatting**: All encoding operations must yield a single, stable representation. Ambiguous or multi-representation encodings are forbidden to prevent consensus drifts.

## Local Validation

Before submitting changes to the shared library, ensure that all deterministic utility tests pass:

```bash
cargo fmt --all -- --check
cargo check -p aoxclibs
cargo clippy -p aoxclibs --all-targets --all-features -- -D warnings
cargo test -p aoxclibs -- --nocapture
Related Components
Top-level architecture: ../../README.md

Sovereign Consensus: ../aoxcunity/README.md

Data Persistence: ../aoxcdata/README.md
