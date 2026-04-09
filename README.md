# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is a deterministic Layer-1 engineering program focused on operational discipline, cryptographic agility, and evidence-backed readiness.

## Why AOXChain and How It Differs

AOXChain is intentionally positioned as an engineering-first protocol program rather than a throughput-marketing project. The design center differs from common L1 narratives in three ways:

- **Determinism before velocity:** consensus, execution, and release claims are governed by reproducible gates and retained artifacts.
- **Crypto-agility by policy:** cryptographic profile evolution (including post-quantum migration) is treated as a governed protocol surface, not an implicit library upgrade.
- **Operational evidence as a release requirement:** channel promotion is bound to explicit readiness criteria, not informal confidence.

This repository does not claim to replace Ethereum, Solana, or other ecosystems by slogan. Instead, it focuses on building a conservative, audit-oriented L1 posture where trust boundaries, failure modes, and activation conditions are explicit.

## ⚠️ Experimental Program Notice (Read First)

AOXChain is currently an **experimental engineering system** under active development.

- It is suitable for engineering evaluation, deterministic testing, and controlled testnet operation.
- It should **not** be interpreted as an unconditional production guarantee.
- Behavior, interfaces, defaults, and operational controls may evolve as gaps are closed.

AOXChain is distributed under the **MIT License** with standard warranty and liability limitations. Operators, integrators, and contributors are responsible for independent validation before high-stakes use.

## 🧭 Current State and Strategic Target

### Current state
- Deterministic node lifecycle and operator CLI surfaces are available.
- Environment identity and readiness gates are implemented for dev/test/main channels.
- Repository-level production-gap closure is actively tracked and audited.

### Strategic target
- **Production-grade testnet discipline** as an intermediate control stage.
- **Post-quantum (PQ) resilient mainnet posture** as a long-term activation target, enabled only after cryptographic, operational, and validation gates are satisfied.

In short: AOXChain is built toward quantum-resilient operation, but is deliberately run with conservative, evidence-first activation policy.

## Repository Layout Note

Kernel-layer crates are grouped under `crates/kernel/`:

- `crates/kernel/aoxcore`
- `crates/kernel/aoxcunity`

Execution, service, and operations crates remain under `crates/` and consume kernel-defined protocol truth through typed interfaces.

## Documentation Naming Policy (README vs READ)

- `README.md` is the canonical human-facing entry point for a directory or component.
- `READ.md` is reserved for compact technical contracts only when it adds non-duplicative operational value.
- If a `READ.md` does not provide unique constraints beyond an adjacent `README.md`, it should be merged or removed to prevent review ambiguity.

## 📊 Repository Health Snapshot (2026-04-07 UTC)

- `cargo check --workspace --locked`: pass
- `cargo test --workspace --exclude aoxchub --all-targets --locked`: pass
- Open production-gap items and closure actions: `docs/REPOSITORY_PRODUCTION_GAP_REPORT.md`

This snapshot is point-in-time engineering evidence, not a perpetual guarantee.

## 🛰️ Release Channel Model

AOXChain uses one codebase across three operational channels:

1. **Devnet (rolling development)** — CI-first iteration and engineering change velocity.
2. **Testnet (candidate validation)** — promotion-candidate hardening and operational evidence collection.
3. **Mainnet (stable target)** — enabled only after policy and readiness gates pass.

Channel identity is policy-driven through environment manifests and registry controls, not hardcoded per build.

---

## 📦 User Path (Binary-First)

This section is for operators who want to use the AOXC binary directly.

### 1) Build the AOXC binary

```bash
cargo build -p aoxcmd --release
```

Binary location:

```text
target/release/aoxc
```

### 2) Verify CLI surface

```bash
./target/release/aoxc --help
./target/release/aoxc version
```

### 3) Initialize local operator keys and genesis

```bash
./target/release/aoxc key-bootstrap --profile localnet --password '<strong-password>'
./target/release/aoxc genesis-init --profile localnet
./target/release/aoxc genesis-validate --strict
```

### 4) Run production-style gate checks before promotion

```bash
./target/release/aoxc genesis-production-gate
./target/release/aoxc role status --profile localnet
```

### 5) `production-bootstrap` first-run vs re-run (important)

When using `production-bootstrap`, use a **clean home directory** for each new bootstrap.

```bash
read -rsp "AOXC password: " AOXC_PASS; echo
./bin/aoxc production-bootstrap \
  --profile testnet \
  --password "$AOXC_PASS" \
  --name validator1 \
  --home /path/to/aoxc/home/testnet
unset AOXC_PASS
./bin/aoxc node start --home /path/to/aoxc/home/testnet
```

If you re-run bootstrap in the same `--home`, clear previous runtime state first:

```bash
rm -f /path/to/aoxc/home/testnet/runtime/db/main.redb
```

Otherwise startup can fail with a parent-hash mismatch (`AOXC-LED-001`) because
historical block rows from a previous run conflict with a newly bootstrapped
height-0 state.

### 6) Fast topology bootstrap (single command)

Use `topology-bootstrap` when you want one-command generation for single-node or four-node layouts:

```bash
# Interactive password mode (asks twice in terminal):
./target/release/aoxc topology-bootstrap \
  --mode single \
  --allocation-preset balanced \
  --profile testnet \
  --output-dir /tmp/aoxc-topology-single

# Explicit password mode (automation-friendly):
./target/release/aoxc topology-bootstrap \
  --mode mainchain-4 \
  --password '<strong-password>' \
  --allocation-preset validator-heavy \
  --output-dir /tmp/aoxc-topology-mainchain4

# Devnet four-node layout:
./target/release/aoxc topology-bootstrap \
  --mode devnet-4 \
  --password '<strong-password>' \
  --output-dir /tmp/aoxc-topology-devnet4
```

What this provides:
- deterministic per-node home directories and identity materials,
- per-node RPC/metrics/start/query hints in output,
- preset-based genesis allocation plans (`minimal`, `balanced`, `validator-heavy`),
- mainchain-4 and devnet-4 port-offset handling for same-host multi-node runs.

---

## 🛠 Developer Path (Make-First)

This section is for contributors and CI-focused workflows.

### Core quality and readiness targets

```bash
make build
make test
make quality
make repo-hygiene-gate
make audit
make testnet-gate
make testnet-readiness-gate
```

Signed release publication flow:

```bash
make repo-release-keygen
make repo-release-signed
make repo-release-signed-verify
```

Repository-independent testnet launch (operator release bundle flow):

```bash
aoxc config-init --profile testnet
aoxc key-bootstrap --profile testnet --password '<strong-password>'
aoxc genesis-init --profile testnet
aoxc genesis-validate --strict
aoxc genesis-production-gate
aoxc network-identity-gate --enforce --env testnet --format json
aoxc node start
```

### Why Make for developers

- keeps repeated engineering checks deterministic,
- keeps command surfaces aligned with CI,
- avoids drift between local and pipeline behavior.

---

## 🌐 Local Network Bring-Up (Node Formation + Connectivity)

Use this flow for deterministic local lifecycle checks.

### 1) Choose environment profile

```bash
export AOXC_NETWORK_KIND=localnet
```

### 2) Validate identity and bundle consistency

```bash
./target/release/aoxc genesis-validate --strict
./target/release/aoxc genesis-production-gate
```

### 3) Validate role topology before start

```bash
./target/release/aoxc role status --profile localnet
# Optional controlled rewrite to core7-only activation:
./target/release/aoxc role activate-core7 --profile localnet --dry-run
```

### 4) Start and inspect node/network health

```bash
./target/release/aoxc node start
./target/release/aoxc node status
./target/release/aoxc network status
./target/release/aoxc network verify
```

### 5) Query chain/API surfaces

```bash
./target/release/aoxc query chain status
./target/release/aoxc query network peers
./target/release/aoxc api status
```

### Minimum startup node counts (by environment)

`aoxc genesis-validate --strict` enforces environment-specific topology minimums.

| Environment | Minimum validators | Minimum bootnodes | Source of enforcement |
|---|---:|---:|---|
| `localnet` | 1 | 1 | Generic non-empty validator/bootnode checks |
| `testnet` | 3 | 2 | `testnet` validation guardrails |
| `mainnet` | 4 | 3 | `mainnet` validation guardrails |

Notes:
- `role activate-core7` controls role activation policy; it is not a validator-count override.
- `core7` means seven canonical role classes, not "seven validators required".

---

## Repository Layout

| Path | Purpose |
|---|---|
| `crates/` | Protocol, kernel, VM, network, service, and operator crates. |
| `configs/` | Runtime and network profile definitions. |
| `tests/` | Integration and adversarial validation suites. |
| `scripts/` | Automation and evidence workflows. |
| `docs/` | Technical and operational documentation surfaces. |
| `artifacts/` | Generated evidence and release/readiness artifacts. |
| `releases/` | Versioned, operator-consumable binary release bundles and verification metadata. |

## Canonical Documents

- `README.md` — repository landing page and operator/developer entry path.
- `READ.md` — repository-level technical contract and invariants.
- `WHITEPAPER.md` — engineering whitepaper with architecture and cryptographic-agility posture.
- `SCOPE.md` — in-scope/out-of-scope and compatibility posture.
- `ARCHITECTURE.md` — component boundaries and dependency direction.
- `SECURITY.md` — security posture and disclosure model.
- `TESTING.md` — validation policy and readiness gates.
- `docs/FULL_NODE_GUIDE.md` — step-by-step full-node installation, bootstrap, and network join guide.
- `docs/API_REFERENCE.md` — current HTTP and gRPC RPC surfaces with request/response examples.
- `docs/REPOSITORY_PRODUCTION_GAP_REPORT.md` — point-in-time production gap register and closure actions.
- `ROADMAP.md` — strategic roadmap and phase gates.
- `docs/NAMING_VERSIONING_SIMPLIFICATION_PLAN.md` — naming/versioning baseline and migration policy.
- `docs/GENESIS_IDENTITY_CHECKLIST.md` — genesis and environment identity consistency checklist.
- `docs/TESTNET_RELEASE_RUNBOOK.md` — full repository-independent testnet launch and CLI operations runbook.

---

## 🧱 Container Runtime (Docker + Podman)

AOXChain container surfaces are maintained to run on both Docker and Podman.

### Build image

```bash
docker build -t aoxchain-node:local .
# or
podman build -t aoxchain-node:local .
```

### Run a single node

```bash
docker run --rm -p 26656:26656 -p 8545:8545 aoxchain-node:local
# or
podman run --rm -p 26656:26656 -p 8545:8545 aoxchain-node:local
```

### Run local multi-node topology

```bash
docker compose up --build
# or
podman compose up --build
```

Additional Podman notes are documented in `PODMAN.md`.

## Compatibility and Change Discipline

Compatibility-impacting changes must include explicit rationale, migration guidance when required, and synchronized documentation updates.

## License

AOXChain is distributed under the [MIT License](./LICENSE), provided **"as is"** without warranties except where prohibited by applicable law.
