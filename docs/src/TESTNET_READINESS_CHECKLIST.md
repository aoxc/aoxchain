# AOXChain Testnet Readiness Checklist

## Build / CI

- [x] `cargo fmt --all --check` passes
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [x] `cargo test` passes
- [x] CI policy in root `Cargo.toml` matches actual enforcement commands
- [x] release evidence includes checksum, SBOM, signature status, compatibility matrix, and provenance

## Network and public-access hardening

- [x] deterministic testnet profile is generated and validated
- [x] peer policy, rate-limit, and secure-mode defaults are documented
- [x] transport/replay controls are verified for public testnet exposure
- [x] no testnet launch profile depends on undocumented insecure-mode assumptions

## Runtime and state safety

- [x] node bootstrap and node state persistence are reproducible
- [x] state snapshot and recovery drill evidence is available
- [x] consensus/finality paths are covered by automated tests

## Keys / identity / genesis

- [x] genesis artifacts are reproducible
- [x] operator key bootstrap / rotation runbooks are current
- [x] identity and genesis test coverage is reviewed before public testnet announcements

## Operations / observability

- [x] telemetry metrics and structured JSON logs are enabled
- [x] soak plan and telemetry snapshot artifacts are current
- [x] incident response and security drill evidence is current
- [x] escalation owners are named for consensus, networking, execution, and keys

## Wallet / hub compatibility

- [x] desktop wallet compatibility is verified against AOXHub
- [x] mainnet/testnet routing and signing flows are consistent across wallet clients

## Launch gate

Testnet readiness should be declared only if every checklist item above is either complete or has an explicit, time-bounded exception accepted by the release owner.

## Progress scoring guidance

- Treat `aoxc testnet-readiness --format json` as the operator summary for current testnet percentage progress.
- Treat `aoxc mainnet-readiness --format json` as the promotion summary that must remain aligned with testnet closure.
- `area_progress` should be used to identify which engineering section is furthest from 100%.
- `next_focus` should drive closure work until every section reports `ready`.
