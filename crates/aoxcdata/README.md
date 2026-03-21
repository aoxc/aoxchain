# aoxcdata

## Purpose

`aoxcdata` is the production-oriented AOXChain data layer. It intentionally avoids
traditional database engines and instead implements a deterministic, auditable,
content-addressed storage architecture composed of:

- atomic filesystem-backed block blob storage,
- append-only metadata journal indexing with snapshot compaction,
- bounded and explicit key-value persistence primitives,
- deterministic Merkle state tree generation and proof verification.

## Design Principles

- No silent data loss paths
- Explicit integrity validation
- Atomic write discipline
- Crash-safe persistence patterns
- Deterministic hashing
- Strict error propagation
- Pluggable storage boundaries

## Current Storage Architecture

- `FsCasBlobStore`: content-addressed block persistence
- `FileMetaIndexStore`: append-only block metadata index with replay
- `FileKvDb`: filesystem KV surface with atomic per-key writes
- `StateTree`: deterministic ordered Merkle tree and proof engine

## Local Validation

```bash
cargo check -p aoxcdata
cargo test -p aoxcdata -- --nocapture
cargo clippy -p aoxcdata --all-targets --all-features -- -D warnings
```
