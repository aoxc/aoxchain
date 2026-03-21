# aoxcai

Production-oriented AI orchestration layer for AOXChain.

This package is intentionally manifest-driven and policy-centric. It provides:

- deterministic request normalization,
- typed manifest validation,
- bounded backend execution,
- hardened remote HTTP endpoint policy enforcement,
- deterministic fusion of model output and pre-model findings,
- registry-safe task binding,
- focused unit tests for critical behaviors.

This implementation is designed to be auditable, deterministic where possible,
and conservative in failure handling.
