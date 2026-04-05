# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is a deterministic Layer-1 engineering program focused on production-grade operation, cryptographic agility, and evidence-backed readiness.

## Program Direction

AOXChain follows a two-stage strategy:

1. **Production-Grade Testnet** — operate testnet with mainnet discipline.
2. **PQ-Resilient Mainnet** — activate mainnet only after cryptographic and operational controls are proven.

The repository intentionally avoids unverifiable claims.

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

---

## 🛠 Developer Path (Make-First)

This section is for contributors and CI-focused workflows.

### Core quality and readiness targets

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
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

## Canonical Documents

- `READ.md` — repository-level technical contract and invariants.
- `SCOPE.md` — in-scope/out-of-scope and compatibility posture.
- `ARCHITECTURE.md` — component boundaries and dependency direction.
- `SECURITY.md` — security posture and disclosure model.
- `TESTING.md` — validation policy and readiness gates.
- `ROADMAP.md` — strategic roadmap and phase gates.
- `docs/NAMING_VERSIONING_SIMPLIFICATION_PLAN.md` — naming/versioning baseline and migration policy.
- `docs/GENESIS_IDENTITY_CHECKLIST.md` — genesis and environment identity consistency checklist.

## Identity and Versioning Quick Reference

Use this vocabulary consistently across code, docs, and operations:

| Term | Meaning | Authority |
|---|---|---|
| **Brand** | Product/system name (`AOXChain`) | repository documentation |
| **Ticker** | Native asset symbol (`AOXC`) | protocol/economic docs |
| **Release line** | Human-facing release stream label (for example `AOXC-QTR-V1`) | release notes + tags |
| **Workspace version** | Build/package/release metadata version | `configs/version-policy.toml` |
| **Chain ID** | Deterministic machine identity (numeric) | `configs/registry/network-registry.toml` |
| **Network ID** | Human-readable network identity string | `configs/registry/network-registry.toml` |
| **Crypto profile** | Consensus-visible cryptography mode/version | topology and profile policy |

Rules:

1. Do not use release-line labels as `chain_id` or `network_id`.
2. Do not derive protocol truth from Git tags alone.
3. Keep machine identity policy in repository-controlled, reviewable files.

## Identity and Versioning Quick Reference

Use this vocabulary consistently across code, docs, and operations:

| Term | Meaning | Authority |
|---|---|---|
| **Brand** | Product/system name (`AOXChain`) | repository documentation |
| **Ticker** | Native asset symbol (`AOXC`) | protocol/economic docs |
| **Release line** | Human-facing release stream label (for example `AOXC-QTR-V1`) | release notes + tags |
| **Workspace version** | Build/package/release metadata version | `configs/version-policy.toml` |
| **Chain ID** | Deterministic machine identity (numeric) | `configs/registry/network-registry.toml` |
| **Network ID** | Human-readable network identity string | `configs/registry/network-registry.toml` |
| **Crypto profile** | Consensus-visible cryptography mode/version | topology and profile policy |

Rules:

1. Do not use release-line labels as `chain_id` or `network_id`.
2. Do not derive protocol truth from Git tags alone.
3. Keep machine identity policy in repository-controlled, reviewable files.

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
