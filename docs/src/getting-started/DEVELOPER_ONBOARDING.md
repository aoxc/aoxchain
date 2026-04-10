# Developer Onboarding

This guide is for contributors implementing code and documentation changes across AOXChain crates.

## Repository orientation

- Root governance surfaces define compatibility, trust, and release expectations.
- Crate-level `README.md`, `SCOPE.md`, and `ARCHITECTURE.md` files describe local contracts.
- `docs/src/` is the mdBook source of operator/developer-facing runbooks and specs.

## Recommended workflow

1. Read root `SCOPE.md`, `ARCHITECTURE.md`, and `TESTING.md`.
2. Read crate-local scope/architecture docs for touched crates.
3. Implement minimal, reviewable changes.
4. Run the smallest relevant test set first, then full required gates.
5. Update docs when behavior, interfaces, or assumptions change.

## Documentation change policy

When changing any of the following, update documentation in the same PR:

- architecture boundaries,
- protocol behavior,
- state/storage assumptions,
- runtime controls or policy toggles,
- operator procedures and release criteria.

## Suggested validation layers

- **Layer 1:** formatting, linting, and crate-targeted unit tests.
- **Layer 2:** workspace integration tests.
- **Layer 3:** readiness/validation scripts for operational and release posture.

## Commit quality checklist

- Change is scoped and intentional.
- Compatibility impact is declared or explicitly "none".
- Tests match changed behavior.
- Documentation is synchronized with code and policy.
