# aoxcai

`aoxcai` is AOXChain's policy-constrained intelligence extension subsystem. It exists to let AOXChain consume optional AI assistance in tightly bounded operator and review workflows without introducing AI into any kernel authority path.

This crate is not a general-purpose AI platform. It is an audit-oriented control surface for requesting, authorizing, labeling, and recording AI-assisted work products under explicit policy.

## 1. What `aoxcai` is

`aoxcai` is a Rust crate that provides:

- a stable adapter-facing integration boundary,
- an explicit capability taxonomy,
- zone-aware and action-class-bound authorization,
- manifest-driven backend execution,
- deterministic fusion of backend output with native findings,
- pluggable audit sinks,
- conservative fallback behavior when AI is unavailable or disallowed.

In architectural terms, `aoxcai` is an extension plane. It is adjacent to the AOXChain kernel, but it is not part of kernel sovereignty.

## 2. Why it exists

AOXChain has legitimate use cases for bounded intelligence assistance:

- explaining diagnostic failures,
- summarizing incidents,
- drafting remediation plans,
- preparing operator runbooks,
- reviewing artifacts in a non-authoritative manner.

Without a dedicated subsystem, those workflows tend to become informal, weakly audited, and difficult to review. `aoxcai` exists to prevent that outcome. It gives the workspace a single constrained mechanism for AI integration so that maintainers and auditors can reason about the boundary clearly.

## 3. What problem it solves

Probabilistic model output is fundamentally different from kernel-grade deterministic logic. The problem is therefore not “how to add AI everywhere,” but “how to permit narrow assistance without contaminating authority, correctness, or auditability.”

`aoxcai` solves that problem by enforcing the following:

- AI invocation must be explicit.
- AI invocation must be policy-gated.
- AI invocation must be capability-scoped.
- AI invocation must be bound to a kernel zone.
- AI invocation must be classified by action type.
- AI invocation must emit audit evidence, including denials.
- AI failure must degrade assistance only, never native correctness.

## 4. What it does

At the crate level, `aoxcai` performs five core functions.

### 4.1 Authorization

Every invocation is checked against an `InvocationPolicy` using exact-match semantics:

- `KernelZone`
- `AiCapability`
- `AiActionClass`

If the tuple is not granted, the invocation is denied.

### 4.2 Audit capture

Both allowed and denied invocations produce a structured `AiInvocationAuditRecord`. The record is suitable for operator display, tests, and future persistence layers.

### 4.3 Backend mediation

Backends are selected through manifests and constructed through the backend factory. This prevents arbitrary ad hoc backend usage and keeps the integration model reviewable.

### 4.4 Deterministic fusion

Backend output is fused with deterministic findings through policy code so the final assessment is bounded, typed, and reviewable.

### 4.5 Conservative fallback

If a backend fails, times out, or becomes unavailable, the engine can return a manifest-defined fallback assessment rather than silently proceeding. The fallback remains bounded and does not upgrade AI into authority.

## 5. What it explicitly does not do

`aoxcai` does not:

- determine consensus outcome,
- mutate chain state,
- define canonical truth,
- accept or reject contracts as authority,
- control validator behavior,
- control treasury behavior,
- replace native policy evaluation,
- bypass audit,
- make AI mandatory for correctness.

Any design or code change that would move `aoxcai` into one of those roles is out of scope for the current AOXC phase.

## 6. Constitutional model

The crate codifies a constitutional model in `constitution.rs`. The essential rules are:

- AI is not root authority.
- Kernel correctness must remain valid without AI.
- AI output is never canonical truth by itself.
- Every invocation must be capability-scoped.
- Side effects must remain policy-gated.
- Audit is required.
- Constitutional or highly sensitive action classes remain restricted.

This constitutional layer exists to keep architectural intent explicit in code rather than relying on convention.

## 7. Capability model

Capabilities represent bounded assistance scope. They are not authority grants.

Current capabilities include categories such as:

- explanation and validation assistance,
- manifest and compatibility review,
- risk and incident summarization,
- operator diagnostics assistance,
- configuration review,
- remediation planning,
- runbook generation.

A capability should answer one question only: “what kind of assistance is being requested?” It should never imply “what the AI is allowed to control.”

## 8. Action class model

`AiActionClass` distinguishes the constitutional sensitivity of the requested work:

- `Advisory`: explanatory, descriptive, or summarizing output.
- `GuardedPreparation`: preparatory artifacts such as runbook drafts that still require human review and separate execution paths.
- `RestrictedConstitutional`: disallowed for AI authority paths.

Action class is separate from capability on purpose. A capability describes assistance type; an action class describes how sensitive that assistance is in context.

## 9. Adapter integration model

Adapters are the only approved integration path.

The intended shape is:

`caller crate -> local adapter -> aoxcai authorization -> backend/fusion -> audit sink`

This design has several purposes:

- it makes the invoking crate declare zone, capability, and action class explicitly,
- it prevents hidden direct coupling between arbitrary crate code and AI backends,
- it gives reviewers a small number of stable integration points,
- it preserves a clear audit boundary.

For the current phase, `aoxcmd` is the only real integration target.

## 10. Audit model

The audit model is first-class rather than incidental.

Each invocation produces an `AiInvocationAuditRecord` containing, at minimum:

- invocation identity,
- caller crate and component,
- requested action,
- provider identity,
- capability,
- action class,
- kernel zone,
- policy identifier,
- output/input classification,
- approval state,
- final disposition,
- timestamp and execution metadata.

Audit sinks are pluggable through the `AiAuditSink` trait. This allows tests, operator surfaces, and future durable sinks to consume the same record shape without altering authorization logic.

Denied invocations are intentionally recorded. Denial is security-relevant system behavior and must remain observable.

## 11. Failure behavior when AI is disabled

AI is optional for correctness.

If AI is disabled, unreachable, denied by policy, or otherwise unavailable:

- native AOXChain logic continues to run,
- native correctness is unchanged,
- operator assistance may be omitted or replaced by bounded fallback behavior,
- the unavailability or denial should still be auditable.

This behavior is required for kernel safety.

## 12. Why `aoxcai` is kernel-safe

`aoxcai` is kernel-safe for the current AOXC phase because it is structurally prevented from becoming a kernel authority path.

Specifically:

- authorization is explicit and exact-match,
- adapters must declare capability, zone, and action class,
- restricted constitutional actions are denied,
- outputs are treated as advisory or guarded-preparation artifacts,
- no crate path grants AI canonical state mutation authority,
- denial and success are both auditable,
- AI failure affects assistance only.

Kernel-safe does not mean “AI is trusted.” It means the subsystem is intentionally designed so that untrusted AI output cannot silently become sovereign behavior.

## 13. Current integration scope

For this phase, the real integration scope is limited to the operator plane through `aoxcmd`.

Representative operator workflows are:

- diagnostics explanation,
- incident summary,
- remediation planning,
- runbook drafting.

Those outputs must remain explanatory or guarded-preparation only. `aoxcmd` must not automatically execute AI-generated instructions, and native readiness verdicts must remain authoritative.

## 14. Future-safe extension approach

Future expansion should remain conservative.

The approved approach is:

1. define a narrowly bounded new capability,
2. assign it to an explicit zone,
3. bind it to an action class,
4. expose it through a dedicated adapter,
5. make the invocation auditable,
6. preserve AI-optional correctness,
7. avoid any authority implication.

Future work should prefer adding small, reviewable policy entries and adapters rather than broadening the meaning of existing capabilities.

## 15. Explicit non-goals

The following are explicit non-goals for `aoxcai` in the current AOXC phase:

- autonomous validator control,
- consensus participation,
- state transition authority,
- treasury or key authority,
- canonical contract acceptance,
- hidden backend invocation paths,
- automatic execution of AI-generated operator actions,
- replacing native review, policy, or audit processes.

## Module overview

- `adapter.rs`: declares the approved adapter-facing invocation shape.
- `audit.rs`: defines audit records and sink abstractions.
- `capability.rs`: defines capability, zone, action-class, and policy types.
- `constitution.rs`: codifies constitutional rules and authorization restrictions.
- `extension.rs`: performs policy-bound authorization and emits audit evidence.
- `engine.rs`: runs the manifest-driven evaluation path.
- `backend/`: contains bounded backend implementations and factory logic.
- `policy/`: contains deterministic fusion policy.
- `registry.rs`: resolves manifests for tasks.
- `model.rs`: defines normalized request, signal, output, and report models.

## Validation expectations

At minimum, maintainers should keep the following clean for changes affecting this crate or its operator integration:

```bash
cargo fmt --all --check
cargo clippy -p aoxcai -p aoxcmd --all-targets --all-features -- -D warnings
cargo test -p aoxcai -- --nocapture
cargo test -p aoxcmd -- --nocapture
```

## Engineering posture

The correct posture for `aoxcai` is conservative completion, not feature expansion.

The subsystem is intended to be complete for the current phase when it is:

- capability-bounded,
- policy-clean,
- audit-complete,
- operator-safe,
- documentation-complete,
- and testable with confidence inside the AOXChain workspace.
