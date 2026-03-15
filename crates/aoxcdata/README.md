# aoxcdata

## Purpose

`aoxcdata` provides the AOXChain hybrid storage layer:

- **Block Body / immutable payloads** via IPFS/IPLD-compatible content addressing semantics.
- **State/Index metadata** via local query-friendly DB backends (**SQLite** or **Redb**).

## Data Policy

| Data Type | Storage Layer |
|---|---|
| Block body bytes | IPFS-style content-addressed blob store |
| Height/cid/hash index | SQLite or Redb |

## Main Types

- `BlockEnvelope`
- `BlockMeta`
- `HybridDataStore`
- `IndexBackend::{Sqlite, Redb}`

## Local Development

```bash
cargo check -p aoxcdata
cargo test -p aoxcdata
```

## Integration

Use from `aoxcmd` via:

```bash
cargo run -p aoxcmd -- storage-smoke --index sqlite
cargo run -p aoxcmd -- storage-smoke --index redb
```
Data/log storage helper crate for local persistence and runtime data hooks.

## Production Intent

This crate is part of the AOXChain relay-oriented mainnet roadmap. Its interfaces are expected to evolve toward:

- deterministic behavior in consensus-critical paths,
- explicit and typed error surfaces,
- testable integration boundaries with other workspace crates,
- audit-friendly documentation and change control.

## Local Development

From repository root:

```bash
cargo check -p aoxcdata
```

## Integration Notes

- Keep API changes synchronized with dependent crates in the same pull request.
- For consensus/network/identity touching changes, include tests or deterministic command paths.
- Avoid introducing implicit defaults in critical runtime logic; prefer explicit parameters.
