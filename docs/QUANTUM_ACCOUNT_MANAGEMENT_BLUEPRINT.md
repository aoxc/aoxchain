# Quantum-Grade Account Management Blueprint (Advanced)

## Document Status

- **Owner:** AOXChain security and runtime engineering.
- **Audience:** protocol maintainers, CLI maintainers, operator security teams, auditors.
- **Normative level:** implementation policy baseline for account-management hardening.
- **Change class:** architecture-sensitive and compatibility-sensitive documentation.

## Purpose

Define an implementation-ready blueprint that upgrades AOXChain account management from classical custody assumptions to a quantum-resilient, policy-governed, and auditable control system.

This blueprint unifies:
- signer boundary design,
- key and recovery lifecycle,
- CLI operational controls,
- host kernel/OS hardening,
- runtime/protocol acceptance rules,
- migration gates and evidence.

## Scope and Boundaries

### In Scope

- Externally controlled account keys and signing workflows.
- Node-to-signer trust boundaries.
- CLI request validation and user confirmation surfaces.
- Kernel/OS controls for signer host protection.
- Governance-gated migration from classical to hybrid/PQ signing.

### Out of Scope

- Consensus algorithm redesign.
- Cross-chain bridge custody frameworks.
- End-user mobile wallet UX details outside AOXChain-operated surfaces.

### System Boundary Definition

This document models the account-management system as five planes:
1. **Intent Plane**: CLI/API request materialization.
2. **Policy Plane**: risk and authorization decisions.
3. **Signing Plane**: key operations and signature generation.
4. **Runtime Admission Plane**: mempool and verifier acceptance.
5. **Evidence Plane**: logs, attestations, and release artifacts.

A production closure is valid only when all five planes satisfy the acceptance gates defined later in this document.

## Threat Model

### Adversary Classes

1. **Remote adversary**: network access, replay attempts, malformed payload injection.
2. **Endpoint adversary**: user-space code execution on CLI/node host.
3. **Privileged insider misuse**: policy bypass attempt or dual-control evasion.
4. **Long-horizon cryptanalytic adversary**: archive-now, break-later strategy.
5. **Supply-chain adversary**: tampered binaries/configs or unsigned runtime artifacts.

### Threat Events and Required Countermeasures

| Threat Event | Blast Radius Without Controls | Required Countermeasure |
|---|---|---|
| Node process signs directly with hot key | Full account compromise | Strict signer process separation + no in-process private key |
| Seed phrase theft | Total custody loss | Threshold recovery + role-separated custody + recovery audits |
| Blind signing | Irreversible asset loss | Typed intent, risk labels, allowlist policy, high-risk quorum |
| Replay across domains/chains | Unauthorized execution | Domain separation, nonce windows, chain identity binding |
| Memory extraction (ptrace/core dump) | Key material leakage | Kernel restrictions, seccomp, dump disablement, enclave/HSM |
| Classical-only auth stagnation | Future signature forgery risk | Hybrid-first migration and governance deprecation schedule |

## Non-Negotiable Security Properties

1. **No direct key access in node runtime** for protected account tiers.
2. **No single secret recovery** for treasury/governance/validator-critical accounts.
3. **Deterministic policy decision** for every accepted signature request.
4. **Cryptographic agility** with explicit `auth_scheme` versioning.
5. **Tamper-evident auditability** linking intent → decision → signature artifact.

## Reference Architecture

## 1) Signer Boundary (`clef`-style + policy authority)

A `clef`-style pattern is mandatory but upgraded into a three-layer signer stack:

- **Ingress Gateway**: validates schema, chain identity, nonce freshness, domain separators.
- **Policy Engine**: evaluates transaction risk class and authorization requirements.
- **Key Engine**: executes signing only after policy permit token is issued.

### Trust Boundary Rules

- Node, RPC, and CLI hold **no long-lived private key material**.
- Key Engine accepts only canonicalized `SignIntent` payloads.
- Signatures are bound to `(chain_id, domain_tag, nonce, intent_hash, auth_scheme)`.
- Every permit token is single-use and expires quickly.

### Canonical Sign API Contract

`SignIntent` MUST include:
- `intent_hash` (canonical hash of typed request),
- `chain_id`,
- `domain_tag`,
- `nonce`,
- `auth_scheme`,
- `policy_profile_id`,
- `tx_class` (`low`, `medium`, `high`, `break_glass`),
- `operator_context` (role, session id, approval references).

`SignDecision` MUST include:
- `decision` (`allow`, `deny`, `defer`),
- `reason_code`,
- `policy_version`,
- `expires_at`,
- `evidence_hash`.

`SignatureArtifact` MUST include:
- `signature_bytes`,
- `algorithm_id`,
- `keyset_id`,
- `signature_context_hash`.

## 2) Key Hierarchy and Lifecycle

### Key Classes

- **Account keys**: external transaction authentication.
- **Session keys**: short-lived delegated authority.
- **Recovery keys**: custody-only; never used for routine signing.
- **Emergency keys**: break-glass path with strict governance controls.

### Lifecycle States

`provisioned -> active -> rotation_pending -> retired -> revoked`

Each state transition MUST emit audit events with actor identity and policy basis.

### Rotation Policy

- Time-based rotation for all critical keysets.
- Immediate rotation on suspected compromise or failed attestation.
- Recovery exercise forces mandatory post-exercise rotation.

## 3) Seed and Recovery Posture

### Required Recovery Model

- Baseline: `2-of-3` threshold for operational criticality.
- Treasury/validator/governance: `3-of-5` or stricter threshold.
- Recovery shares split across independent custodial domains.

### Recovery Ceremony Controls

- Dual (or greater) operator presence.
- Signed checklist execution with timestamped evidence.
- Hardware identity attestation for participating signer hosts.
- Mandatory post-ceremony compromise assessment.

### Seed Restrictions

- Seed export disabled by default in production profiles.
- Any export path is break-glass only, with governance traceability.
- Prohibit secret injection via shell env vars and command arguments.

## 4) Quantum Migration Strategy

## Stage Q0 — Crypto Agility Preparation

- Add explicit `auth_scheme` everywhere signatures are created or verified.
- Add compatibility matrix for classical, hybrid, and PQ-native paths.

## Stage Q1 — Hybrid Default for Critical Accounts

- New high-assurance accounts must use hybrid signatures.
- Runtime accepts both classical and hybrid based on policy tier.

## Stage Q2 — Progressive Classical Constraining

- Disallow creation of classical-only keys for protected account tiers.
- Increase policy friction for classical-only execution paths.

## Stage Q3 — Classical Deprecation Window

- Governance defines final classical-only sunset schedule.
- Forced migration campaigns with clear exception registry.

### Migration Safety Constraints

- No abrupt cutover without dual-path validation period.
- No revocation event without deterministic rollback plan.
- Maintain replay protections consistently across all schemes.

## 5) CLI Security Model

CLI is a request-construction and operator-confirmation surface; not a custody boundary.

### CLI Production Requirements

- Render full typed intent before any signature request.
- Show chain id, destination, contract/method, value, fee ceiling, nonce, auth scheme.
- Require explicit confirmation for `medium` and quorum confirmation for `high`.
- Local policy hints must never override signer policy authority.
- Enforce anti-replay preflight with bounded nonce/time windows.
- Zeroize sensitive memory and disable crash artifact leakage where possible.

### CLI Risk Workflow

| Transaction Class | Example | Required Authorization |
|---|---|---|
| `low` | bounded transfer to allowlisted target | signer policy auto-allow |
| `medium` | contract call with value transfer | explicit operator confirmation |
| `high` | governance, validator, treasury operation | dual control / multi-approval |
| `break_glass` | emergency override | governance-linked emergency quorum |

### CLI Hardening Checklist

- Binary provenance verification before execution.
- Strict configuration parsing and schema validation.
- No plaintext secrets in CLI logs.
- Strong defaults: deny-on-ambiguity and explicit override flags.
- Deterministic output mode for automation pipelines.

## 6) Kernel and OS Hardening Baseline

Signer hosts are security-critical infrastructure.

### Mandatory Host Controls

- Dedicated signer host role; no mixed workload placement.
- Mandatory access controls (AppArmor/SELinux).
- seccomp syscall minimization profile for signer service.
- `ptrace` restrictions and core dump disablement.
- Signed binaries and verified boot chain.
- Encrypted storage and strict file permission model.
- Time sync integrity controls for replay-window enforcement.

### Recommended Advanced Controls

- HSM/TPM-backed keys or enclave-backed key operations.
- Immutable infrastructure pattern for signer hosts.
- Egress allowlisting and outbound policy enforcement.
- Continuous runtime integrity checks and anomaly alerts.

### Kernel Compliance Signals

Release evidence should include:
- active LSM mode,
- seccomp profile hash,
- ptrace/core dump policy snapshot,
- signer process UID/GID isolation proof,
- boot integrity attestation state.

## 7) Runtime Admission and Protocol Coupling

Runtime verification MUST enforce account-management controls instead of trusting client behavior.

### Admission Requirements

- Verify `auth_scheme` support and governance activation state.
- Validate signature domain binding and nonce freshness.
- Reject ambiguous or malformed typed-intent envelopes.
- Enforce replay cache semantics across mempool and finalization paths.

### Governance Coupling

- Feature gates define activation windows for hybrid/PQ modes.
- Policy profile changes require versioned governance artifacts.
- Emergency rollback toggles must be auditable and time-bounded.

## 8) Observability and Audit Evidence

### Mandatory Event Set

Every sign flow must emit linked events:
1. `intent_received`
2. `policy_evaluated`
3. `operator_confirmed` (when applicable)
4. `signature_issued`
5. `admission_verified`

### Evidence Requirements

- Immutable event log with hash chaining.
- Mapping from `intent_hash` to policy decision and signature artifact.
- Retention and export format suitable for external audit.
- Exception register for any temporary policy bypass.

## 9) Operational Runbooks

## A) Compromise Runbook (Signer Host)

1. isolate host from network,
2. revoke affected keyset,
3. activate incident policy profile,
4. rotate keys under threshold ceremony,
5. publish signed incident closure evidence.

## B) Recovery Runbook (Threshold)

1. initiate quorum-authenticated recovery request,
2. execute share assembly under dual control,
3. re-provision new active keyset,
4. revoke old keyset,
5. run post-recovery validation and publish audit package.

## C) Break-Glass Runbook

- limited-time emergency policy override,
- mandatory multi-party authorization,
- automatic expiration,
- mandatory retrospective governance review.

## 10) Acceptance Gates (Release Blocking)

A release is not quantum-grade for account management unless all gates pass:

1. **Signer Isolation Gate**: protected profiles cannot sign in-process.
2. **Policy Determinism Gate**: every decision has stable reason codes.
3. **Recovery Gate**: threshold recovery drill succeeded in current release window.
4. **Kernel Gate**: signer host baseline compliance is proven.
5. **Hybrid Gate**: hybrid path passes integration and staging checks.
6. **Audit Gate**: complete event linkage exists for sampled transactions.

## 11) Implementation Backlog (Execution-Oriented)

### Workstream A — Signer Service

- implement canonical `SignIntent` schema validation,
- implement permit-token model and decision expiry,
- enforce single-use decision token semantics,
- add structured reason-code registry.

### Workstream B — CLI Surface

- render typed intent with deterministic field ordering,
- implement risk-class confirmation gates,
- add strict anti-replay preflight checks,
- add non-interactive policy-safe mode for automation.

### Workstream C — Runtime Admission

- bind signature verification to `auth_scheme` governance state,
- enforce envelope domain and nonce window checks,
- wire rejection telemetry and operator-facing diagnostics.

### Workstream D — Kernel Hardening

- provide signer-host baseline profile and validation script,
- add build/release evidence export for host compliance,
- add policy to block release closure when host baseline fails.

### Workstream E — Recovery Governance

- formalize threshold ceremony checklist,
- codify recovery evidence artifact schema,
- enforce mandatory post-recovery rotation policies.

## 12) Metrics and SLO Targets

Track at minimum:
- policy denial rate by reason code,
- high-risk request confirmation latency,
- replay rejection count,
- signature issuance success/error ratio,
- percentage of protected accounts on hybrid/PQ-capable schemes,
- recovery drill completion rate per quarter.

Suggested SLOs:
- 100% of high-risk operations require multi-party authorization,
- 100% signer decisions include deterministic reason codes,
- 0 protected-profile signatures from in-process node keys,
- 100% release windows include current recovery drill evidence.

## 13) Practical Position

- A `clef`-style split is required as the minimum trust-boundary control.
- Seed-only custody is insufficient for high-value AOXChain roles.
- Quantum readiness requires simultaneous closure across cryptography, operations, and host security.

Without signer isolation, threshold recovery, runtime enforcement, and kernel hardening, cryptographic migration alone does not establish quantum-grade account management.


## 14) Repository Integration Map (AOXChain-Specific)

This section maps blueprint controls to repository surfaces so implementation work can be tracked with clear ownership.

| Control Surface | Primary Repository Area | Integration Goal |
|---|---|---|
| CLI intent rendering and risk prompts | `crates/aoxcmd/src/cli/` | canonical typed-intent UX and risk-class confirmations |
| Key material and manager flow | `crates/aoxcmd/src/keys/` | enforce no-seed-in-env policy, rotation hooks, safe loaders |
| Auth scheme and envelope rules | `crates/aoxcvm/src/auth/` | scheme versioning, hybrid acceptance, replay protections |
| PQ migration and crypto inventory | `crates/aoxcvm/src/crypto/pq/` | hybrid/PQ capability gates and compatibility reports |
| Runtime and admission semantics | `crates/aoxcvm/src/vm/` + `crates/aoxcvm/src/verifier/` | enforce policy-coupled signature admission |
| Kernel transaction and mempool flow | `crates/kernel/aoxcore/src/transaction/` | preserve nonce/domain integrity through submission path |
| Governance and feature gating | `crates/aoxcvm/src/governance/` | staged activation and deprecation windows |
| Release readiness and validation gates | `scripts/validation/` | block release when signer/policy/hardening evidence is absent |

## 15) Configuration Profiles (Normative Baselines)

Define three policy profiles for operational clarity:

### `profile_dev_local`
- classical or hybrid allowed,
- single-operator confirmation,
- relaxed host hardening,
- mandatory audit logging still enabled.

### `profile_staging_assurance`
- hybrid required for critical paths,
- medium/high risk transaction gating enabled,
- signer host baseline hardening enforced,
- replay and domain checks in strict mode.

### `profile_prod_high_assurance`
- protected accounts require hybrid/PQ-capable auth scheme,
- threshold recovery mandatory,
- high-risk actions require multi-party approval,
- break-glass path time-bounded and governance-linked,
- release gate blocks on missing evidence artifacts.

## 16) Control Verification Commands (Operator Runbook Interface)

Use a standard command contract (actual command names may differ by release branch):

1. `aoxc readiness full-surface --profile staging --format json`
2. `aoxc readiness full-surface --profile production --require-hybrid`
3. `aoxc policy audit --window 24h --emit evidence`
4. `aoxc tx verify-intent --file <intent.json> --strict`
5. `aoxc tx verify-signature --file <envelope.json> --auth-scheme <scheme>`
6. `aoxc security signer-host-check --profile hardened`

All production releases SHOULD archive command outputs in release evidence bundles.

## 17) Exception Governance Model

Exceptions are permitted only as explicit, time-bounded risk acceptances.

### Exception Record Fields

- `exception_id`
- `scope` (account tier / environment / operation class)
- `owner`
- `reason`
- `compensating_controls`
- `expiry`
- `review_signatures`

### Exception Rules

- No perpetual exception entries.
- No exception may disable audit event linkage.
- Any exception affecting protected accounts requires governance review.
- Expired exceptions auto-fail release gate checks.

## 18) Quarterly Assurance Program

Minimum quarterly activities:

1. threshold recovery dry run,
2. signer compromise tabletop exercise,
3. replay-resistance regression suite,
4. auth-scheme distribution review,
5. open exception burn-down review.

Outputs must be attached to release evidence archives and referenced by release notes.

## 19) Closure Criteria for "Quantum-Grade" Claim

AOXChain may assert a quantum-grade account-management posture only when:

- protected-account signatures are isolated behind policy-governed signer boundaries,
- threshold recovery and rotation evidence is current,
- hybrid/PQ migration milestones are met for declared tiers,
- runtime admission enforces scheme-aware replay-safe verification,
- kernel hardening compliance evidence is attached to release artifacts,
- no unresolved high-severity exception is past expiry.

If any criterion fails, release posture must be downgraded in public readiness reporting.
