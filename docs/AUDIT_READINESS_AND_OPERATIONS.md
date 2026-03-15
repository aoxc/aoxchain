# AOXChain Audit Readiness and Operations Guide

## 1. Purpose

This document defines the current engineering posture for auditability, operational determinism, and staged mainnet hardening.

It is intentionally technical and implementation-oriented, intended for:
- protocol engineers,
- runtime maintainers,
- security reviewers,
- and infrastructure operators preparing deterministic testnets.

## 2. Deterministic Execution Envelope

The current deterministic smoke path is centered on `aoxcmd`:

1. Strategy introspection (`vision`)
2. Genesis materialization (`genesis-init`)
3. Key lifecycle bootstrap (`key-bootstrap`)
4. Node bootstrap (`node-bootstrap`)
5. Single-block production and local finalization attempt (`produce-once`)
6. Network compatibility check against gossip stub (`network-smoke`)

### Canonical commands

```bash
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- genesis-init --path AOXC_DATA/identity/genesis.json
cargo run -p aoxcmd -- key-bootstrap --password "change-me"
cargo run -p aoxcmd -- node-bootstrap
cargo run -p aoxcmd -- produce-once --tx "audit-path"
cargo run -p aoxcmd -- network-smoke
```

## 3. Security-Oriented Design Notes

### 3.1 Key lifecycle and trust artifacts

- Key material is managed through `KeyManager` and `KeyLoader` abstractions.
- Certificates are issued via post-quantum CA primitives (Dilithium-based implementation in core identity module).
- Runtime output exposes only summary material by default in CLI responses.

### 3.2 Consensus bootstrap discipline

- Validator rotation is constructed via explicit `ValidatorRotation::new(...)` result handling.
- Quorum threshold is instantiated explicitly (`2/3`) rather than implied defaults.
- Consensus state initialization requires explicit rotation and quorum parameters.

### 3.3 Mempool and block production boundaries

- Mempool enforces tx-count, byte-size, and TTL limits.
- Single-block production path inserts explicit transaction payloads, derives deterministic tx IDs, and attempts local commit finalization.
- Block archival uses atomic-style persistence flow (`tmp` + `sync_all` + `rename`).

## 4. Known Non-Mainnet Surfaces (Explicit)

1. `aoxcnet` gossip remains transport-stubbed; inbound messages are intentionally `None` until p2p binding is finalized.
2. `produce-once` is a deterministic smoke path, not a full multi-node consensus driver.
3. Runtime service orchestration modules still require deeper integration with persistent state backends and RPC layers.

## 5. Audit Checklist (Engineering)

- [x] Build reproducibility at workspace level (`cargo check --workspace`)
- [x] Consensus unit test execution in CI-like path
- [x] Explicit error propagation for bootstrap-critical phases
- [x] Deterministic one-block execution flow available from CLI
- [ ] Multi-node adversarial consensus simulation suite
- [ ] Cryptographic key custody hardening (HSM/external signer design)
- [ ] Structured observability baseline (metrics, trace correlation IDs, SLO gates)
- [ ] Third-party external security assessment report

## 6. Recommended Next Implementation Steps

1. Introduce transport-backed gossip and peer routing in `aoxcnet`.
2. Add integration tests spanning `aoxcmd + aoxcnet + aoxcunity` for proposal/vote/finalize lifecycle.
3. Add deterministic fixture-driven state transition tests for replay safety.
4. Introduce signed release manifests and reproducible build attestations.

## 7. Documentation Governance

- Operational command examples in this file must remain executable against the current codebase.
- Any CLI contract change must update this document in the same pull request.
- Mainnet-readiness claims must be backed by reproducible test artifacts.
