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
