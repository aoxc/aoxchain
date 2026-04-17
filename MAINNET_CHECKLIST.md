# AOXChain Mainnet Readiness Checklist (Quantum-Ready)

This checklist defines release gates for launching AOXChain mainnet with a quantum-ready security posture, auditable operations, and explicit go/no-go control.

## Usage Model

- Track every item using one status value: `NOT_STARTED`, `IN_PROGRESS`, `BLOCKED`, `DONE`, or `WAIVED`.
- Record an owner and due date for every non-trivial item.
- Treat all `BLOCKER` and `CRITICAL` items as launch-gating.
- Do not mark this checklist complete unless all mandatory go-live criteria are satisfied.

## Program Governance and Launch Control

- [ ] Mainnet target date is defined and approved.
- [ ] Release manager, protocol owner, security owner, and operations owner are explicitly assigned.
- [ ] Weekly readiness review cadence is active.
- [ ] Risk register exists with severity levels `CRITICAL`, `HIGH`, `MEDIUM`, and `LOW`.
- [ ] Escalation path is documented for launch blockers.
- [ ] Go/no-go authority and sign-off quorum are documented.

**Exit criteria**
- [ ] Open `CRITICAL` risks: `0`.
- [ ] Security and protocol owners both provide written launch sign-off.

## Protocol and Cryptography (Quantum-Ready Core)

### Cryptographic Architecture

- [ ] A single signature abstraction interface is used across transaction, consensus, and wallet surfaces.
- [ ] Algorithm identifiers (`alg_id`) are protocol-level fields, not implicit metadata.
- [ ] Crypto-agility controls are documented for future algorithm migration.
- [ ] Domain separation is consistently enforced across hashes, signatures, commitments, and transcript-bound messages.

### Post-Quantum Signature Integration

- [ ] Primary PQ signature scheme (for example ML-DSA / Dilithium profile) is selected and version-pinned.
- [ ] Key generation, signing, verification, and serialization test vectors are versioned.
- [ ] Negative verification tests cover malformed signatures, mismatched keys, altered payloads, and wrong algorithm IDs.
- [ ] Runtime constraints for large signature payloads are benchmarked and accepted.

### Addressing and Identity

- [ ] Address derivation includes algorithm awareness and checksum guarantees.
- [ ] Address and key-encoding formats are canonical and documented.
- [ ] Collision resistance and malformed address rejection are tested.

**Exit criteria**
- [ ] Cryptographic specification and test vectors are published and reproducible.
- [ ] No consensus-critical path depends on undocumented crypto assumptions.

## Transaction, Block, and Encoding Surfaces

### Transaction Envelope

- [ ] Transaction format includes explicit `signature_algorithm_id`.
- [ ] Variable-length key/signature handling is bounded and validated.
- [ ] Maximum transaction size and weight model are updated for PQ payloads.
- [ ] Fee policy accounts for both payload bytes and verification cost.

### Block and Consensus Messages

- [ ] Block headers include explicit validator signature algorithm data.
- [ ] Vote, commit, and finality proofs use the same canonical auth envelope.
- [ ] Block size and throughput limits are calibrated against realistic validator hardware.

### Serialization and Parser Hardening

- [ ] Canonical encoding rules are enforced.
- [ ] Decoders reject malformed, oversized, and ambiguous payloads deterministically.
- [ ] Fuzz coverage exists for all external decode paths.

**Exit criteria**
- [ ] Differential codec tests pass across all supported clients.
- [ ] No parser accepts ambiguous encodings.

## Consensus Safety, Liveness, and Finality

- [ ] Byzantine fault assumptions are documented and tested.
- [ ] Safety and liveness tests cover partition, delayed delivery, equivocation, and adversarial leader behavior.
- [ ] Validator set rotation, slashing paths, and rejoin semantics are validated.
- [ ] Finality latency SLO is measured under nominal and stress conditions.
- [ ] Reorg and rollback behavior is bounded and documented.

**Exit criteria**
- [ ] Safety is preserved under defined adversarial thresholds.
- [ ] Finality SLO is met under sustained load.

## P2P, Mempool, and Denial-of-Service Resistance

### Mempool Admission and Policy

- [ ] Two-phase admission is implemented: inexpensive validation before expensive cryptographic checks.
- [ ] Peer-scoped rate limits and fairness controls are active.
- [ ] Invalid transaction penalties are enforced.
- [ ] Priority and fee rules prevent trivial starvation and spam dominance.

### Network Hardening

- [ ] Anti-eclipse and peer diversity controls are active.
- [ ] Flood resilience is validated for connection, gossip, and malformed payload traffic.
- [ ] Sync behavior under churn is tested and within operational limits.

**Exit criteria**
- [ ] Targeted DoS scenarios do not induce consensus failure.
- [ ] Sustained adversarial load remains within error budget.

## State, Storage, and Recovery

- [ ] State growth projections exist for 6-, 12-, and 24-month horizons.
- [ ] Pruning and archival policies are documented and tested.
- [ ] Snapshot, restore, and state sync procedures are validated.
- [ ] Corruption detection and recovery runbooks are approved.
- [ ] Node hardware minimums and recommended profiles are published.

**Exit criteria**
- [ ] Disaster recovery drill completes within operational RTO/RPO targets.

## Wallet, CLI, SDK, and External API Readiness

- [ ] Wallet defaults to approved PQ signature profile.
- [ ] Export/import/backup formats include algorithm metadata and integrity checks.
- [ ] CLI workflows (`create`, `sign`, `verify`, `submit`) are updated and tested.
- [ ] SDKs expose stable, versioned APIs for new auth envelopes.
- [ ] RPC and API schemas are versioned with compatibility policy.

**Exit criteria**
- [ ] End-to-end transaction flow succeeds using reference wallet and SDK.

## Smart Contract and Runtime Security (If Enabled)

- [ ] Runtime determinism is proven for all consensus-relevant execution paths.
- [ ] Gas and metering rules are calibrated to prevent computational abuse.
- [ ] Privileged host functions and precompiles have explicit authorization constraints.
- [ ] Critical contract/system modules receive independent security review.
- [ ] Runtime panic, trap, and rollback semantics are deterministic and tested.

**Exit criteria**
- [ ] No unresolved critical runtime security findings.

## Verification, Testing, and Quality Gates

### Mandatory Test Surfaces

- [ ] Unit test baseline is green in CI.
- [ ] Integration and end-to-end suites are green in CI.
- [ ] Property-based tests cover codec, auth envelope, and consensus invariants.
- [ ] Continuous fuzzing covers transaction decode, block decode, and network message handling.
- [ ] Differential tests compare canonical behavior against an independent verifier or secondary client path.

### Performance and Soak

- [ ] Throughput, latency, and finality benchmarks are captured and approved.
- [ ] 24-hour and 7-day soak tests complete without consensus-critical faults.
- [ ] Resource envelopes (CPU, memory, disk, bandwidth) are within target operator budgets.

**Exit criteria**
- [ ] All launch-gating CI and preflight checks are green.

## Security Assurance and Adversarial Evaluation

- [ ] At least two independent security assessments are completed.
- [ ] All `CRITICAL` and `HIGH` findings are remediated or formally risk-accepted.
- [ ] Penetration tests cover node APIs, operator interfaces, and deployment posture.
- [ ] Red-team exercises include key compromise, partition, validator collusion, and replay attempts.
- [ ] Public or private bug bounty program is active before launch.

**Exit criteria**
- [ ] Security owner provides final risk statement and approval.

## Supply Chain, Build Integrity, and Release Process

- [ ] Reproducible build process is documented and verified.
- [ ] Release artifacts are signed and signature verification is automated.
- [ ] SBOM generation is integrated into release flow.
- [ ] Dependency, license, and provenance checks are enforced in CI.
- [ ] Rollback and emergency patch release procedure is rehearsed.

**Exit criteria**
- [ ] Release artifacts are verifiable, signed, and attributable.

## Staged Network Readiness

### Devnet

- [ ] Feature completeness achieved.
- [ ] Core stability and deterministic behavior validated.

### Public Testnet

- [ ] Open-participant stress behavior validated.
- [ ] Wallet, SDK, and RPC ecosystem compatibility confirmed.

### Incentivized Testnet

- [ ] Economic and validator behavior under incentive stress is analyzed.
- [ ] Slashing and fault penalties are validated under adversarial conditions.

**Exit criteria**
- [ ] No unresolved launch-blocking defects from testnet phases.

## Tokenomics and Economic Security

- [ ] Emission, reward, and fee dynamics are simulation-tested.
- [ ] Stake concentration and cartel risk are measured.
- [ ] Slashing parameters are calibrated for deterrence without false-positive overreach.
- [ ] Governance treasury and privileged actions are time-locked and multi-controlled.

**Exit criteria**
- [ ] Economic attack cost exceeds accepted security threshold.

## Governance and Upgrade Safety

- [ ] Governance lifecycle is documented from proposal through activation.
- [ ] Emergency upgrade process includes delay, disclosure, and audit trail requirements.
- [ ] Hard fork criteria and compatibility policy are explicit.
- [ ] Parameter change boundaries are constrained and reviewable.

**Exit criteria**
- [ ] Governance dry-run completed on test network.

## Operations, SRE, and Incident Readiness

- [ ] SLO/SLI definitions are published (availability, propagation, finality, API health).
- [ ] Monitoring dashboards exist for consensus, mempool, networking, storage, and API layers.
- [ ] Alerting rules, paging rotation, and on-call ownership are active.
- [ ] Incident response runbook and severity matrix are documented.
- [ ] Post-incident review template and closure requirements are enforced.

**Exit criteria**
- [ ] Incident simulation drill succeeds with documented corrective actions.

## Documentation and Operator Enablement

- [ ] `README.md`, `SCOPE.md`, `ARCHITECTURE.md`, `SECURITY.md`, and `TESTING.md` align with implementation state.
- [ ] Validator and full-node runbooks are current.
- [ ] API and schema references are up to date.
- [ ] Launch communication includes explicit risk and warranty posture aligned with MIT licensing context.

**Exit criteria**
- [ ] A new operator can deploy, sync, and validate using only published documentation.

## Genesis and Launch Rehearsal

- [ ] Genesis configuration is frozen and checksum-published.
- [ ] Validator set, keys, and certificates are verified.
- [ ] Chain ID and protocol version are final and immutable for launch.
- [ ] Bootstrap peers and discovery metadata are validated.
- [ ] Full launch rehearsal completes successfully in a production-like environment.

**Exit criteria**
- [ ] Launch-day runbook is approved by protocol, security, and operations owners.

## Launch Day Execution

- [ ] Final go/no-go review is completed and recorded.
- [ ] Signed release hashes are published.
- [ ] Genesis and first blocks are independently validated.
- [ ] Explorer, RPC, and wallet critical paths are functional.
- [ ] 24-hour launch war-room staffing is active.

**Exit criteria**
- [ ] No uncontained critical incident in first 24 hours.

## Post-Launch Stabilization (T+7 / T+30 / T+90)

- [ ] T+7 health report published.
- [ ] T+30 performance and security posture report published.
- [ ] T+90 parameter recalibration proposals reviewed.
- [ ] Deferred non-critical items are re-prioritized with owner assignment.

## Final Go/No-Go Matrix

### Mandatory GO Conditions

- [ ] Open `CRITICAL` security findings: `0`.
- [ ] Consensus safety and liveness acceptance tests pass.
- [ ] PQ signature flow is validated across transaction and consensus paths.
- [ ] Reproducible, signed release artifacts are available.
- [ ] Incident and recovery drills are complete.
- [ ] Staged testnet program is complete with no unresolved blockers.

### Automatic NO-GO Triggers

- [ ] Any unresolved `CRITICAL` finding.
- [ ] Unbounded consensus instability or finality failure.
- [ ] Unmitigated remote crash or trivial DoS path.
- [ ] Failed launch rehearsal for genesis or validator activation.
- [ ] Missing or untested incident recovery process.

## Weekly Readiness Review Template

- Week:
- Completed items:
- Newly blocked items:
- Risk severity changes:
- Next-week commitments:
- Go/no-go impact summary:
