# AOXChain Mainnet Readiness Checklist

## Build / CI

- [x] `cargo fmt --all --check` passes
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [x] `cargo test` passes
- [x] CI policy in root `Cargo.toml` matches actual enforcement commands
- [x] release evidence includes checksum, SBOM, signature status, compatibility matrix, and provenance

## Consensus / state safety

- [x] `aoxcunity` finality, fork-choice, equivocation, and quorum tests pass
- [x] `aoxcmd` block production / state persistence tests pass
- [x] rollback procedure for consensus-sensitive changes is documented

## Network security

- [x] `aoxcnet` secure-mode tests pass
- [x] mutual-auth assumptions are documented and reviewed
- [x] certificate binding, attestation, replay, and handshake controls are verified
- [x] no production profile relies on insecure-mode behavior

## Execution lanes

- [x] `aoxcvm` multi-lane gas accounting tests pass
- [x] lane state isolation and resource-boundary tests pass
- [x] runtime-flow coverage exists for all supported lanes

## Keys / identity / genesis

- [x] genesis artifacts are reproducible
- [x] key rotation / revocation procedures are documented
- [x] identity and genesis test coverage is reviewed before release sign-off

## Operations / response

- [x] on-call runbook exists and is current
- [x] incident response drill has been performed recently
- [x] security drill / penetration baseline evidence is current
- [x] 10-minute onboarding guide is current
- [x] escalation owners are named for consensus, networking, execution, and keys

## Wallet / hub compatibility

- [x] desktop wallet compatibility is verified against AOXHub
- [x] mainnet/testnet routing and signing flows are consistent across wallet clients

## Launch gate

Mainnet readiness should be declared only if every checklist item above is either complete or has an explicit, time-bounded exception accepted by the release owner.

## Progress scoring guidance

- Treat `aoxc mainnet-readiness --format json` as the operator summary for current percentage progress.
- Review both `track_progress.testnet` and `track_progress.mainnet` before promotion decisions.
- `area_progress` should be used to identify which engineering section is furthest from 100%.
- `next_focus` should drive the next closure sprint until every section reports `ready`.
