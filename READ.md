# AOXChain Audit Companion

This file is the audit-oriented companion to `README.md` and the root architecture/security/state model documents.

## 1) Audit scope statement

The repository should be reviewed as a modular chain stack with explicit boundaries:

- kernel authority,
- deterministic runtime authority,
- system service availability surfaces,
- operator control-plane surfaces,
- ecosystem/peripheral surfaces.

## 2) Current-state honesty summary

- **Currently implemented:** clear multi-crate decomposition across core, consensus, runtime, networking, RPC, persistence, operator CLI/UI, and integration crates.
- **Partially implemented:** uniform end-to-end evidence packaging for replay determinism, partition recovery, and control-plane mutating-action audit trails.
- **Target state:** release-time policy checks that prevent boundary violations and require reproducible evidence bundles for tier-0 through tier-3 changes.

## 3) Audit-critical boundary assertions

1. `aoxcore` + `aoxcunity` define kernel authority.
2. `aoxcexec` + `aoxcvm` (with `aoxcenergy`) define deterministic runtime authority surfaces.
3. `aoxcmd` remains the authoritative operational shell.
4. `aoxchub` remains operator UX and is not protocol authority.
5. AI/advisory surfaces (`aoxcai`) remain outside deterministic consensus authority.

## 4) Required evidence classes per release candidate

- exact command list executed,
- pass/fail/limited outcomes,
- changed-files inventory,
- boundary-impact statement (which tier/domain changed),
- known gaps and risk acceptance notes.

## 5) Baseline command set (workspace verification)

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings
cargo test --workspace --exclude aoxchub --all-targets
cargo check -p aoxchub --all-targets
```

## 6) Reference set

- `docs/ARCHITECTURE.md`
- `docs/SECURITY_MODEL.md`
- `docs/EXECUTION_MODEL.md`
- `docs/STATE_MODEL.md`
- `docs/RELEASE_TIERS.md`
- `docs/SYSTEM_INVARIANTS.md`
- `docs/LICENSING.md`
- `docs/TRADEMARK_POLICY.md`
- `README.md`
