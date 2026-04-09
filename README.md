# AOXChain

<p align="center">
  <img src="logos/aoxc_transparent.png" alt="AOXChain logo" width="220" />
</p>

<p align="center">
  Deterministic Layer-1 engineering with policy-governed authority, cryptographic agility, and evidence-gated release discipline.
</p>

> [!WARNING]
> **Experimental repository:** AOXChain is under active development.
> Treat this codebase as experimental unless a specific release artifact and readiness evidence explicitly declare production suitability.

## 1) What AOXChain is

AOXChain is a Rust workspace for building and operating a deterministic blockchain stack with explicit governance surfaces.

Primary goals:

- deterministic execution and replayable behavior,
- explicit trust boundaries and operational controls,
- staged classical-to-post-quantum migration readiness,
- release assertions backed by verifiable artifacts.

## 2) Who this repository is for

- **Node operators:** install, verify, activate, and run AOXC runtime surfaces.
- **Protocol/runtime developers:** work across kernel, VM, networking, and tooling crates.
- **Release/reliability engineers:** run readiness gates and generate evidence bundles.
- **Security reviewers:** inspect scope, architecture, compatibility, and disclosure posture.

## 3) Repository map

- `crates/` — workspace crates (`aoxcmd`, `aoxcvm`, `aoxcnet`, kernel crates, tooling).
- `configs/` — canonical environment/runtime source bundles and topology policies.
- `docs/` — runbooks, matrixes, plans, and deep technical references.
- `scripts/` — operational automation, gates, orchestration, and release utilities.
- `tests/` — external/readiness integration surfaces.
- `artifacts/` — generated evidence outputs and closure snapshots.
- `models/` — machine-readable governance/readiness models.
- `contracts/` — contract/system integration references.

## 4) Canonical governance documents

Read these first for policy-sensitive work:

- `READ.md` — repository technical contract and invariants.
- `SCOPE.md` — in-scope/out-of-scope and sensitive-change boundaries.
- `ARCHITECTURE.md` — major components, dependency direction, trust boundaries.
- `TESTING.md` — required validation and readiness criteria.
- `SECURITY.md` — disclosure and security posture expectations.
- `ROADMAP.md` — phased execution and closure milestones.
- `VERSIONING.md` — compatibility/versioning posture.
- `CONTRIBUTING.md` — contribution and review workflow.

## 5) Quick start (developer)

### 5.1 Prerequisites

- Rust + Cargo
- GNU Make
- Bash
- Git

### 5.2 Build key binaries

```bash
cargo build -p aoxcmd --release
cargo build -p aoxchub --release
cargo build -p aoxckit --release
```

### 5.3 Discover available command surfaces

```bash
cargo run -p aoxcmd --bin aoxc -- --help
cargo run -p aoxchub -- --help
cargo run -p aoxckit -- --help
make help
```

### 5.4 Baseline quality and readiness gates

```bash
make fmt
make check
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

## 6) Quick start (operator)

### 6.1 Runtime lifecycle via Make

```bash
make env-check
make runtime-source-check
make runtime-install
make runtime-verify
make runtime-activate
make runtime-status
make runtime-doctor
```

### 6.2 Runtime daemon flow

```bash
./scripts/runtime_daemon.sh start
./scripts/runtime_daemon.sh status
./scripts/runtime_daemon.sh tail
./scripts/runtime_daemon.sh stop
```

### 6.3 Persistent service install (Linux/systemd)

```bash
./scripts/runtime_daemon.sh install-service
systemctl --user start aoxc-runtime.service
```

### 6.4 Useful operational wrappers

```bash
./scripts/preflight_check.sh
./scripts/validator_bootstrap.sh
./scripts/finality_smoke.sh
./scripts/transfer_smoke.sh
./scripts/runtime_recover.sh
```

## 7) AOXCHub (localhost control plane)

AOXCHub provides a localhost-only operator UI for approved AOXC/Make workflows.

```bash
cargo run -p aoxchub
# then open http://127.0.0.1:7070
```

Alternative launch surface from Make:

```bash
make ui
```

## 8) Configuration and environment policy

Canonical runtime source material is managed under:

- `configs/environments/mainnet/`
- `configs/environments/testnet/`
- `configs/environments/deterministic-testnet/`

These files are operationally sensitive. Modifications can change network identity, runtime source integrity, topology behavior, and compatibility assumptions.

## 9) Testing and release evidence model

AOXChain readiness uses layered validation:

- crate-level unit/integration tests,
- repository integration/readiness tests,
- scripted validation gates under `scripts/validation/`,
- release/evidence generation workflows under `scripts/release/` and `artifacts/`.

A local successful build is necessary but not sufficient for production claims.

## 10) Change-discipline rules

If a change affects authority model, consensus semantics, trust boundaries, compatibility, or operational policy:

1. update implementation,
2. update corresponding governance docs,
3. include explicit migration/rollback implications,
4. include reproducible validation evidence.

Silent architecture-policy drift is not acceptable.

## 11) Security posture

- Follow `SECURITY.md` for reporting and handling security issues.
- Keep trust-boundary assumptions explicit in code and docs.
- Do not merge compatibility-sensitive changes without test and evidence context.

## 12) License and liability

AOXChain is distributed under the MIT License.

Unless required by applicable law or agreed in writing, software is provided **"AS IS"**, without warranties or liability assumptions by maintainers/contributors.

---

If you are new to the repository, begin with `READ.md` and `ARCHITECTURE.md`, then use crate-local `README.md` / `SCOPE.md` / `ARCHITECTURE.md` documents for the subsystem you are modifying.
