# aoxcai

> Scope: `crates/aoxcai`

## Purpose

`aoxcai` is the AOXChain AI extension plane.  
It provides policy-constrained, auditable, capability-scoped AI assistance for
non-consensus workflows. It is explicitly non-authoritative and must not be
treated as a kernel authority path.

## Repository role

- Provides reusable AI runtime primitives (manifest handling, backend execution,
  policy fusion, audit artifacts, registry wiring).
- Exposes a library surface for upstream crates and operator tooling.
- Does not define an operator command-line binary by itself.

## Binary and CLI placement model

### Why this crate remains a library

`aoxcai` is intentionally a reusable runtime crate. Keeping it library-only:

- preserves separation of concerns,
- avoids duplicate CLI entrypoints,
- keeps runtime logic reusable for node, RPC, and operator surfaces,
- minimizes release and compatibility complexity.

### Where CLI commands should live

Primary operator CLI commands should be implemented in `crates/aoxcmd` under an
`ai` command group, backed by `aoxcai` APIs.

Recommended command namespace:

- `aoxc ai manifest ...`
- `aoxc ai infer ...`
- `aoxc ai policy ...`
- `aoxc ai audit ...`

If long-term independent distribution is required, a separate `aoxcai-cli`
crate can be introduced later, still consuming `aoxcai` as a library.

## Advanced implementation roadmap (production-oriented)

This roadmap defines a low-chaos, incremental rollout model.

### Phase 0 — Design baseline

- [ ] Define final command taxonomy and naming contract for `aoxc ai ...`.
- [ ] Define stable machine-readable output contract (`--format json`).
- [ ] Define deterministic exit code matrix for CI and automation.
- [ ] Freeze input/output examples for each command family.

### Phase 1 — Manifest and configuration operations

- [ ] Implement `aoxc ai manifest validate <path>`.
- [ ] Implement `aoxc ai manifest explain <path>` (human-readable diagnostics).
- [ ] Implement `aoxc ai manifest policy-check <path>` for endpoint and security checks.
- [ ] Add schema- and policy-focused negative tests.

### Phase 2 — Inference and policy introspection

- [ ] Implement `aoxc ai infer dry-run ...` with explicit non-authoritative labeling.
- [ ] Implement `aoxc ai policy explain --report <path>`.
- [ ] Implement replayable test fixtures for deterministic policy outcomes.
- [ ] Add redaction-safe logging defaults for CLI output.

### Phase 3 — Audit and operational workflows

- [ ] Implement `aoxc ai audit inspect ...` for local artifact analysis.
- [ ] Implement `aoxc ai audit export ... --format {json,yaml}`.
- [ ] Provide correlation metadata (invocation ID, manifest ID, backend type).
- [ ] Add incident-response-oriented command examples.

### Phase 4 — Hardening and release gate

- [ ] Add end-to-end tests across command families.
- [ ] Add compatibility tests for CLI output schema.
- [ ] Add security checks for endpoint controls and secret handling.
- [ ] Add performance checks for bounded runtime and memory behavior.
- [ ] Define release checklist and rollback guidance.

## Production readiness checklist (operator surface)

The following list must be satisfied before declaring production readiness:

- [ ] CLI command contract is documented and versioned.
- [ ] JSON output schema is stable and covered by regression tests.
- [ ] Exit codes are deterministic and documented.
- [ ] Manifest validation covers all security-sensitive fields.
- [ ] Endpoint policy failures are explicit and actionable.
- [ ] Audit artifacts are queryable and exportable.
- [ ] Redaction and sensitive-field handling are validated.
- [ ] Failure modes (timeout/unreachable/schema) are represented consistently.
- [ ] CI gate includes lint, tests, and command-surface regression checks.
- [ ] Rollout and rollback procedures are documented for operators.

## Change discipline for this scope

When editing this crate or its operator surfaces:

- keep policy and trust-boundary language explicit,
- avoid ambiguous CLI behavior,
- preserve deterministic behavior and backward compatibility where required,
- make non-trivial operational changes visible in review context.
