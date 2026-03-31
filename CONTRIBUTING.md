# Contributing

Thank you for contributing to AOXChain.

This repository is developed with an explicit preference for deterministic behavior, reviewability, and auditability. Contributors are expected to preserve consensus safety, operational clarity, and architectural discipline in every change.

## Core Principles

- Keep all changes minimal, explicit, and reviewable.
- Prefer deterministic behavior and auditable logic over convenience shortcuts.
- Preserve fail-closed behavior for safety-critical code paths.
- Treat consensus, state, cryptography, networking, and persistence as high-sensitivity surfaces.
- Document architectural intent for any non-trivial change.
- Ensure operational and security assumptions remain visible in code, tests, and review notes.

## Rules

### 1. Scope Discipline
- Keep pull requests tightly scoped.
- Separate refactors, behavior changes, and cleanups unless there is a clear technical reason to combine them.
- Avoid mixing protocol changes with cosmetic or unrelated edits.
- Every change must have a clear justification and an identifiable validation path.

### 2. Determinism First
- Do not introduce non-deterministic behavior into consensus-adjacent paths.
- Do not rely on implicit environment behavior, unstable ordering, or convenience abstractions that weaken reproducibility.
- Use explicit data flow, explicit validation, and explicit error handling.
- If a component influences state transition, finality, validator behavior, signature validation, or recovery, determinism is mandatory.

### 3. Auditability
- Code must be understandable under adversarial review.
- Security-relevant logic must be explicit, narrow in scope, and easy to trace.
- Important invariants, assumptions, and failure conditions must be visible in code or accompanying documentation.
- Non-obvious architectural decisions must be documented in a concise and reviewable form.

### 4. Backward Compatibility
- Do not introduce implicit breaking changes to state, protocol, storage, or API contracts.
- Any intentional breaking change must be explicitly documented in the PR description and supporting documentation.
- Migration implications must be stated clearly whenever layouts, serialization, message formats, or validator/state rules change.
- Compatibility-sensitive changes must include validation evidence.

## Do Not

The following changes are not acceptable for merge:

- Do not merge dead code.
- Do not merge placeholders.
- Do not merge silent fallbacks.
- Do not merge TODO-only behavior.
- Do not merge speculative abstractions without current operational value.
- Do not suppress errors in safety-critical paths.
- Do not introduce hidden behavior changes behind refactors.
- Do not weaken validation logic for convenience.
- Do not add ambiguous recovery behavior to consensus, storage, or networking flows.
- Do not merge untested bug fixes for protocol, persistence, cryptography, or state transition logic.
- Do not introduce implicit breaking changes to state, protocol, or API contracts.

## Required Engineering Standard

### Code Quality
- Keep implementations concise and intentional.
- Remove obsolete branches, unused helpers, and dead paths as part of the change when safe to do so.
- Name things according to protocol or architectural meaning, not temporary convenience.
- Prefer straightforward logic over framework-heavy or opaque abstractions.

### Documentation
Document the following whenever relevant:

- architectural intent,
- invariant changes,
- state or storage implications,
- protocol message or serialization implications,
- validator or consensus behavior changes,
- operational impact,
- migration or rollback considerations.

For non-trivial changes, update the relevant documentation surface, such as:

- `README.md`
- `READ.md`
- `ARCHITECTURE.md`
- `SECURITY.md`
- `TESTING.md`
- runtime or operator documentation
- release or migration notes

### Security-Critical Changes
The following areas require heightened care and explicit review notes:

- consensus rules,
- finality logic,
- fork choice,
- validator rotation,
- voting and quorum logic,
- slashing or evidence handling,
- signature verification,
- post-quantum enforcement,
- persistence and recovery,
- networking and peer validation,
- genesis/bootstrap material,
- serialization and hashing logic.

For these areas, contributors must explicitly describe:
- what changed,
- why it changed,
- what assumptions it relies on,
- how it was validated,
- what failure modes were considered.

## Testing

Before opening a PR, contributors must run formatting, linting, and the relevant unit/integration tests.

### Minimum Expectations
- Run formatting checks.
- Run linting checks.
- Run all tests relevant to the changed surface.
- Add regression tests for every bug fix.
- Add behavior tests for every material logic change.
- Add negative-path tests when validation or rejection behavior changes.
- Add determinism tests where state transition or consensus behavior is affected.
- Add compatibility or replay-oriented tests where storage, recovery, or serialization behavior changes.

### Production-Leaning Changes
For production-leaning changes, validation should be stronger and may include:
- targeted integration tests,
- replay or recovery validation,
- state compatibility validation,
- serialization compatibility checks,
- adversarial or malformed input tests,
- evidence of deterministic outcomes across repeated runs.

If a change touches a high-risk surface and cannot be fully validated locally, the PR must clearly state:
- what was validated,
- what was not validated,
- what residual risk remains.

## Pull Request Expectations

Every PR should make review easy.

### PR Description Should Include
- summary of the change,
- reason for the change,
- scope boundaries,
- risk notes,
- affected components,
- validation commands and outcomes,
- documentation updates,
- migration or compatibility notes when applicable.

### Good PR Behavior
- Keep commits understandable.
- Keep the diff reviewable.
- Call out non-obvious design choices directly.
- State clearly whether the change is behavior-preserving, behavior-changing, or protocol-affecting.
- Identify any follow-up work separately rather than hiding it in partial implementation.

## Review Standard

Reviewers expect:

- clear scope,
- explicit risk notes,
- explicit validation evidence,
- visible architectural intent,
- deterministic and auditable implementation,
- no hidden behavioral drift,
- no dead code or placeholder logic,
- no silent fallback paths in critical systems.

For production-leaning changes, reviewers may additionally expect:
- failure mode analysis,
- compatibility reasoning,
- operational impact notes,
- recovery considerations,
- evidence that the implementation remains inspectable under audit.

## Merge Readiness

A change is considered merge-ready only when all of the following are true:

- the scope is clear,
- the logic is explicit,
- the implementation is deterministic where required,
- validation is adequate for the risk level,
- regression coverage exists where appropriate,
- documentation is updated where necessary,
- no dead code, placeholder paths, or silent fallbacks remain,
- no implicit breaking change is introduced.

## Preferred Contributor Mindset

Contributors should optimize for long-term safety and maintainability, not short-term convenience.

When in doubt:
- make the smaller change,
- choose the more explicit path,
- preserve deterministic behavior,
- document the architectural reason,
- and leave behind evidence that the change can be audited with confidence.
