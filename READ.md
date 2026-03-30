# AOXChain Canonical Technical Definition

## Mission

AOXChain is a modular Layer-1 engineering platform designed to provide deterministic execution, auditable operations, and evidence-based release governance.

## Canonical architecture layers

1. **Consensus and state kernel** (`aoxcore`, `aoxcunity`)
   - Defines canonical state transition primitives and safety/finality rules.
2. **Execution and economics** (`aoxcexec`, `aoxcvm`, `aoxcenergy`)
   - Executes lane-specific workloads under deterministic envelopes and metering constraints.
3. **Platform services** (`aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`)
   - Provides networking, API surfaces, persistence, and configuration controls.
4. **Operator plane** (`aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`)
   - Delivers lifecycle automation, readiness assessment, and release evidence generation.

## Non-negotiable invariants

- **Determinism:** equivalent canonical inputs must yield equivalent canonical outputs.
- **Fail-closed control:** invalid policy/configuration/input states must block forward progress.
- **Evidence traceability:** operational claims must map to reproducible artifacts.
- **Boundary discipline:** operator/UI surfaces may observe and control workflows but must not bypass consensus boundaries.

## Environment and release posture

Environment profiles under `configs/` define localnet, devnet, testnet, mainnet, and sovereign templates. Mainnet progression is controlled by readiness gates and closure evidence under `artifacts/`.

## Governance references

- Repository index and status: `README.md`
- Scope and compatibility: `SCOPE.md`
- Architecture boundaries: `ARCHITECTURE.md`
- Security policy: `SECURITY.md`
- Validation policy: `TESTING.md`

## License and disclaimer

This project is distributed under MIT. All materials are provided "as is" with no warranty and no implied liability by maintainers or contributors.
