# Security Policy

## Supported Release Line

AOX Chain is currently tracking the **testnet stabilization baseline** `aoxc.v.0.1.0-testnet.1` with Cargo-compatible version `0.1.0-testnet.1`. Security support is aligned to the active pre-production release train shown below.

| Release Line | Status | Security Support |
| --- | --- | --- |
| `aoxc.v.0.1.0-testnet.1` | Active Testnet Baseline | Full support for security fixes, release gating, and coordinated testnet rollouts |
| `aoxc.v.0.1.0-dev` | Development Only | Best-effort support; not eligible for public deployment |
| `< aoxc.v.0.1.0-testnet.1` | Superseded Pre-Testnet Builds | No support unless explicitly reactivated for incident forensics |

## Reporting a Vulnerability

If you discover a vulnerability in AOX Chain core, consensus, networking, execution, identity, RPC, mobile, or operator tooling, do **not** open a public issue.

### Submission Channel
Send the report to **security@aoxchain.io**. For sensitive material, use encrypted delivery and attach reproducible artifacts only through approved private channels.

### Minimum Report Contents
Please include:
- vulnerability class,
- affected component and version,
- reproduction steps or proof of concept,
- impact on finality, validator safety, funds, or operational continuity,
- suggested mitigations if already known.

## Coordinated Response Policy

Upon receiving a valid report, the security response process is:
1. acknowledge receipt within 24 hours,
2. triage severity and reproduce within 3-5 business days,
3. prepare private fixes and validator/operator guidance,
4. coordinate testnet rollout or emergency patch procedure,
5. publish an advisory and post-incident review after containment.

## Priority Scope

The highest-priority classes are:
- consensus safety or liveness failures,
- cross-lane execution isolation failures,
- identity, certificate, or key-custody bypasses,
- RPC authentication, authorization, or replay defects,
- P2P eclipse, Sybil, or high-amplification denial-of-service vectors,
- release supply-chain compromise or artifact signing failures.

## Mandatory Companion Documents

Security review and release approval should be read together with:
- `README.md`,
- `READ.md`,
- `VERSION.md`,
- `docs/MAINNET_READINESS_CHECKLIST.md`,
- `docs/RELEASE_AND_PROVENANCE_RUNBOOK.md`,
- `docs/BACKUP_RESTORE_AND_ROLLBACK_RUNBOOK.md`,
- `docs/THREAT_MODEL_AND_ATTACK_SURFACE.md`,
- `docs/RELEASE_OWNERSHIP_AND_ESCALATION.md`.
