# AOXChain (aoxc/aoxchain)

AOXChain is a multi-crate Rust workspace organized as a sovereign modular chain stack.

This README is architecture-first and evidence-oriented. It distinguishes current implementation from target-state intent.

## Repository status (current-state honesty)

- **Currently implemented:** kernel/consensus crates, runtime crates, network/RPC/data services, CLI tooling, desktop operator surface, SDK/peripheral crates.
- **Partially implemented:** complete cross-lane replay evidence, full partition/recovery evidence bundles, and uniform control-plane audit evidence packaging.
- **Target state:** release gates that enforce deterministic replay, boundary-policy checks, and incident-grade evidence packaging across all critical tiers.

No “100% complete” production claim is made in this document.

## Architecture map

| Domain | Primary crates/surfaces | Authority model |
|---|---|---|
| Kernel | `aoxcore`, `aoxcunity` | Consensus/state-transition authority (deterministic, release-blocking). |
| Runtimes | `aoxcexec`, `aoxcvm`, `aoxcenergy` | Deterministic execution policy/orchestration; feeds kernel outcomes. |
| System Services | `aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`, `aoxclibs`, `aoxchal` | Availability, ingress, persistence, config, shared infra. |
| Operator Environment | `aoxcmd`, `aoxckit`, `aoxchub` | Human control plane; non-consensus authority. |
| Applications / Peripheral | `aoxcsdk`, `aoxcontract`, `aoxcai`, `aoxcmob` | Ecosystem and integration surfaces outside kernel authority. |

## Critical boundary rules

1. Kernel and deterministic runtime paths are consensus-sensitive.
2. UI/control-plane surfaces are operator-only and must remain command-transparent.
3. `aoxchub` is not protocol authority and must never become a hidden consensus path.
4. Non-deterministic inputs (network/RPC/operator/AI) must be normalized before deterministic execution.

## Release tier overview

- **Tier 0 (Consensus Critical):** `aoxcore`, `aoxcunity`
- **Tier 1 (Deterministic Runtime Critical):** `aoxcexec`, `aoxcvm`, `aoxcenergy`
- **Tier 2 (Network/Availability/Persistence):** `aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`, `aoxclibs`, `aoxchal`
- **Tier 3 (Operator/Control Plane):** `aoxcmd`, `aoxckit`, `aoxchub`
- **Tier 4 (Application/Peripheral):** `aoxcsdk`, `aoxcontract`, `aoxcai`, `aoxcmob`

See `docs/RELEASE_TIERS.md` for full rationale.

## Documentation index (root architecture set)

- `docs/ARCHITECTURE.md`
- `docs/SECURITY_MODEL.md`
- `docs/EXECUTION_MODEL.md`
- `docs/STATE_MODEL.md`
- `docs/RELEASE_TIERS.md`
- `docs/SYSTEM_INVARIANTS.md`
- `docs/LICENSING.md`
- `docs/TRADEMARK_POLICY.md`
- `READ.md` (audit companion)

## Minimal verification commands

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings
cargo test --workspace --exclude aoxchub --all-targets
cargo check -p aoxchub --all-targets
```

## License

Repository code is licensed under **AGPL-3.0-only** (see `LICENSE` and `docs/LICENSING.md`).

Trademark and brand usage are governed separately by `docs/TRADEMARK_POLICY.md`.
