# AOXCHub Security Baseline

## Core Constraints
- Listener is bound to `127.0.0.1:7070`.
- Localhost middleware blocks non-loopback request origins when peer information is available.
- Command execution uses explicit binary path plus argument list.
- No `sh -c`, no shell string interpolation, and no freeform command text input.
- Runtime users cannot edit the command catalog.

## Execution Controls
- Working directory is fixed to repository root for controlled workflow behavior.
- Process output is bounded to avoid unbounded memory growth.
- Process timeout defaults to five minutes to limit stalled executions.
- Exit code, timeout state, and full captured output remain visible to the operator.

## Environment Policy Controls
- MAINNET forbids local release and custom path binaries.
- TESTNET permits local release and custom path binaries for controlled experimentation.
- AOXC command execution validates selected binary source policy at launch time.

## Liability and Governance
AOXCHub participates in AOXChain pre-release software governance under MIT terms. Operators remain responsible for environment hardening, host security, and production change control.
