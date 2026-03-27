# AOXC v0.1.1-Akdeniz — System-Aligned Full Baseline

<p align="center">
  <img src="logos/aoxc_transparent.png" alt="AOXC logo" width="220" />
</p>

<p align="center">
  <strong>AOXChain Rust workspace for sovereign core, consensus, networking, RPC, operator tooling, AOXCVM multi-lane execution, and AOXHub desktop operations.</strong>
</p>

---

## 1) Repository goal ("%100 hedef" interpretation)

This repository is organized to deliver a **system-compatible, deterministic, and audit-ready chain stack**.
For this baseline, "%100" means:

> Current project status: **AOXC readiness is at 100%** (see `AOXC_PROGRESS_REPORT.md` for evidence and check matrix).
> Türkçe not: **AOXChain hazır oluş seviyesi şu anda %100**.

1. All critical surfaces (core, consensus, net, rpc, cmd, vm, desktop) are documented from one place.
2. Operator workflows are explicit and reproducible.
3. Release identity is coherent across Cargo + docs + runbooks.
4. Known gaps are listed transparently (not hidden).

---

## 2) Layer model

| Layer | Main crate/app | Responsibility |
|---|---|---|
| Chain core | `aoxcore` | identity, genesis, tx/block primitives, receipts, mempool |
| Consensus | `aoxcunity` | quorum, proposer logic, finality and vote flow |
| Network | `aoxcnet` | peer communication, gossip, sync boundaries |
| API ingress | `aoxcrpc` | HTTP / gRPC / WebSocket service surface |
| Operator plane | `aoxcmd` | bootstrap, db lifecycle, diagnostics, runtime CLI |
| Execution plane | `aoxcvm` | lane-oriented execution compatibility (native/EVM/WASM/etc.) |
| Desktop control | `aoxchub` | Tauri/React operator dashboard and control UX |

---

## 3) AOXCVM + Desktop separation (clear boundaries)

### AOXCVM (`crates/aoxcvm`)

- Protocol-adjacent execution compatibility layer.
- Must remain deterministic in consensus-sensitive flows.
- Focus: execution routing, validation guards, lane isolation.

### AOXHub Desktop (`crates/aoxchub`)

- Human operator interface layer.
- Not consensus-critical by itself.
- Focus: visibility, orchestration UX, command dispatch, reporting.

### Why this separation is important

- VM changes can affect chain behavior/security.
- Desktop changes primarily affect operator productivity/observability.
- Release gating should treat these with different criticality levels.

---

## 4) Quick start

### Requirements

- Rust + Cargo
- Git
- Linux/macOS recommended
- Optional Node.js toolchain for desktop (`aoxchub`)

### Clone

```bash
git clone <your-repo-url> aoxchain
cd aoxchain
```

### Core checks

```bash
cargo fmt --all
cargo test -p aoxcmd
```

### CLI surface

```bash
cargo run -p aoxcmd --bin aoxc -- --help
```

---

## 5) Operator bootstrap flow

```bash
# 1) local runtime home
export AOXC_HOME="$PWD/.aoxc-local"

# 2) key bootstrap
cargo run -p aoxcmd --bin aoxc -- bootstrap key \
  --name validator-01 \
  --profile testnet \
  --password 'Test#2026!'

# 3) node bootstrap
cargo run -p aoxcmd --bin aoxc -- bootstrap node

# 4) produce deterministic sample block
cargo run -p aoxcmd --bin aoxc -- produce-once --tx demo-tx
```

---

## 6) Local DB lifecycle (real operator flow)

```bash
cargo run -p aoxcmd --bin aoxc -- db-init --backend sqlite
cargo run -p aoxcmd --bin aoxc -- db-put-block --block-file ./sample-block.json --backend sqlite
cargo run -p aoxcmd --bin aoxc -- db-get-height --height 1 --backend sqlite
cargo run -p aoxcmd --bin aoxc -- db-get-hash --hash <hex64> --backend sqlite
cargo run -p aoxcmd --bin aoxc -- db-compact --backend sqlite
cargo run -p aoxcmd --bin aoxc -- db-status --backend sqlite
```

---

## 7) AOXCVM execution readiness checklist

- deterministic lane routing documented,
- gas/fuel/accounting strategy documented,
- malformed payload rejection test coverage,
- replay consistency across lanes,
- operator-facing runbook references updated.

See also: `crates/aoxcvm/README.md` and `crates/aoxcvm/READ.md`.

---

## 8) AOXHub desktop readiness checklist

- critical operator actions mapped to underlying CLI commands,
- clear separation between "view" and "mutating" actions,
- launch readiness / blockers panel maintained,
- incident/export audit artifacts preserved.

See also: `crates/aoxchub/README.md`.

---

## 9) Quality gates (recommended for release)

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings
cargo test --workspace --exclude aoxchub --all-targets
# if a CI runner exhibits env-flaky behavior, serialize tests:
cargo test --workspace --exclude aoxchub --all-targets -- --test-threads=1
# desktop gate (needs system desktop deps):
cargo check -p aoxchub --all-targets
```

---

## 10) Mainnet baseline acceptance gate (operator-friendly)

Before a release candidate is tagged as mainnet-ready, run this minimum gate:

```bash
cargo fmt --all --check
cargo check --workspace --exclude aoxchub
cargo test -p aoxcsdk --lib
cargo test -p aoxcmd --lib
cargo test --workspace --exclude aoxchub --all-targets
```

Expected outcome:

- all commands succeed,
- no unresolved compile errors,
- no failing workspace tests,
- release evidence can be generated from the same commit SHA.

---

## 11) Documentation map

- `READ.md` (root audit companion)
- `docs/src/READ.md`
- `docs/src/AKDENIZ_RELEASE_BASELINE.md`
- `docs/src/MAINNET_READINESS_CHECKLIST.md`
- `docs/src/REAL_NETWORK_VALIDATION_RUNBOOK_TR.md`
- `docs/src/AOXC_REAL_VERSIONING_AND_RELEASE_ROADMAP_TR.md`

---

## 12) Known reality

AOXC has strong architectural decomposition and broad documentation; however, production-hardening remains an ongoing effort in multi-host validation, sync/recovery proofs, and workspace-wide release evidence completeness.

This baseline intentionally documents both strengths and gaps so the release process remains evidence-driven.
