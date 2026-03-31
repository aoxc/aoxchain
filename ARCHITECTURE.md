# AOXChain Architecture

AOXChain is organized as a layered system designed to preserve deterministic execution, explicit trust boundaries, and operational auditability. Each layer has a distinct responsibility and must remain disciplined in both dependency direction and runtime behavior.

---

## Component Topology

### Kernel Layer
Core protocol and consensus authority surfaces:

- `aoxcore`
- `aoxcunity`

### Execution Layer
Deterministic execution, virtual machine behavior, and economic metering:

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
