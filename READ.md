# AOXC Audit Companion — v0.1.1-akdeniz (System Compatibility Update)

This file is the audit-oriented companion to the root `README.md`.

## 1) Release identity

- **Release label:** `AOXC v0.1.1-akdeniz`
- **Cargo baseline:** `0.1.1-akdeniz`
- **Documentation baseline:** `aoxc.v.0.1.1-akdeniz`

## 2) What changed in this document update

This update aligns the root guidance with a single "full target" interpretation:

- unified scope across core/consensus/net/rpc/cmd/vm/desktop,
- explicit AOXCVM vs AOXHub responsibility separation,
- deterministic operator lifecycle emphasis,
- transparent statement of remaining readiness gaps.

## 3) Audit intent

The repository should be reviewed as a deterministic multi-crate chain workspace with explicit trust boundaries for:

- cryptographic identity and custody,
- consensus correctness/finality,
- runtime persistence and DB lifecycle,
- network transport and observability,
- execution-lane safety,
- desktop/operator control-plane hygiene,
- release evidence traceability.

## 4) "%100 hedef" audit interpretation

For this baseline, "%100" means full documentation consistency and release discipline, **not** a claim that all future production risks are already eliminated.

A compliant release should show:

1. consistent version identity,
2. reproducible commands and outcomes,
3. documented limitations,
4. crate-level responsibility clarity,
5. operator-usable runbooks.

## 5) AOXCVM and Desktop governance split

### AOXCVM (`crates/aoxcvm`)

- consensus-adjacent execution compatibility layer,
- deterministic execution constraints required,
- stronger validation + replay guarantees expected.

### AOXHub Desktop (`crates/aoxchub`)

- operator UX and orchestration layer,
- should remain auditable and explicit in command mapping,
- should not silently bypass CLI/runtime controls.

## 6) Required release evidence

- executed command list,
- test/check outcomes (pass/fail/limited),
- updated docs list,
- known limitation registry,
- commit SHA linked to release notes.

Minimum baseline evidence should include non-crate surfaces too:

- root governance docs (`README.md`, `READ.md`),
- environment/config docs under `configs/`,
- operator and validation scripts under `scripts/`,
- docs index consistency under `docs/src/`.

## 7) Mainnet-ready baseline command set

Use this compact command set before candidate sign-off:

```bash
cargo fmt --all --check
cargo check --workspace --exclude aoxchub
cargo test -p aoxcsdk --lib
cargo test -p aoxcmd --lib
cargo test --workspace --exclude aoxchub --all-targets
```

If CI/runtime shows sporadic env-sensitive behavior, add serialized execution:

```bash
cargo test --workspace --exclude aoxchub --all-targets -- --test-threads=1
```

## 8) Primary references

- `README.md`
- `crates/aoxcvm/README.md`
- `crates/aoxchub/README.md`
- `docs/src/AKDENIZ_RELEASE_BASELINE.md`
- `docs/src/MAINNET_READINESS_CHECKLIST.md`
- `docs/src/REAL_NETWORK_VALIDATION_RUNBOOK_TR.md`
- `docs/src/AOXC_REAL_VERSIONING_AND_RELEASE_ROADMAP_TR.md`
