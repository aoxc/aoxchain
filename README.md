# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is an experimental, modular Layer-1 engineering workspace focused on deterministic execution, auditable operations, and production-governed release evidence.

> **Current repository status (as of March 30, 2026):** Active development. Interfaces and operational procedures can change while readiness controls mature.

## 1) Purpose and operating intent

This repository exists to build and validate:
- deterministic state transition and execution paths,
- consensus and networking safety envelopes,
- operator-grade runtime controls,
- auditable release evidence suitable for internal and external audits.

The codebase is organized for engineering assurance: every important operational claim is expected to be supportable by tests, machine-readable artifacts, and reviewable documentation.

## 2) System status model (single root view)

AOXChain tracks status through evidence-driven gates instead of subjective labels:
- **Code status:** workspace crates compile and test under pinned toolchains,
- **Operations status:** readiness checks evaluate profile, key state, artifacts, and closure controls,
- **Audit status:** release-evidence and production-closure bundles are generated and retained in `artifacts/`.

Primary status signals are produced by the CLI/readiness flow in `crates/aoxcmd` and persisted as markdown/JSON evidence artifacts.

## 3) Repository map

| Path | Role |
|---|---|
| `crates/` | Rust workspace crates (protocol, runtime, networking, RPC, SDK, operator tooling). |
| `configs/` | Environment profiles and deterministic network definitions (mainnet, testnet, devnet, localnet, sovereign templates). |
| `docs/` | mdBook-backed governance and technical documentation set. |
| `models/` | Canonical readiness/risk/profile models used for operational checks. |
| `scripts/` | Automation for runtime, quality gates, and release evidence generation. |
| `tests/` | Integration and readiness validation suite. |
| `artifacts/` | Generated evidence bundles (release-evidence and production-closure snapshots). |
| `contracts/` | System-contract reference surfaces and deployment matrix inputs. |

## 4) Documentation baseline (audit-oriented)

Top-level governance documents:
- `README.md` (this file): system intent, repository status, and navigation.
- `READ.md`: canonical technical definition and invariants.
- `SCOPE.md`: in-scope/out-of-scope and compatibility boundaries.
- `ARCHITECTURE.md`: component/data-flow boundaries.
- `SECURITY.md`: vulnerability handling and security expectations.
- `TESTING.md`: validation strategy and mandatory checks.

mdBook entry points:
- `docs/src/README.md`
- `docs/src/SYSTEM_STATUS.md`
- `docs/src/AI_TRAINING_AND_AUDIT_GUIDE.md`

## 5) AI training and audit usage

For AI training or audit pipelines, treat this repository as a **traceable corpus**:
1. Ingest crate-level READMEs and root governance docs first.
2. Bind claims to source files and generated artifacts.
3. Use readiness and matrix models from `models/` as structured labels.
4. Preserve evidence lineage from scripts -> artifacts -> reports.

This approach supports reproducible model training, compliance reviews, and institutional reporting.

## 6) License and liability context

AOXChain is distributed under the [MIT License](./LICENSE). Repository content is provided on an "as is" basis, without warranties or liability assumptions by maintainers or contributors.

Nothing in this repository constitutes legal, financial, or production-readiness certification.
