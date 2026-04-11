# AOXChain Security Policy

This document defines repository-level security posture, disclosure workflow, and high-risk engineering controls.

## 1. Security Posture

AOXChain targets deterministic, fail-closed Layer-1 behavior with policy-governed cryptographic validation and migration safety.

Bounded claims:

- no claim of absolute or permanent cryptographic security,
- no readiness claim without reproducible evidence,
- no acceptance of hidden trust bypasses in consensus-relevant paths.

## 2. Supported Surface

This policy applies to repository-maintained protocol, kernel, execution, networking, RPC, configuration, and operations tooling surfaces.

Third-party dependencies are in scope for triage and containment planning, though remediation may require upstream action.

## 3. Vulnerability Reporting

Report vulnerabilities privately to:

- **Security contact:** `admin@aoxcore.com`

Do not disclose unpatched vulnerabilities via public issue, pull request, or social channels.

For high-sensitivity reports, request encrypted communication in the initial contact.

## 4. Minimum Report Contents

Provide, when available:

1. affected component and trust boundary,
2. deterministic reproduction steps or PoC,
3. expected versus observed behavior,
4. impact assessment (safety, liveness, integrity, economic, governance),
5. commit hash, environment, and configuration context.

Reports with partial data are accepted, but triage may take longer.

## 5. Triage and Response Process

AOXChain follows coordinated vulnerability disclosure:

1. acknowledge receipt,
2. validate and classify severity,
3. define containment and remediation path,
4. publish advisory after mitigation and operator guidance are available.

Response timing is impact-driven; consensus and key-management vulnerabilities are prioritized.

## 6. Priority Vulnerability Classes

Highest-priority classes include:

- consensus safety/liveness/finality violations,
- deterministic execution divergence,
- cryptographic validation bypass/downgrade paths,
- replay/migration/recovery authorization flaws,
- validator/governance authority escalation,
- P2P or RPC abuse causing protocol-level denial or partition risk.

## 7. Readiness Security Gates

Security-sensitive changes must include synchronized updates where applicable:

- `ARCHITECTURE.md` (trust-boundary impact),
- `ROADMAP.md` (phase/checklist impact),
- `TESTING.md` (validation/evidence impact).

Readiness claims are non-authoritative without retained evidence tied to the tested commit.

## 8. Legal and Liability Context

AOXChain is distributed under MIT on an **"AS IS"** basis, without warranties or liability assumptions except where prohibited by law.
