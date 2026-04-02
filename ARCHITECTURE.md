# AOXChain Architecture

AOXChain is organized as a layered system designed to preserve deterministic execution, explicit trust boundaries, and operational auditability. Each layer has a distinct responsibility and must remain disciplined in both dependency direction and runtime behavior.

---

## Component Topology

### Kernel Layer
Core protocol, consensus authority, and interoperability-intelligence surfaces:

- `aoxcore`
- `aoxcunity`

### Execution Layer
Deterministic execution engines and economic metering, kept modular behind kernel policy:

- `aoxcexec`
- `aoxcvm`
- `aoxcenergy`

### Service Layer
Networking, RPC, persistence, and typed configuration control surfaces:

- `aoxcnet`
- `aoxcrpc`
- `aoxcdata`
- `aoxconfig`

### Operations Layer
Operator tooling, workflow orchestration, runtime control, and supporting scripts:

- `aoxcmd`
- `aoxckit`
- `aoxchub`
- `scripts/`

---

## Data and Control Flow

AOXChain follows a controlled and traceable flow of configuration, execution, and operational evidence:

1. **Environment and policy inputs** originate in `configs/`.
2. **Runtime and protocol crates** consume typed configuration and execute deterministic workflows.
3. **Node and service surfaces** expose telemetry, APIs, and operational interfaces.
4. **Operator workflows** evaluate readiness, lifecycle state, and release posture, then emit evidence under `artifacts/`.

This flow is intentionally structured so that operational actions remain observable and protocol behavior remains reproducible.

For cross-chain workflows, kernel control flow is explicit:

1. **Ingress classification:** the kernel resolves foreign chain profile, authority domain, and proof/finality expectations.
2. **Canonicalization:** foreign events and intents are normalized into canonical cross-chain message records.
3. **Verification dispatch boundary:** proof-type selection and verifier dispatch are decided at kernel boundary interfaces.
4. **Settlement policy evaluation:** inbound and outbound settlement decisions are evaluated under deterministic policy classes.
5. **Execution delegation:** if execution is required, the kernel delegates to execution modules with explicit constraints and expected outputs.
6. **Finality bookkeeping:** finality class and replay-protection state are persisted as canonical kernel truth.

---

## Dependency Direction

Dependencies must flow **from foundational primitives upward** into higher-level services and tools.

### Expected Direction
- Shared primitives and protocol-safe types should sit at the base.
- Execution and service crates may depend on lower layers, but not bypass them.
- Operational tooling may observe and orchestrate runtime behavior, but must not redefine protocol truth.

### Prohibited Direction
- Runtime-critical code must not depend on operator artifacts as execution prerequisites.
- Documentation, reports, and release evidence are governance outputs, not runtime dependencies.
- Higher-level convenience layers must not introduce hidden influence over kernel behavior.

This directional discipline exists to preserve determinism, auditability, and architectural clarity.

---

## Boundary Controls

AOXChain treats boundary enforcement as a first-class architectural concern.

### Consensus Boundary
- Consensus logic must remain isolated from non-deterministic side effects.
- Kernel truth must not depend on UI behavior, local convenience helpers, or operator shortcuts.
- Validation and finality-critical decisions must remain explicit and reproducible.

### Interoperability Boundary (Kernel-First)
- Foreign chain identity classification is a kernel concern, not an RPC or execution concern.
- Proof type taxonomy and verifier dispatch decisions are kernel policy surfaces.
- Finality class interpretation must be explicit, typed, and auditable at kernel level.
- Authority-domain mapping and universal identity translation must remain deterministic and fail-closed.
- Replay protection and message-domain separation must be enforced before execution dispatch.
- Cross-chain routing semantics must be canonicalized in kernel-owned types.

### Execution Boundary
- Execution lanes must preserve deterministic semantics under shared policy constraints.
- Metering, state transition behavior, and execution outputs must remain stable for identical canonical inputs.

### Network and API Boundary
- Network and API surfaces must not bypass kernel validation.
- External requests, peer inputs, and service adapters must be treated as untrusted until explicitly validated.
- Transport convenience must never supersede protocol correctness.

### Key Material Boundary
- Key handling must remain explicit, auditable, and least-privilege.
- Signing authority, validator identity, and operational access should remain clearly separated wherever possible.
- Sensitive material must never flow through ambiguous or convenience-oriented interfaces.

---

## Architectural Intent

The architecture is designed around a small number of durable principles:

- **Determinism over convenience**
- **Explicit boundaries over implicit behavior**
- **Auditable control flow over hidden coupling**
- **Operational evidence over unverifiable claims**
- **Kernel-native interoperability over execution-centric identity**

AOXChain is therefore positioned as:

- execution-agnostic where possible,
- proof-aware,
- finality-aware,
- policy-governed,
- interoperability-native at the kernel layer.

Execution remains essential but secondary: execution engines are replaceable implementation surfaces, while cross-chain trust interpretation and settlement safety are kernel responsibilities.

---

## Proposed Responsibility Map (Layer-Aligned)

### `aoxcore` (Kernel protocol domain)
- chain profile registry types and lookup contracts;
- canonical cross-chain message model and domain separation rules;
- proof type registry interfaces and verifier dispatch boundary traits;
- finality classification types and decision inputs;
- settlement policy evaluation input/output models;
- authority-domain and universal identity mapping boundaries;
- replay protection and canonical routing keys.

### `aoxcunity` (Kernel consensus and finalization)
- local consensus safety/liveness and constitutional constraints;
- anchoring kernel finality decisions into chain state;
- deterministic integration points for cross-chain settlement admission.

### `aoxcexec` + `aoxcvm` + `aoxcenergy` (Execution layer)
- deterministic state execution and metering under kernel-provided policy decisions;
- no redefinition of foreign trust, proof validity class, or finality semantics.

### `aoxcnet` + `aoxcrpc` + `aoxcdata` + `aoxconfig` (Service layer)
- transport, storage, API, and configuration delivery of kernel-defined types;
- no authority to override kernel settlement policy outcomes.

### `aoxcmd` + `aoxckit` + `aoxchub` + `scripts/` (Operations layer)
- orchestration, diagnostics, evidence generation, and operational control;
- no influence over canonical protocol truth.

Every non-trivial change should preserve or strengthen these principles.

---

## Engineering Note

When modifying AOXChain architecture-sensitive code, contributors should evaluate whether the change:

- preserves dependency direction,
- keeps kernel logic isolated,
- avoids non-deterministic side effects,
- maintains explicit validation boundaries,
- and improves, rather than weakens, auditability.

If a change affects any of these properties, the architectural intent should be documented clearly in the associated pull request and, where appropriate, in repository documentation.


## Quantum-Profile Architecture Alignment (v2)

### Kernel (`aoxcore`, `aoxcunity`)
- owns cryptographic profile truth, activation state, and deprecation policy boundaries;
- validates profile identifiers and rejects unknown or invalid profile payloads before settlement.

### VM (`aoxcvm`, `aoxcexec`, `aoxcenergy`)
- executes profile-gated cryptographic verification paths under deterministic metering;
- cannot locally override kernel-selected profile policy;
- must preserve deterministic behavior for identical inputs across supported hardware classes.

### Network (`aoxcnet`, `aoxcrpc`)
- enforces handshake/profile negotiation constraints and downgrade protections;
- exports operator-visible telemetry for profile mismatches and rejected downgrade attempts.

### Operations (`aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`)
- orchestrates staged rollout and rollback drills for profile transitions;
- emits reproducible readiness artifacts used by release and closure gates.

This alignment must remain consistent with `QUANTUM_ROADMAP.md` and `QUANTUM_CHECKLIST.md`.

---

## Network and RPC Security Addendum

`NETWORK_SECURITY_ARCHITECTURE.md` defines AOXChain's deployment and trust-boundary requirements for:

- validator, sentry, RPC, and control-plane separation,
- profile-driven network cryptography and downgrade rejection,
- staged RPC admission and DDoS-resilience controls,
- kernel/host isolation and resource-confinement baseline.

This addendum is normative for networking and API changes that can affect consensus continuity, availability posture, or key-trust boundaries.
