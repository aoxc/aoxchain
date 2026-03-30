# AOXCHub Architecture

## Components
- `app`: bootstrap and runtime wiring.
- `web`: axum routes for UI assets, state APIs, execution APIs, and SSE streams.
- `services`: environment, command policy, binary policy, and execution orchestration.
- `commands`: immutable compile-time command catalog.
- `binaries`: AOXC binary source discovery and metadata shaping.
- `runner`: constrained process execution, output capture, timeout handling, and job state.
- `security`: local-loopback request gate.
- `embed`: compile-time embedded HTML, CSS, and JavaScript assets.
- `domain`: serialized view models for UI and API responses.

## Data Flow
1. UI loads embedded assets from the same AOXCHub binary.
2. UI calls `/api/state` for environment, binary sources, and command views.
3. Operator selects environment and binary source through policy-aware API calls.
4. Operator confirms command execution.
5. Service resolves immutable command spec to explicit program path plus argument list.
6. Runner executes process without shell indirection and streams output over SSE.

## Dependency Boundaries
- AOXCHub never reimplements AOXC command logic.
- AOXCHub delegates AOXC behavior to selected AOXC binaries and Make targets.
- Command preview strings are generated from the same immutable specs used by execution.

## Operational Boundaries
- MAINNET enforces trusted release source constraints.
- TESTNET allows experimental binary source classes.
- All command execution remains local and explicit.
