# AOX Chain

**Release Baseline:** `aoxc.v.0.0.0-alpha.3`
**Cargo Workspace Version:** `0.0.0-alpha.3`

AOX Chain is a Rust workspace for a multi-component blockchain platform. The repository combines protocol modeling, consensus, execution, networking, storage, RPC gateways, SDK surfaces, operator tooling, mobile-facing security utilities, and AI-governance extensions. This README is intentionally detailed so that a reviewer can understand the system shape before drilling into individual crates or operational folders.

## Architectural Overview
The workspace is organized around clear responsibility boundaries.

- **`crates/aoxcore`** contains the foundational protocol models for blocks, transactions, identities, genesis, contracts, receipts, mempool state, and related core primitives.
- **`crates/aoxcunity`** contains the consensus kernel responsible for validator rotation, vote admission, finality, safety checks, and recovery-oriented state transitions.
- **`crates/aoxcvm`** contains the multi-VM execution layer for native, EVM, Wasm, Sui Move, Cardano, and system lanes.
- **`crates/aoxcnet`** and **`crates/aoxcrpc`** expose network and API surfaces, covering peer communication, gossip, gRPC, HTTP, WebSocket, and middleware.
- **`crates/aoxcmd`** is the operational binary and CLI surface used to bootstrap, run, inspect, and audit nodes.
- **`crates/aoxcdata`**, **`crates/aoxconfig`**, **`crates/aoxcontract`**, **`crates/aoxcsdk`**, **`crates/aoxcmob`**, **`crates/aoxckit`**, **`crates/aoxclibs`**, **`crates/aoxcexec`**, **`crates/aoxchal`**, **`crates/aoxcenergy`**, and **`crates/aoxcai`** fill storage, configuration, contract policy, SDK, mobile, crypto-tooling, shared utility, execution, hardware, economics, and AI-governance roles.

## Repository Layout
- `README.md`: root system overview and audit-friendly entry point.
- `READ.md`: root production audit guide with release discipline and evidence expectations.
- `VERSION.md`: root version governance rules and ledger.
- `crates/`: Rust packages implementing the product surface.
- `tests/`: cross-workspace validation flows.
- `docs/`: runbooks, readiness plans, operational manuals, and architecture notes.
- `configs/`: environment and network configuration bundles.
- `scripts/`: validation, certification, and operational automation helpers.
- `models/`: machine-readable governance and risk model definitions.
- `contracts/system/`: privileged or system-level contract artifacts.

## Security Design Priorities
AOX Chain is being documented and hardened toward a 99.99% production-readiness target. The dominant security themes are:
- deterministic execution and hashing,
- explicit error propagation instead of panic-driven control flow,
- bounded resource usage for network, storage, and execution paths,
- trust-boundary clarity between crates and operator surfaces,
- version and release traceability that connects source, tests, binaries, and documentation.

## Verification Strategy
The workspace aims to support multiple evidence layers.
1. Unit tests for individual validation and state-transition rules.
2. Integration tests for cross-crate flows.
3. Adversarial and hack-style tests for hostile conditions.
4. Fuzz-style repetition for deterministic and parser-sensitive paths.
5. Lint, formatting, and release-gating checks enforced before promotion.

## Versioning and Release Governance
The canonical documentation version is `aoxc.v.0.0.0-alpha.3` and the Cargo-compatible semantic version is `0.0.0-alpha.3`. Every significant change must advance the documented release baseline, update affected compatibility declarations, and record the reason for the version increase. This repository now treats documentation, manifests, and operational evidence as one release package rather than independent concerns.

## Where to Read Next
- Start with `READ.md` for the root audit guide.
- Open `VERSION.md` for release-governance rules.
- Then read the relevant folder-level `READ.md` and `VERSION.md` files before changing or reviewing a subsystem.

## Security Audit Log
- `aoxc.v.0.0.0-alpha.3`: root README expanded into a substantive system guide, root and folder-level audit documents upgraded from template-style text to folder-specific descriptions, and the release baseline advanced to alpha.3.
- `aoxc.v.0.0.0-alpha.2`: initial full audit-roadmap baseline introduced for production tracking.

## Audit Checklist
- [ ] Memory leaks and ownership assumptions reviewed.
- [ ] Race-condition and async integrity risks reviewed.
- [ ] Error propagation remains explicit.
- [ ] Version metadata and compatibility declarations are synchronized.
- [ ] Verification evidence is attached to the release record.
