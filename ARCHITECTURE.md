# AOXChain Architecture

## Component topology

- **Kernel layer:** `aoxcore`, `aoxcunity`
- **Execution layer:** `aoxcexec`, `aoxcvm`, `aoxcenergy`
- **Service layer:** `aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`
- **Operations layer:** `aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`

## Data and control flow

1. Environment and policy inputs originate in `configs/`.
2. Runtime and protocol crates consume typed configuration and execute deterministic workflows.
3. Node/service surfaces expose telemetry and APIs.
4. Operator workflows assess readiness and emit evidence under `artifacts/`.

## Dependency direction

Dependencies should flow from shared primitives to higher-level services and tools. Operational artifacts and documentation are outputs and governance controls, not runtime dependencies.

## Boundary controls

- Consensus boundaries must remain isolated from non-deterministic side effects.
- Execution lanes must preserve deterministic semantics under common policy constraints.
- Network/API surfaces must not bypass kernel validation.
- Key material handling must remain explicit, auditable, and least-privilege.
