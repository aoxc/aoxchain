# Repository Architecture

## Main Components
AOXChain is organized as a modular workspace with protocol crates, runtime and service crates, environment configuration packs, and operational evidence artifacts.

## Data Flow
Configuration and chain parameters flow from `configs/` into node/runtime crates in `crates/`, then into operational workflows through `scripts/` and `tests/`.

## Dependency Direction
Top-level dependency direction should remain:
- Shared primitives (`aoxcore`, utility crates) ->
- Runtime/network/service crates ->
- CLI/UI/operator surfaces.

Operational docs and artifacts must not become hard runtime dependencies.

## Integration Points
Key integration points include RPC boundaries, deterministic network configs, contract deployment references, and release evidence pipelines.

## Security and Logic Boundaries
Consensus, execution, networking, and signing boundaries must remain explicit. Changes affecting deterministic behavior, consensus safety, state transitions, or key handling are security-sensitive and require heightened review.
