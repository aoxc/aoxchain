# aoxcore

## Purpose

`aoxcore` is responsible for the **core protocol primitives** domain within the AOXChain workspace.

## Code Scope

- `identity/`
- `genesis/`
- `transaction/`
- `mempool/`
- `state/`
- `block/`
- `native_token.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcore && cargo test -p aoxcore
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)

## Native Token Support

`aoxcore` now exposes a minimal in-memory native AOXC ledger with deterministic transfer/mint receipt helpers for higher-level execution layers.
