# Documentation Conventions

This mdBook follows an operational documentation model.

## Principles

- Concise and implementation-aware.
- Explicit about assumptions, risk, and boundaries.
- Free of placeholder or decorative prose.
- Structured for both operators and developers.

## Required structure expectations

Each major document should answer:

1. What this component/runbook is.
2. What it is responsible for.
3. What it depends on.
4. What it must not do.
5. What changes are high-risk.

## Naming and organization

- Keep major policy/spec documents under dedicated sections (`architecture`, `operations`, `testing`, `governance`).
- Prefer stable filenames to preserve links and review continuity.
- Use section ordering in `SUMMARY.md` as the canonical reading flow.

## mdBook compatibility checklist

- Every page is reachable from `SUMMARY.md`.
- Internal links are relative and case-correct.
- `book.toml` points to `src` and builds into `docs/book`.
- Build with `create-missing = false` to prevent accidental orphan references.

## Review discipline

Documentation changes that alter policy, compatibility, or trust boundaries must be explicit in PR context and easy to audit.
