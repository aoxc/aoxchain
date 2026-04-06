# Versioning Policy

## Purpose

This document defines how AOXChain versions are advanced in a slow, controlled, and reviewable way.

Current active release line target: `AOXC-QTR-V1` (communication label only).

Current workspace release version: `0.2.0-alpha.1`.

## Canonical Sources

- Repository release version: `Cargo.toml` (`[workspace.package].version`)
- Machine-readable governance policy: `configs/version-policy.toml`
- Component schema versions: maintained in component-specific code and manifests (for example protocol/schema/policy version constants)

All three surfaces must remain internally consistent.

## Version Model

AOXChain uses a **hybrid model**:

1. **Global workspace version** (single release number)
   - Represents the release identity for the full repository.
   - Bumped when merged changes alter shipped behavior, operator behavior, or release artifacts.
2. **Component schema tracks**
   - Protocol/schema/policy versions evolve independently when a format or contract changes.
   - Schema bumps are explicit and must be accompanied by migration/compatibility rationale.

This means versioning is **not per-file** and **not per-folder by default**.

## Bump Rules (Global Workspace Version)

- **Patch-level bump**: bug fixes, hardening, tests/docs/scripts with no public compatibility break.
- **Minor-level bump**: backward-compatible feature additions or new operator-visible capabilities.
- **Major-level bump**: compatibility-breaking changes to protocol, APIs, storage, or governance contracts.

## Change Discipline

When changing version-sensitive behavior:

1. Update `Cargo.toml` workspace version when release identity changes.
2. Update `configs/version-policy.toml` to match the new workspace version.
3. If a schema contract changed, bump the relevant schema track and document compatibility impact.
4. Ensure tests pass, including version-governance validation in `tests/`.

## Operator Guidance

If uncertain whether to bump global version, prefer conservative bump and document rationale in PR description.
