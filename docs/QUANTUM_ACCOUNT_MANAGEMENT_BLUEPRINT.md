# Quantum-Grade Account Management Blueprint

## Purpose

Define a production-oriented account-management architecture for AOXChain that closes the remaining gaps between current cryptographic readiness and an end-to-end quantum-resilient operating model.

This blueprint covers:
- account lifecycle controls,
- signer isolation (`clef`-style pattern),
- seed and recovery posture,
- CLI-side controls,
- host kernel and OS hardening,
- phased migration and gate criteria.

## Scope and Assumptions

### In Scope

- External account signing lifecycle from key creation to retirement.
- Operator and user-facing signing workflows in CLI surfaces.
- Runtime admission implications for hybrid and post-quantum signatures.
- Host controls needed to protect in-memory key material and signer processes.

### Out of Scope

- Consensus-level algorithm changes unrelated to account authentication.
- Historical key compromise remediation for already-exposed secrets.
- Consumer wallet UI implementation details outside AOXChain CLI/operator scope.

### Security Assumptions

- Network adversaries can observe and replay public traffic.
- Endpoint adversaries may obtain user-space execution on non-hardened hosts.
- Long-horizon adversaries can archive artifacts for future cryptanalysis.
- Governance can activate protocol features with explicit rollout windows.

## Threat Model for Account Management

### Primary Threats

1. **Signer surface compromise**
   - Malware or injection into node/CLI process signs unauthorized transactions.
2. **Seed exfiltration and replay**
   - Single-secret recovery models create catastrophic failure domains.
3. **Signature algorithm obsolescence**
   - Classical-only signatures create future break risk under large-scale quantum capabilities.
4. **Policy bypass and blind signing**
   - Lack of typed intent and spend constraints increases phishing and operator-error risk.
5. **Host memory extraction**
   - Debug, ptrace, core dumps, and weak syscall policies expose key material.

### Residual Risk Posture Target

- No direct private-key access in node runtime.
- No single artifact that can unilaterally recover treasury-grade accounts.
- Deterministic policy evidence for every signature acceptance.
- Bounded blast radius through threshold controls and role separation.

## Target Architecture

## 1) Signer Separation (`clef`-style plus policy engine)

Implement a dedicated signer service with explicit trust-boundary separation:

- Node and execution runtime submit unsigned payloads only.
- Signer process validates request domain, chain identity, nonce, and policy.
- Human/operator confirmation required for high-risk classes.
- Signer returns signed envelope and immutable audit record.

### Required Interfaces

- `SignIntent` (typed, canonical, hash-committed request).
- `PolicyDecision` (allow/deny/defer with reason codes).
- `SignatureArtifact` (algorithm id, key id, signature bytes, evidence hash).
- `AuditEvent` (request hash, policy version, operator context, timestamp).

## 2) Crypto-Agile Account Profile

All account authentication artifacts MUST be algorithm-versioned.

### Mandatory fields

- `auth_scheme` (e.g., `ed25519-v1`, `hybrid-ed25519-ml_dsa-v1`, `ml_dsa-v1`).
- `keyset_id` and rotation epoch.
- `policy_profile_id` for signer-side enforcement.

### Migration posture

- Phase A: classical + PQ hybrid acceptance.
- Phase B: default hybrid issuance for new accounts.
- Phase C: policy-gated deprecation of classical-only accounts.

## 3) Seed and Recovery Model

Single mnemonic recovery is not sufficient for high-value roles.

### Required controls

- Threshold recovery (minimum `2-of-3`; treasury `3-of-5` or stronger).
- Recovery shares distributed across role-separated custody domains.
- Recovery ceremony with dual control and signed evidence.
- Mandatory rotation after any recovery exercise.

### Seed policy

- Seed export disabled by default for production profiles.
- Any export operation requires break-glass workflow and audit entry.
- No seed handling through environment variables or shell history.

## 4) CLI Security Profile

CLI is treated as an untrusted request client unless explicitly hardened.

### Controls required for production mode

- Offline signing mode for critical accounts.
- Human-readable typed-intent rendering (chain id, contract, method, value, nonce).
- Risk scoring and staged confirmations for policy-sensitive actions.
- Strict allowlists for destination domains/contracts where applicable.
- Nonce freshness and anti-replay preflight checks.
- Secret zeroization, memory locking, and crash dump suppression.

### Transaction classes

- `low-risk`: routine bounded operations with policy auto-approval.
- `medium-risk`: requires explicit user confirmation.
- `high-risk`: requires dual authorization and delayed execution window.

## 5) Kernel/OS Hardening Baseline for Signer Hosts

Signer hosts MUST run a hardened profile.

### Minimum baseline

- Dedicated signer user and service account isolation.
- seccomp profile minimizing syscall surface.
- AppArmor/SELinux policy confinement.
- `ptrace` restrictions and core dump disablement.
- Encrypted storage for key material at rest.
- Secure/Measured boot and signed binary provenance validation.
- Time synchronization integrity for replay-window enforcement.

### Recommended advanced controls

- Hardware-backed keys (TPM/HSM/secure enclave where available).
- Runtime integrity monitoring with tamper-evident logs.
- Network egress allowlist from signer hosts.
- Continuous anomaly detection over signing velocity and policy denials.

## Control Matrix (CLI + Kernel + Protocol)

| Control | CLI | Signer Service | Kernel/OS | Protocol/Governance |
|---|---|---|---|---|
| Typed intent | Required render and hash preview | Required canonical validation | N/A | Envelope schema versioning |
| Policy gates | Local pre-check hints | Authoritative decision point | N/A | Policy profile activation |
| Key exposure | Never plaintext export in normal flow | Key operations only in signer boundary | Memory/process isolation | Scheme and keyset metadata |
| Replay resistance | Nonce preflight | Nonce/time-window enforcement | Clock integrity | Replay rejection rules |
| PQ migration | Scheme-aware UX | Hybrid/PQ signing capability | Crypto provider hardening | Feature-gated acceptance |
| Auditability | Local operation logs | Immutable decision/event logs | Host attest logs | Governance evidence artifacts |

## Implementation Roadmap

## Stage 0 — Foundation (2-4 weeks)

- Introduce signer boundary contract and typed intent schema.
- Add policy decision codes and deterministic audit events.
- Add CLI display contract for risk-critical transaction fields.

**Exit criteria**
- Every sign request carries canonical intent hash.
- Every decision emits machine-readable reason code.

## Stage 1 — Operational Hardening (4-8 weeks)

- Production signer daemon profile and isolated execution context.
- Kernel baseline rollout scripts and compliance checks.
- Recovery workflow with threshold shares and operator runbook.

**Exit criteria**
- Seed export disabled in production profile.
- All critical accounts enrolled in threshold recovery.

## Stage 2 — Hybrid Quantum Migration (6-12 weeks)

- Default hybrid signature enrollment for new high-assurance accounts.
- Governance-gated acceptance policy for hybrid-auth envelopes.
- Cross-surface compatibility testing (CLI, RPC, mempool, validation).

**Exit criteria**
- Hybrid path green across integration matrix.
- Classical-only creation blocked for defined account tiers.

## Stage 3 — Classical De-Risk and Closure (ongoing)

- Risk-window reduction for classical-only accounts.
- Rotation campaigns to PQ-capable keysets.
- Periodic red-team drills on signer compromise and recovery ceremonies.

**Exit criteria**
- Governance-approved timeline for classical-only deprecation.
- Reproducible evidence bundle for audit closure.

## Acceptance Gates

A release is not quantum-grade for account management unless all gates pass:

1. **Signer Isolation Gate**: no direct in-process node signing for protected profiles.
2. **Policy Gate**: deterministic allow/deny with reason and evidence hash.
3. **Recovery Gate**: threshold recovery exercised and documented.
4. **Kernel Gate**: signer host hardening profile validated.
5. **Hybrid Gate**: hybrid signature path tested in CI and staging.
6. **Audit Gate**: immutable logs map every accepted signature to policy context.

## Evidence Artifacts

Each release should publish the following artifacts:

- signer-policy profile manifest,
- account auth-scheme distribution report,
- threshold recovery drill report,
- signer host hardening compliance output,
- hybrid signature interoperability matrix,
- exception register with approved risk ownership.

## Practical Position on `clef` Pattern and Seed Security

- A `clef`-style architecture is required as the minimum boundary control, but not sufficient by itself.
- Single-seed custody is acceptable only for low-value and short-lived profiles.
- Treasury, governance, and validator-affecting accounts require threshold recovery, policy-constrained signer flow, and hardened host controls.

Without these controls, cryptographic migration alone does not deliver a quantum-grade account-management posture.
