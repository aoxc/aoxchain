# README.md

> Scope: `crates/aoxcexec`

## Purpose
Implements deterministic execution policies, lane envelopes, and execution accounting.

## Quantum-transition authentication posture
- `ExecutionPayload.auth_scheme` supports:
  - `Ed25519` for legacy-compatible transaction admission,
  - `HybridEd25519MlDsa65` for migration-safe dual-signature validation.
- Hybrid payloads require both:
  - a valid Ed25519 detached signature over the canonical execution payload digest,
  - a valid ML-DSA-65 detached signature over a domain-separated PQ message derived from the same canonical digest.
- For `Ed25519`, post-quantum fields are rejected to prevent ambiguous envelope shapes.

## Contents at a glance
- The code and files in this directory define the runtime behavior of this scope.
- The folder contains modules and supporting assets bounded by this responsibility domain.
- Any change should be evaluated together with its testing and compatibility impact.
