# AOXChain Production Readiness Checklist

## Purpose

This checklist converts repository policy and blueprint intent into auditable go/no-go controls.

A branch is **not production-ready** unless every mandatory item below is complete with retained evidence.

## 0. Baseline Governance and Freeze Controls

- [ ] A release candidate tag and immutable commit SHA are declared.
- [ ] Scope-impact summary is published for consensus, VM, crypto, storage, API, and operator surfaces.
- [ ] Change freeze window is active for production-sensitive modules until gate completion.
- [ ] Risk owner, approver, and rollback owner are explicitly assigned.

## 1. Determinism and Consensus Closure

- [ ] Deterministic replay matrix passes across supported environments.
- [ ] Equal-height fork-choice tie-break behavior is deterministic and regression-tested.
- [ ] Adversarial quorum and partition/rejoin scenarios pass without invariant drift.
- [ ] Consensus version activation and rollback rules are tested on a controlled network.

**Required evidence**
- replay matrix artifacts,
- adversarial test report,
- fork-choice determinism report,
- activation/rollback rehearsal logs.

## 2. VM Runtime and Admission Closure

- [ ] Bytecode/package admission is fail-closed for malformed, incompatible, and policy-banned artifacts.
- [ ] Gas metering remains deterministic under replay and stress paths.
- [ ] Syscall capability boundaries are enforced with explicit denial evidence.
- [ ] Transaction replay protections, nonce windows, and receipt consistency are validated.

**Required evidence**
- determinism matrix,
- malicious input rejection corpus,
- syscall policy test outputs,
- runtime constitution regression logs.

## 3. Cryptography and Key Lifecycle Closure

- [ ] Active cryptographic profile policy (classical/hybrid/PQ) is declared and enforced.
- [ ] Downgrade attempts are rejected at session and admission boundaries.
- [ ] Key issuance, rotation, and revocation flows are exercised end-to-end.
- [ ] Emergency cryptographic kill-switch and rollback procedures are rehearsed.

**Required evidence**
- profile compatibility matrix,
- downgrade rejection tests,
- rotation/revocation drill logs,
- governance activation records.

## 4. Network and External Surface Hardening

- [ ] mTLS or equivalent peer identity admission controls are validated.
- [ ] Rate-limit and abuse-path protections pass adversarial regression.
- [ ] P2P quarantine/reconnect controls are deterministic and observable.
- [ ] External ingress fuzz and envelope tamper suites pass.

**Required evidence**
- resilience/adversarial suite output,
- ingress fuzz report,
- peer quarantine and reconnect logs,
- API abuse test report.

## 5. Storage Safety and Recovery Closure

- [ ] Storage format/version policy is explicit and migration-tested.
- [ ] Crash-consistency and corruption-detection scenarios are validated.
- [ ] Snapshot/backup/restore flow is rehearsed with deterministic replay verification.
- [ ] Forensic retention policy is operationally exercised.

**Required evidence**
- migration validation logs,
- corruption-injection results,
- backup/restore drill output,
- replay-after-restore consistency report.

## 6. API and Operator Surface Closure

- [ ] CLI commands used in production operations are compatibility-tested and documented.
- [ ] API schemas are versioned and backward-compatibility tested.
- [ ] Error models and status surfaces are deterministic and machine-consumable.
- [ ] Operator runbooks for bootstrap, incident handling, and recovery are tested.

**Required evidence**
- API contract tests,
- CLI compatibility matrix,
- operator rehearsal logs,
- runbook validation notes.

## 7. Security and Supply Chain Closure

- [ ] Dependency vulnerability and license checks pass at release cut.
- [ ] SBOM, provenance, and signature outputs are generated and verified.
- [ ] Security incident response and disclosure paths are validated.
- [ ] Secrets, key custody, and access-control assumptions are documented and current.

**Required evidence**
- security gate output,
- SBOM + provenance artifacts,
- signature verification report,
- incident rehearsal artifact.

## 8. CI Enforcement (Must Be Automated)

- [ ] Mandatory commands in `TESTING.md` run in CI for protected branches.
- [ ] Gate jobs fail closed on skipped checks or missing evidence artifacts.
- [ ] Artifact publication is deterministic and linked to commit SHA.
- [ ] Release promotion requires explicit PASS for all gate categories.

## 9. Final Production Declaration Format

Use this exact structure in release notes or gate summary:

- `Status: PRODUCTION_READY` or `Status: NOT_READY`
- `Commit: <sha>`
- `Environment: <mainnet/testnet/...>`
- `Failed gates: <none|list>`
- `Evidence bundle: <path or URL>`
- `Rollback owner: <name/team>`

A declaration without complete evidence linkage is non-authoritative.
