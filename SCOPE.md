# AOXChain Scope Statement

## In scope

- Deterministic L1 engineering, including consensus, execution, networking, and state primitives.
- Operator tooling and automation required to run, validate, and audit environments.
- Environment profiles and deterministic fixtures used for reproducible deployment/testing.
- Release-evidence and production-closure artifacts that substantiate operational readiness claims.
- Documentation required for engineering governance, security coordination, and institutional audits.

## Out of scope

- Any guarantee of production fitness, legal compliance, or financial suitability.
- Custodial services, regulated financial workflows, or contractual SLA commitments.
- Backward compatibility guarantees across all experimental interfaces while development is active.

## Sensitive change classes

The following changes require heightened review and explicit documentation updates:
- Consensus safety/finality behavior.
- Deterministic state transition or execution semantics.
- Key lifecycle or signer trust boundaries.
- Persisted data formats and migration logic.
- Public RPC/API contract changes.

## Compatibility policy

Compatibility is managed by release-line governance and evidence-based readiness checks. Breaking changes are permitted when justified by safety or determinism objectives, but must be explicitly declared in release documentation.

## License and liability context

AOXChain is provided under MIT. Materials are provided "as is" without warranty; maintainers and contributors do not assume liability for operational, financial, or legal outcomes.
