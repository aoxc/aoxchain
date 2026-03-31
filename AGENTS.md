# AGENTS Guidance

Repository-root instructions for contributors, maintainers, reviewers, and automation surfaces.

---

## Scope

These instructions apply to the **entire repository** unless a deeper `AGENTS.md` defines a more specific rule set for a subdirectory or component.

Where multiple `AGENTS.md` files exist, the **nearest applicable file takes precedence** for that subtree.

---

## Documentation Standard

For important modules, crate roots, and repository-governance surfaces, documentation should remain **concise, production-oriented, and operationally useful**.

### Expected Top-Level Documents

#### `README.md`
Should communicate:
- project purpose,
- repository contents,
- primary usage surfaces,
- important operational or compatibility notes.

#### `SCOPE.md`
Should define:
- what is in scope,
- what is explicitly out of scope,
- what categories of change are considered sensitive,
- what compatibility expectations must be preserved.

#### `ARCHITECTURE.md`
Should describe:
- major components,
- data and control flow,
- dependency direction,
- trust and validation boundaries.

#### `SECURITY.md`
Add where risk exposure, security posture, or disclosure handling requires an explicit security document.

#### `TESTING.md`
Add where validation requirements, test surfaces, or release-readiness criteria justify a dedicated testing reference.

---

## Documentation Expectations

Documentation should be written for **real engineering use**, not as placeholder governance text.

### Required Qualities
- concise,
- precise,
- reviewable,
- implementation-aware,
- production-oriented.

### Avoid
- vague summaries,
- decorative prose without engineering value,
- ambiguous TODO-style language,
- placeholder sections with no operational meaning,
- documentation that conceals uncertainty instead of stating it explicitly.

Where a change affects architecture, protocol behavior, storage layout, validation, runtime controls, or operational assumptions, the relevant document should be updated as part of the same change.

---

## Quality Rules

### Language Standard
- Use **institutional, precise, and professional English**.
- Prefer direct technical language over informal or promotional phrasing.
- Keep wording explicit where behavior, compatibility, or safety matters.

### Content Standard
- Avoid placeholders.
- Avoid ambiguous TODO-style prose.
- Avoid non-committal wording in governance or architecture-sensitive documents.
- Prefer explicit statements of scope, assumptions, constraints, and risk.

### Governance Standard
- Keep **MIT license and liability context** explicit in top-level governance documents where appropriate.
- Preserve repository-level clarity around authorship, warranty disclaimer, and project posture.
- Ensure governance documentation remains aligned with the repository’s actual engineering and operational state.

---

## Repository Writing Style

The preferred documentation style for this repository is:

- structured,
- concise,
- technically disciplined,
- audit-aware,
- operationally legible.

Good documentation should help a reviewer quickly answer:
- what this component is,
- what it does,
- what it depends on,
- what it must not do,
- what assumptions it relies on,
- and what changes would be considered risky.

---

## Change Discipline

When editing repository-root documentation or crate-level governance surfaces:

- keep changes minimal and intentional,
- preserve compatibility-sensitive meaning unless explicitly changing policy,
- avoid silent shifts in architectural or operational interpretation,
- and make non-trivial intent visible to reviewers.

If a document changes the meaning of scope, architecture, compatibility, or security posture, that change should be clearly stated in the associated pull request or review context.

---

## Final Note

Documentation in AOXChain is treated as an engineering surface, not as decorative project metadata.

Where risk, compatibility, runtime behavior, or architectural boundaries are involved, clarity is mandatory.
