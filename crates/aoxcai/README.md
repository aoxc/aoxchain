# aoxcai

## Purpose

`aoxcai` is the production-grade **AI Orchestration and Policy Layer** for the AOXChain workspace. 

Because artificial intelligence models are inherently probabilistic and non-deterministic, this crate acts as a strict, auditable bridge. It intentionally avoids raw, unbounded AI execution. Instead, it provides a manifest-driven, policy-centric environment that safely integrates bounded AI inferences, heuristic fallbacks, and remote model interactions into the sovereign core without compromising consensus determinism.

## Core Components

The orchestration layer is designed to be highly conservative and structurally bounded:
- **`manifest.rs`**: Handles deterministic request normalization and strict, typed manifest validation before any AI task is scheduled.
- **`engine.rs` & `registry.rs`**: The central execution coordinator providing registry-safe task binding and routing.
- **`backend::remote_http` & `backend::heuristic`**: Bounded backend executors enforcing hardened remote endpoint policies (timeouts, payload limits), alongside local heuristic fallbacks.
- **`policy::fusion`**: The critical logic responsible for the deterministic fusion of probabilistic model outputs with pre-model verifiable findings, ensuring outputs are safe for state transitions.

## Code Scope

- `src/lib.rs` - Main orchestration entry point.
- `src/backend/` - Remote and local execution backends (HTTP, Heuristics, Factory).
- `src/policy/` - Output fusion, validation policies, and strict safety bounds.
- `src/manifest.rs` & `src/registry.rs` - Task declarations and capability routing.

## Security & Operational Notes

- **Conservative Failure Handling**: AI endpoints can fail, timeout, or hallucinate. The engine must aggressively trap these errors. Any malformed response or timeout must result in a graceful heuristic fallback or explicit rejection, **never** a node panic.
- **Hardened HTTP Boundaries**: All external communication (`remote_http`) must strictly enforce connection timeouts, maximum payload bounds, and endpoint whitelisting to prevent resource exhaustion and DDoS vectors.
- **Deterministic Fusion Strictness**: Before any AI-generated output is allowed to influence chain state or logic, the `fusion` policy must sanitize, type-check, and normalize the output into a 100% deterministic format.
- **Manifest-Driven Execution**: AI tasks cannot be executed arbitrarily. Every request must conform to a statically typed manifest that defines its exact resource bounds and policy constraints.

## Local Validation

Before submitting changes to the AI orchestration layer, ensure all focused unit tests and static analysis checks pass flawlessly:

```bash
cargo fmt --all -- --check
cargo check -p aoxcai
cargo clippy -p aoxcai --all-targets --all-features -- -D warnings
cargo test -p aoxcai -- --nocapture
Related Components
Top-level architecture: ../../README.md

Sovereign Consensus: ../aoxcunity/README.md

Execution Orchestrator: ../aoxcexec/README.md
