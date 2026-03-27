# AOXC v0.1.1-Akdeniz

<p align="center">
  <img src="logos/aoxc_transparent.png" alt="AOXC logo" width="220" />
</p>

<p align="center">
  <strong>AOXChain Rust workspace for sovereign core, consensus, networking, RPC, operator tooling, and production-oriented runbooks.</strong>
</p>

---

## 1. What this repository is

AOXC is a multi-crate Rust workspace that organizes the chain into clearly separated layers:

- **`aoxcore`**: identity, genesis, mempool, receipts, transactions, and canonical block primitives.
- **`aoxcunity`**: consensus kernel, quorum logic, validator rotation, vote handling, and finality rules.
- **`aoxcnet`**: peer discovery, gossip, transport, sync, and resilience surfaces.
- **`aoxcrpc`**: HTTP, gRPC, and WebSocket access layers.
- **`aoxcmd`**: operator-facing bootstrap, runtime, diagnostics, and CLI workflows.
- **`aoxcvm`**: multi-lane execution compatibility for native, EVM, WASM, and external lanes.

The repository also includes deterministic testnet fixtures, operations scripts, architecture documentation, and release/readiness material under `configs/`, `scripts/`, and `docs/`.

---

## 2. Recommended release naming

I recommend using the following release convention:

- **Marketing / operator label:** `AOXC v0.1.1-akdeniz`
- **Cargo / package version:** `0.1.1-akdeniz`
- **Documentation baseline label:** `aoxc.v.0.1.1-akdeniz`

Why this naming works:

1. `0.1.1` is still honest about maturity.
2. `akdeniz` gives the release line an identity without pretending the system is already final-mainnet.
3. It avoids being stuck forever in vague `alpha` wording, which the repository’s own versioning roadmap argues against.

---

## 3. Current workspace structure

### Core crates

| Crate | Role |
|---|---|
| `crates/aoxcore` | chain domain model, identity, genesis, mempool, receipts |
| `crates/aoxcunity` | consensus, fork choice, quorum, proposer logic |
| `crates/aoxcnet` | networking, gossip, sync, transport |
| `crates/aoxcrpc` | API ingress surfaces |
| `crates/aoxcmd` | node lifecycle and operator command plane |
| `crates/aoxcvm` | execution lane compatibility |

### Supporting crates

| Crate | Role |
|---|---|
| `aoxcdata` | storage and state persistence |
| `aoxcontract` | contract metadata and validation |
| `aoxconfig` | chain and contract config models |
| `aoxckit` | keyforge/operator utility commands |
| `aoxcsdk` | SDK-facing helper APIs |
| `aoxcai` | policy/AI extension interfaces |
| `aoxcenergy`, `aoxclibs`, `aoxcexec`, `aoxcmob`, `aoxchal` | supporting runtime, utility, and integration surfaces |

---

## 4. Production goals for the Akdeniz baseline

The `v0.1.1-akdeniz` line should mean:

- deterministic workspace behavior is preserved,
- node bootstrap and operator workflows are testable,
- key custody and lifecycle surfaces have explicit verification paths,
- documentation is rich enough for operators, reviewers, and contributors,
- release-readiness claims are tied to evidence, not only aspirations.

This repository already contains the foundation for that through:

- release/readiness planning in `docs/src/`,
- deterministic environment bundles in `configs/environments/`,
- operator scripts in `scripts/`,
- integration coverage in `tests/`.

---

## 5. Quick start

### Requirements

- Rust toolchain with `cargo`
- `git`
- Linux or macOS recommended
- enough disk space for target artifacts and local fixtures

### Clone

```bash
git clone <your-repo-url> aoxchain
cd aoxchain
```

### Format and test

```bash
cargo fmt --all
cargo test -p aoxcmd
```

### Explore the operator CLI

```bash
cargo run -p aoxcmd --bin aoxc -- --help
```

---

## 6. Operator bootstrap flow

Typical local operator flow:

```bash
# 1) create or inspect the data home
export AOXC_HOME="$PWD/.aoxc-local"

# 2) bootstrap operator key material
cargo run -p aoxcmd --bin aoxc -- bootstrap key \
  --name validator-01 \
  --profile testnet \
  --password 'Test#2026!'

# 3) bootstrap runtime state
cargo run -p aoxcmd --bin aoxc -- bootstrap node

# 4) produce a deterministic sample block
cargo run -p aoxcmd --bin aoxc -- produce-once --tx demo-tx
```

If the exact CLI surface changes, the canonical source of truth is `crates/aoxcmd/src/cli/`.

---

## 7. Deterministic testnet and local validation

Useful entry points:

- `configs/environments/localnet/`
- `configs/environments/testnet/`
- `configs/environments/localnet/launch-localnet.sh`
- `scripts/run-local.sh`
- `scripts/validation/multi_host_validation.sh`

Suggested validation order:

1. local single-node bootstrap,
2. deterministic testnet fixture launch,
3. targeted crate tests,
4. multi-node and cross-host validation,
5. readiness evidence update.

---

## 8. Documentation map

Start here:

- `docs/src/READ.md` — documentation entry and release baseline
- `docs/src/SUMMARY.md` — mdBook navigation
- `docs/src/AKDENIZ_RELEASE_BASELINE.md` — Akdeniz release definition
- `docs/src/MAINNET_READINESS_CHECKLIST.md` — readiness expectations
- `docs/src/REAL_NETWORK_VALIDATION_RUNBOOK_TR.md` — real-network validation
- `docs/src/AOXC_REAL_VERSIONING_AND_RELEASE_ROADMAP_TR.md` — versioning policy

---

## 9. Quality gates

Recommended minimum release gates:

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings
cargo test --workspace --exclude aoxchub --all-targets
# desktop surface (requires Linux desktop system dependencies):
cargo check -p aoxchub --all-targets
```

For this change set, the focused verification path remained:

```bash
cargo fmt --all
cargo test -p aoxcmd
```

---

## 10. Release evidence expectations

Every named release should capture:

- exact commit SHA,
- exact commands executed,
- test outcomes,
- documentation changes,
- known limitations and blockers,
- artifact and provenance policy status.

---

## 11. Known reality

This repository has strong architecture and documentation breadth, but some production-readiness areas still require additional closure, especially:

- real cross-host validation,
- state sync / recovery proof,
- broader workspace-wide release gates,
- full integration maturity across all crates.

The goal of the Akdeniz line is to make the release surface coherent and evidence-driven, not to hide remaining work.
