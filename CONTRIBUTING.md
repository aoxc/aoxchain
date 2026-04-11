# Contributing to AOXChain

Thank you for contributing.

AOXChain development prioritizes determinism, auditability, and fail-closed safety in protocol-relevant paths. Contributors are expected to preserve consensus integrity, architectural clarity, and reviewability.

## 1. Core Principles

- keep changes minimal, explicit, and testable,
- prefer deterministic behavior over convenience abstractions,
- preserve fail-closed behavior on safety-critical paths,
- document non-trivial architectural intent,
- keep operational/security assumptions visible.

## 2. Scope Discipline

- keep pull requests tightly scoped,
- avoid mixing refactor, behavior change, and cleanup unless justified,
- avoid protocol-sensitive edits bundled with cosmetic changes,
- ensure every change has clear rationale and validation path.

## 3. Determinism and Auditability

- do not introduce nondeterminism in consensus-adjacent logic,
- avoid implicit ordering or environment-dependent behavior,
- keep security-relevant logic explicit and traceable,
- document invariants and failure semantics when they change.

## 4. Compatibility Requirements

- do not introduce implicit breaking changes,
- document intentional breaking changes in PR notes,
- include migration implications for storage/serialization/protocol updates,
- provide validation evidence for compatibility-sensitive changes.

## 5. Prohibited Merge Patterns

Do not merge:

- dead code,
- placeholders/TODO-only behavior,
- silent fallbacks in critical paths,
- hidden behavior changes disguised as refactor,
- weakened validation for convenience,
- untested fixes in protocol, persistence, or cryptographic surfaces.

## 6. Documentation Requirements

Update relevant documents for non-trivial changes, including:

- `README.md`
- `READ.md`
- `ARCHITECTURE.md`
- `SECURITY.md`
- `TESTING.md`
- runtime/operator documentation and migration notes where applicable.

## 7. Testing Requirements

Before opening a PR:

- run formatting and linting,
- run tests relevant to changed surfaces,
- add regression tests for bug fixes,
- add negative-path tests for validation logic,
- add determinism/compatibility tests where required.

If full validation is not possible locally, explicitly state what was validated, what was not, and remaining risk.

## 8. Pull Request Requirements

PR description should include:

- summary and rationale,
- scope boundaries,
- risk notes,
- affected components,
- validation commands/outcomes,
- documentation updates,
- migration/compatibility notes when relevant.

## 9. Merge Readiness

A change is merge-ready only when:

- scope is clear,
- logic is explicit,
- determinism requirements are satisfied,
- validation is adequate to risk,
- documentation is synchronized,
- no dead code or hidden fallback remains,
- no implicit breaking change is introduced.
