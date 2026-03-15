<div align="center">

# 🔷 AOXChain

**Interoperability-first relay chain architecture for deterministic cross-chain coordination.**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-000000?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Workspace](https://img.shields.io/badge/Workspace-Multi%20Crate-6f42c1)](Cargo.toml)

</div>

---

## What AOXChain is (and is not)

AOXChain is being designed as a **relay-oriented coordination chain**.

It is intended to prioritize:

- **deterministic coordination** across heterogeneous ecosystems,
- **cross-chain compatibility** over short-term TPS maximization,
- **auditability and explicit control flow** in identity/consensus/runtime paths.

It is **not** positioned as a pure monolithic throughput chain.

## Production-Level Vision

AOXChain mainnet target posture:

1. Deterministic settlement and verifiable consensus transitions.
2. Strong identity model (actor IDs, certificates, passports, PQ-ready primitives).
3. Multi-lane compatibility (EVM/WASM/Sui/Cardano-oriented adapters).
4. Hardened operations (CLI runbooks, reproducible builds, threat-model-backed releases).

## Repository Structure

| Path | Responsibility |
|---|---|
| `crates/aoxcore` | Core protocol primitives: identity, genesis, tx model, mempool, base state |
| `crates/aoxcunity` | Consensus kernel: rotation, quorum, votes, fork-choice, finalization |
| `crates/aoxcvm` | Multi-lane execution compatibility layer |
| `crates/aoxcnet` | Networking, gossip, discovery, sync surfaces |
| `crates/aoxcrpc` | RPC ingress surfaces (HTTP / gRPC / WebSocket) |
| `crates/aoxcmd` | Operational node bootstrap and deterministic smoke commands |
| `crates/aoxckit` | Keyforge and operator tooling |
| `docs/` | Architecture, audit readiness, and mainnet blueprint docs |

Each crate now includes a local `README.md` with purpose and integration guidance.

## Quickstart (Deterministic Operator Path)

From repository root:

```bash
cargo check --workspace
cargo test -p aoxcmd
cargo test -p aoxcunity
```

### CLI flow (`aoxcmd`)

```bash
# 1) Inspect strategic chain posture
cargo run -p aoxcmd -- vision

# 2) Materialize genesis
cargo run -p aoxcmd -- genesis-init \
  --path AOXC_DATA/identity/genesis.json \
  --chain-num 1 \
  --block-time 6 \
  --treasury 1000000000

# 3) Bootstrap key + identity material
cargo run -p aoxcmd -- key-bootstrap \
  --password "change-me" \
  --base-dir AOXC_DATA/keys \
  --name relay-1 \
  --chain AOXC-MAIN \
  --role relay \
  --zone global \
  --issuer AOXC-ROOT-CA \
  --validity-secs 31536000

# 4) Validate node bootstrap
cargo run -p aoxcmd -- node-bootstrap

# 5) Produce one deterministic block (single-node smoke)
cargo run -p aoxcmd -- produce-once --tx "relay-coordination-demo"

# 6) Validate gossip stub behavior
cargo run -p aoxcmd -- network-smoke

# 7) Validate hybrid data layer (IPFS + SQLite/Redb)
cargo run -p aoxcmd -- storage-smoke --index sqlite
cargo run -p aoxcmd -- storage-smoke --index redb
```


## Storage Architecture (Mainnet Direction)

| Data Type | Storage Layer | Rationale |
|---|---|---|
| Block Body (immutable data) | IPFS/IPLD-compatible content addressing | Global accessibility, integrity, immutable content IDs |
| State & Index metadata | SQLite or Redb | Fast local query/index lookups for balances, block-height references, and runtime checks |

Current implementation provides a deterministic hybrid storage abstraction via `aoxcdata` and `aoxcmd storage-smoke`.

## Mainnet Hardening Backlog (Explicit)

- Transport-backed peer gossip and queueing in `aoxcnet`.
- Multi-node adversarial integration tests (`proposal -> vote -> finalize`).
- RPC-to-runtime persistent state integration.
- Threat model + fuzzing + external security audit closure.
- Reproducible release pipeline with signed artifacts and attestations.

## Engineering Standards

- Keep consensus/network/identity changes explicit and typed.
- Update dependent crates in the same PR when interfaces change.
- Prefer deterministic command/test paths over ad-hoc manual verification.
- Keep production claims tied to reproducible test evidence.

## Additional Documentation

- `docs/AUDIT_READINESS_AND_OPERATIONS.md`
- `docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`
- `docs/TEKNIK_DERIN_ANALIZ_TR.md`

## License

MIT (`LICENSE`).
