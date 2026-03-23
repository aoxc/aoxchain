# AOX Chain

**Release Baseline:** `aoxc.v.0.1.0-testnet.1`
**Cargo Workspace Version:** `0.1.0-testnet.1`

AOX Chain is a Rust workspace for a multi-component blockchain platform. The repository combines core protocol modeling, deterministic consensus, multi-VM execution, networking, RPC gateways, storage abstractions, operator tooling, mobile security utilities, SDK support, and AI-governance extensions. This root document is intended to be the primary entry point for engineers, auditors, operators, and release managers who need to understand how the system is organized and how it is expected to be operated.

## Architectural Overview
The workspace is partitioned into crates and operational folders that each own a specific part of the system.

### Core protocol and consensus
- **`crates/aoxcore`** defines the canonical protocol vocabulary for blocks, transactions, identities, genesis data, receipts, contracts, state, and mempool-facing primitives.
- **`crates/aoxcunity`** implements the consensus kernel, including validator rotation, block admission, vote validation, quorum logic, finality, safety policy, timeout handling, and recovery-oriented state transitions.

### Execution and runtime orchestration
- **`crates/aoxcvm`** coordinates native and external execution lanes such as EVM, Wasm, Sui Move, Cardano, and system flows.
- **`crates/aoxcexec`** provides execution-facing coordination interfaces that connect runtime services to execution engines.
- **`crates/aoxchal`** contains lower-level host and hardware-aware support utilities.
- **`crates/aoxcenergy`** models execution-economics primitives used for bounded runtime accounting.

### Networking and client interfaces
- **`crates/aoxcnet`** owns peer discovery, gossip, transport, ports, sync behavior, and network configuration.
- **`crates/aoxcrpc`** exposes HTTP, gRPC, WebSocket, and middleware-based client API surfaces.
- **`crates/aoxcmob`** provides mobile and edge-side secure signing, session, gateway, and transport abstractions.

### Storage, configuration, and contracts
- **`crates/aoxcdata`** contains state persistence and contract storage abstractions.
- **`crates/aoxconfig`** provides typed configuration models for blockchain and contract settings.
- **`crates/aoxcontract`** defines the canonical contract manifest, policy, artifact, compatibility, and validation model.
- **`crates/aoxcsdk`** contains developer-facing integration helpers and contract-building support.

### Operator workflows and governance
- **`crates/aoxcmd`** is the operational binary and library crate used to bootstrap, run, inspect, and audit nodes.
- **`crates/aoxckit`** provides cryptographic and identity-lifecycle tooling.
- **`crates/aoxclibs`** contains shared low-level utility helpers.
- **`crates/aoxcai`** defines AI-governance, capability policy, manifest, audit, and backend integration layers.

## Repository Layout
The root folders are organized for both implementation work and operational traceability.

- `README.md`: the high-level system overview and command guide.
- `READ.md`: the root production audit guide.
- `VERSION.md`: root version-governance rules and change ledger.
- `crates/`: the Rust workspace packages that implement protocol, runtime, networking, storage, tooling, and integration behavior.
- `tests/`: cross-crate integration validation flows.
- `docs/`: runbooks, readiness plans, architecture notes, and operational guidance.

- `docs/RELEASE_AND_PROVENANCE_RUNBOOK.md`: release evidence, SBOM, signing, and provenance workflow.
- `docs/BACKUP_RESTORE_AND_ROLLBACK_RUNBOOK.md`: recovery and rollback expectations.
- `docs/THREAT_MODEL_AND_ATTACK_SURFACE.md`: threat boundary summary for audit and release review.
- `docs/RELEASE_OWNERSHIP_AND_ESCALATION.md`: role and escalation ownership for releases.
- `configs/`: network, genesis, and deterministic testnet configuration packs.
- `scripts/`: operational automation, validation harnesses, and release helpers.
- `models/`: machine-readable risk and governance models.
- `contracts/system/`: privileged or system-level contract artifacts.

## Security Design Priorities
AOX Chain is being documented and hardened toward a 99.99% production-readiness target. The dominant security priorities are:

- deterministic execution and hashing,
- explicit error propagation instead of panic-driven control flow,
- bounded resource usage for execution, storage, and network-facing paths,
- strict trust-boundary clarity between crates and operator surfaces,
- version traceability across source, manifests, binaries, tests, and audit records,
- release procedures that are observable, reproducible, and reviewable.

## Build and Release Profile
The workspace release profile is configured for production-oriented builds with:

- `panic = "abort"`,
- `lto = true`,
- `codegen-units = 1`.

These settings are intended to align release binaries with deterministic and optimized production expectations defined in the workspace manifest.

## Primary Commands
The main day-to-day operational commands are exposed through the `Makefile` and the `aoxc` binary built from `crates/aoxcmd`.

### Developer quality gates
| Command | Purpose |
| --- | --- |
| `make fmt` | Format the entire workspace. |
| `make check` | Run `cargo check --workspace`. |
| `make test` | Run `cargo test --workspace`. |
| `make clippy` | Run Clippy across all workspace targets and features. |
| `make quality-quick` | Run the quick quality gate from `scripts/quality_gate.sh`. |
| `make quality` | Run the full quality gate. |
| `make quality-release` | Run the release-oriented quality gate. |

### Build and release commands
| Command | Purpose |
| --- | --- |
| `make build` | Build the full workspace in debug mode. |
| `make build-release` | Build the release AOXC CLI binary. |
| `make package-bin` | Copy the release binary to `./bin/aoxc`. |
| `make version` | Print version/build information via the CLI. |
| `make manifest` | Print the build manifest and supply-chain policy. |
| `make policy` | Print node connection policy information. |

### Local workflow and node operation
| Command | Purpose |
| --- | --- |
| `make dev-bootstrap` | Print a suggested local bootstrap sequence. |
| `make run-local` | Package the binary and execute the local helper script. |
| `make supervise-local` | Run the local supervisor helper. |
| `make produce-loop` | Run the continuous producer helper. |
| `make real-chain-prep` | Prepare local directories for a bounded real-chain workflow. |
| `make real-chain-run-once` | Run one bounded real-chain daemon cycle. |
| `make real-chain-run` | Run the local real-chain daemon loop. |
| `make real-chain-health` | Execute a network smoke/health probe. |
| `make real-chain-tail` | Tail the real-chain runtime and health logs. |

## Suggested Local Development Flow
A practical baseline flow for a contributor is:

1. Format and validate the workspace:
   - `make fmt`
   - `make check`
   - `make test`
2. Run linting:
   - `make clippy`
3. Build the release CLI if local operator testing is needed:
   - `make package-bin`
4. Bootstrap a local environment using the guidance printed by:
   - `make dev-bootstrap`
5. If needed, run a local smoke flow:
   - `make run-local`
   - `make real-chain-run-once`
   - `make real-chain-health`

## CLI and Operational Entry Points
The `aoxc` binary is the main operational interface. The `Makefile` shows several representative commands that are important when evaluating the system:

- `cargo run -p aoxcmd -- version`
- `cargo run -p aoxcmd -- build-manifest`
- `cargo run -p aoxcmd -- node-connection-policy`
- `cargo run -p aoxcmd -- key-bootstrap ...`
- `cargo run -p aoxcmd -- genesis-init ...`
- `cargo run -p aoxcmd -- node-bootstrap ...`
- `cargo run -p aoxcmd -- produce-once ...`

These commands are meaningful because they reflect the operator lifecycle from key material initialization to genesis creation, node bootstrap, and bounded transaction/block production.

## Local Helper Scripts
The repository includes several operational helper scripts under `scripts/`. Representative examples include:

- `scripts/run-local.sh`: package-binary smoke helper for local bootstrap and production of a sample transaction.
- `scripts/node_supervisor.sh`: local supervision helper.
- `scripts/quality_gate.sh`: quality-gate orchestration.
- `scripts/real_chain_daemon.sh`: bounded local real-chain loop.
- `scripts/release_artifact_certify.sh`: release artifact certification helper.

These scripts should be treated as part of the security and release surface, not as informal utilities.

## Verification Strategy
The workspace aims to support multiple evidence layers.

1. Unit tests for individual validation and state-transition rules.
2. Integration tests for cross-crate flows.
3. Adversarial and hack-style tests for hostile conditions.
4. Fuzz-style repetition for deterministic and parser-sensitive paths.
5. Lint, formatting, and release-gating checks enforced before promotion.

For high-risk logic such as consensus, identity, configuration, and contract policy, reviewers should expect stronger-than-minimal evidence.

## Versioning and Release Governance
The canonical documentation version is `aoxc.v.0.1.0-testnet.1` and the Cargo-compatible semantic version is `0.1.0-testnet.1`. The repository treats documentation, manifests, binaries, compatibility fixtures, and verification evidence as a single release package. A material change is expected to update the relevant version and ledger documentation together.

For the strict governance rules, read:
- `READ.md`
- `VERSION.md`
- the folder-level `READ.md` and `VERSION.md` files for the subsystem being changed

## Where to Read Next
Use the following sequence when onboarding or auditing the repository:

1. `README.md` for the high-level architecture and command map.
2. `READ.md` for the root production audit guide.
3. `VERSION.md` for root-level release governance.
4. `crates/README.md` for the crate portfolio map.
5. the relevant subsystem `README.md`, `READ.md`, and `VERSION.md` before making or reviewing a change.

## Security Audit Log
- `aoxc.v.0.1.0-testnet.1`: root README expanded into a full system-level guide with subsystem mapping, command documentation, operational entry points, and release-governance context.
- `aoxc.v.0.0.0-alpha.2`: initial full audit-roadmap baseline introduced for production tracking.

## Audit Checklist
- [ ] Memory leaks and ownership assumptions reviewed.
- [ ] Race-condition and async integrity risks reviewed.
- [ ] Error propagation remains explicit.
- [ ] Version metadata and compatibility declarations are synchronized.
- [ ] Verification evidence is attached to the release record.
