# aoxcdata

## Purpose

`aoxcdata` is the production-grade **Data and Persistence Layer** for the AOXChain workspace. 

To guarantee absolute consensus determinism and bypass the hidden non-determinism of traditional database engines, this crate implements a bespoke, auditable, and content-addressed storage architecture. It provides the sovereign core with crash-safe persistence, atomic writes, and cryptographic state verification.

## Core Components

The architecture relies on bounded, explicit persistence primitives rather than black-box databases:
- **`FsCasBlobStore`**: An atomic, filesystem-backed Content-Addressable Storage (CAS) for raw block data and large payloads.
- **`FileMetaIndexStore`**: An append-only metadata journal and indexing engine, featuring automatic snapshot compaction and crash-resilient replay mechanisms.
- **`FileKvDb`**: Bounded, explicit key-value persistence primitives enforcing atomic, per-key disk writes.
- **`StateTree`**: A deterministic, ordered Merkle tree engine responsible for generating consensus-critical state roots and verifying cryptographic inclusion proofs.

## Code Scope

- `src/lib.rs` - Main entry point and persistence traits.
- `src/store/` - Implementations of CAS, KV databases, and journal indexing.
- `src/tree/` - Merkle tree generation and deterministic hashing engines.

## Security & Operational Notes

- **Zero Silent Data Loss**: The architecture strictly enforces an atomic write discipline. Partial writes or corrupted flushes must be detected and fully recovered via the append-only journal.
- **Absolute Determinism**: All hashing and state tree generations must yield the exact same Merkle root across all validator nodes, regardless of the underlying OS or filesystem block size.
- **Strict Error Propagation**: Disk-level failures, I/O timeouts, or integrity validation faults must immediately and gracefully halt the node. The system must **never** serve corrupted or partial state to the consensus layer.
- **Crash-Safe Persistence**: The node must be able to recover flawlessly from an unexpected power loss or process kill event by replaying the `FileMetaIndexStore` over the `StateTree`.

## Local Validation

Before submitting changes to the data layer, ensure all deterministic tests and static analysis checks pass flawlessly. Due to the nature of filesystem operations, ensure tests clean up their mock directories.

```bash
cargo fmt --all -- --check
cargo check -p aoxcdata
cargo clippy -p aoxcdata --all-targets --all-features -- -D warnings
cargo test -p aoxcdata -- --nocapture
Related Components
Top-level architecture: ../../README.md

Core Primitives & Hashing: ../aoxcore/README.md

Sovereign Consensus: ../aoxcunity/README.md
