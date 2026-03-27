# AOXC (Advanced Omnichain Execution Core)

> Release Track: `v0.01-foundation`  
> Status: `Experimental / Not Production Ready`  
> License: `MIT`

AOXC is a modular blockchain runtime and infrastructure workspace focused on deterministic execution, consensus safety, and operator-grade delivery discipline.

---

## Executive Summary

AOXC targets an advanced chain architecture built around:

1. **Deterministic transaction processing** with explicit payload and signature discipline.
2. **Consensus safety and finality controls** with measurable invariants.
3. **Multi-runtime execution strategy** for heterogeneous workloads.
4. **Operational excellence**: reproducible builds, observability, incident readiness.
5. **Cross-platform parity**: Linux, macOS, Windows, and Docker-first workflows.

---

## Repository Architecture

- `crates/aoxcore` → transaction/block/state/identity domain primitives
- `crates/aoxcunity` → consensus engine, finality, safety surfaces
- `crates/aoxcvm` → execution runtime lanes and routing
- `crates/aoxcnet` → networking, discovery, resilience tooling
- `crates/aoxcrpc` → gRPC/HTTP/WebSocket API surfaces
- `crates/aoxcmd` → node bootstrap and operations flow
- `tests` → integration + readiness scenarios
- `configs` → environment and network configuration
- `scripts` → operational and release automation

---

## Platform Compatibility Matrix

| Platform | Supported | Notes |
|---|---:|---|
| Linux (Ubuntu/Debian/Fedora) | ✅ | Primary development target |
| macOS (Intel/Apple Silicon) | ✅ | CI-compatible workflow |
| Windows (PowerShell/WSL2) | ✅ | Use rustup + native shell guidance |
| Docker | ✅ | Preferred for reproducible verification |

---

## Prerequisites

- Rust stable toolchain (`rustup` recommended)
- Cargo workspace support
- Git
- Optional: Docker / Docker Compose

---

## Quick Start

```bash
git clone <repo-url> aoxchain
cd aoxchain
cargo build --workspace
```

### Mandatory validation commands

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings
cargo test --workspace --exclude aoxchub --all-targets
cargo check -p aoxchub --all-targets
```

---

## Docker-First Workflow

Use Docker to guarantee deterministic build/test context across contributors and CI.

Recommended policy:

1. Validate every release candidate in a clean container.
2. Store reproducible command runbooks for all mandatory gates.
3. Keep host-specific assumptions out of scripts.

---

## Engineering Quality Contract

No change is considered complete unless all are satisfied:

- Determinism impact assessed.
- Backward compatibility impact documented.
- Tests added/updated where behavior changed.
- Validation gates pass.
- Operational implications documented.

---

## Security and Readiness Expectations

- Treat cryptography, consensus, and networking changes as high-risk.
- Add threat notes for sensitive surfaces.
- Prefer explicit invariants and fail-fast validation.
- Maintain release evidence and rollback readiness.

---

## Primary Planning and Execution Documents

- [`ROADMAP.md`](./ROADMAP.md) → phase plan, checklist gates, delivery milestones
- [`READ.md`](./READ.md) → mirrored canonical entry document

---

## License

MIT License.
All contributed code and documentation must remain license-compliant, and third-party obligations must be tracked.
