# AOXChain Security Policy (Repository Root)

This document defines AOXChain's repository-level security posture, vulnerability disclosure workflow, and high-risk engineering controls.

## Security Posture

AOXChain targets deterministic, fail-closed Layer-1 behavior with policy-governed cryptographic validation and migration safety.

Security claims are bounded:

- no claim of permanent or absolute cryptographic security,
- no claim of readiness without reproducible validation evidence,
- no acceptance of hidden trust bypasses in consensus-relevant paths.

## Supported Scope

This policy applies to repository-maintained protocol, kernel, execution, networking, RPC, configuration, and operational tooling surfaces.

Third-party dependencies are in scope for triage and containment planning, but may require upstream remediation.

## How to Report a Vulnerability

Report vulnerabilities privately to:

- **Security contact:** `admin@aoxcore.com`

Do not open a public issue, pull request, or social post for an unpatched vulnerability.

For high-sensitivity reports, request encrypted communication in the first email.

## Required Report Content

Include, at minimum:

1. affected component(s) and trust boundary,
2. deterministic reproduction steps or proof-of-concept,
3. expected vs observed behavior,
4. impact assessment (safety, liveness, integrity, economic or governance risk),
5. commit/hash, environment, and configuration context.

Reports missing key data are still accepted but may extend triage time.

## Triage and Response Model

AOXChain uses coordinated vulnerability disclosure.

Process:

1. acknowledge receipt,
2. validate and classify severity,
3. define containment (code fix, config mitigation, or operator action),
4. publish advisory after mitigation is available and operators have update guidance.

Response speed is impact-driven; critical consensus or key-management issues are prioritized.

## Priority Vulnerability Classes

Highest-priority classes include:

- consensus safety/liveness/finality violations,
- deterministic execution divergence,
- cryptographic validation bypass or downgrade path,
- replay/migration/recovery authorization flaws,
- validator/governance authority escalation,
- cross-domain key reuse or key-derivation isolation failures,
- P2P/RPC abuse that can cause protocol-level denial or partition risk.

## Release and Readiness Security Gates

Security-sensitive changes must ship with synchronized updates where relevant:

- `ARCHITECTURE.md` (trust boundary impact),
- `ROADMAP.md` (phase/checklist impact),
- `TESTING.md` (required validation/evidence impact).

Readiness or promotion claims are non-authoritative without retained evidence linked to the tested commit.

## Legal and Liability Context

AOXChain is distributed under the MIT License on an "as is" basis, without warranty or liability assumptions by maintainers or contributors except where prohibited by applicable law.
