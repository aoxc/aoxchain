# AOXChain Documentation Index

This mdBook is the structured documentation surface for operators and developers working on AOXChain runtime, governance, release, and readiness workflows.

## Audience Paths

### If you are an operator

1. `Getting Started -> Quickstart`
2. `Overview -> System Status and Operational Posture`
3. `Operations -> Full Node Guide`
4. `Testing -> Test Matrix`

### If you are a developer

1. `Getting Started -> Developer Onboarding`
2. `Architecture` section for protocol and implementation blueprints
3. `Testing` section for invariant and matrix coverage
4. `Governance -> AI Training and Audit Guide`

## Canonical Root Documents

For repository-level policy and system boundaries, use root files as the final source of truth:

- `README.md` — project purpose and entry points.
- `READ.md` — canonical technical contract and boundary rules.
- `WHITEPAPER.md` — protocol architecture and production closure narrative.
- `SCOPE.md` — in-scope/out-of-scope and compatibility posture.
- `ARCHITECTURE.md` — component boundaries and dependency direction.
- `SECURITY.md` — disclosure handling and security posture.
- `TESTING.md` — validation criteria and go/no-go expectations.
- `ROADMAP.md` — staged execution roadmap.

## mdBook Integrity Notes

- Navigation is governed by `src/SUMMARY.md`.
- Build configuration is defined in `book.toml`.
- Generated output lives in `docs/book/`.
