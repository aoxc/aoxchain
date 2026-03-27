# AOXCVM

Multi-lane virtual machine orchestration for native/system, EVM, WASM, and external execution lanes.

## Mission

AOXCVM is the execution compatibility layer for AOXChain. It is responsible for routing execution lanes while protecting deterministic behavior for consensus-sensitive flows.

## Scope

- execution lane selection and guardrails,
- envelope/payload pre-validation,
- deterministic accounting and execution boundaries,
- lane-level isolation strategy for fault containment.

## Non-goals

- AOXCVM is **not** the desktop operator UI layer.
- AOXCVM should not own human workflow presentation concerns.

## Determinism contract

For the same canonical input envelope and state root assumptions, AOXCVM must produce stable and explainable outcomes.

Minimum expectations:

1. reject malformed envelopes before mutation,
2. preserve explicit error semantics,
3. bound resource consumption per execution lane,
4. keep replay outcomes consistent under documented constraints.

## WASM strategy (recommended)

- Primary candidate: deterministic `Wasmtime` profile with strict host-call policy.
- Secondary candidate: controlled `WasmEdge` profile where deterministic wrappers are enforced.
- Experimental lane: minimal internal deterministic executor for protocol test/fallback scenarios.

## Security verification priorities

- lane-gate validation unit tests,
- malformed payload adversarial tests,
- cross-lane replay consistency checks,
- gas/fuel accounting regressions,
- serialization/hash stability checks.

## Operational references

- Root baseline: `README.md`
- Audit companion: `READ.md`
- Crate roadmap: `crates/aoxcvm/READ.md`
