# VM OVERVIEW

AOXCVM is AOXChain’s deterministic, object-centric, L1-native execution environment.
It is designed to execute transaction payloads under explicit policy control, with strict host
boundaries and forward-compatible cryptography.

## 1) What AOXCVM is

AOXCVM is not only an instruction interpreter. It is the composition of:

- a bytecode acceptance pipeline (decode, canonicalization, verification),
- a policy layer (governance, feature gates, syscall authorization),
- an execution engine (scheduler, executor, rollback/finalization),
- object and state rules (typed object classes, capability checks, storage constraints),
- transaction envelope validation (admission, replay controls, fee model),
- deterministic host interface surfaces (syscalls, imports/exports, limits).

This structure keeps execution semantics stable while still allowing protocol evolution through
explicit governance artifacts.

## 2) Why AOXCVM exists (design intent)

AOXCVM is intended to satisfy these L1 requirements:

- **Determinism first**: identical accepted inputs must produce identical outcomes across nodes.
- **Operational auditability**: policy, governance, and execution boundaries are explicit and testable.
- **Security by construction**: capability model + verifier rules reduce ambient authority.
- **Crypto agility**: authentication and verification surfaces can evolve toward post-quantum profiles.
- **Upgrade discipline**: compatibility and activation are controlled through versioned policy paths.

## 3) Core architecture (high-level)

Execution is organized around the following control flow:

1. **Transaction admission** validates envelope structure, replay domain, and fee constraints.
2. **Package/module loading** checks integrity, compatibility, and governance gates.
3. **Bytecode verification** enforces determinism, state-access, syscall, and capability invariants.
4. **Engine execution** runs instructions under gas, memory, and syscall limits.
5. **Object/state commit** applies class-specific constraints and access policy checks.
6. **Receipt finalization** emits deterministic outcomes, events, and rejection details.

Any violation in these stages results in deterministic rejection and rollback semantics.

## 4) How AOXCVM differs from generic VMs

AOXCVM is intentionally different from a generic smart-contract VM in these ways:

- **Object model is first-class**: state is modeled as typed objects with lifecycle and policy semantics,
  not only unstructured key-value writes.
- **Capability-oriented authority**: operations are gated by explicit capabilities and access witnesses,
  reducing global implicit privilege.
- **Policy-governed syscalls**: host interactions are versioned, whitelisted, and auditable.
- **Governance-integrated evolution**: feature activation/deprecation is part of VM policy, not ad-hoc.
- **Crypto migration surfaces**: auth and verifier paths are structured to support mixed classical/PQ eras.

## 5) Current implementation status (repository reality)

AOXCVM currently contains significant scaffolding plus implemented primitives across tx envelopes,
verification surfaces, policy domains, object classes, syscall registries, and execution lifecycle modules.

However, the repository still includes placeholder/early-baseline documents and evolving implementation
surfaces. Production-readiness must therefore be established through the documented audit, testing,
compatibility, and release-gate artifacts before any deployment decision.

## 6) What “full VM” means in AOXCVM context

In AOXChain, a “full VM” is not just opcode completeness. It means all of the following are mature:

- stable and versioned bytecode format + verifier invariants,
- deterministic engine behavior under adversarial inputs,
- complete syscall registry with governance-controlled authorization,
- object model lifecycle and access control guarantees,
- replay, fee, and receipt semantics finalized for network policy,
- upgrade and compatibility guarantees proven by test/audit evidence,
- operator-facing observability and failure-mode documentation.

## 7) Out-of-scope for this overview

This overview does not redefine the protocol constitution, consensus behavior, or network policy.
It only summarizes AOXCVM’s role and boundaries inside AOXChain.

For normative details, use the dedicated documents under `docs/`, `audit/`, `schemas/`, and
crate-level governance files.
