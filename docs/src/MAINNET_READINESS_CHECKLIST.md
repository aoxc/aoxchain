# AOXChain Mainnet Readiness Checklist

## Build / CI

- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] CI policy in root `Cargo.toml` matches actual enforcement commands
- [ ] release evidence includes checksum, SBOM, signature status, compatibility matrix, and provenance

## Consensus / state safety

- [ ] `aoxcunity` finality, fork-choice, equivocation, and quorum tests pass
- [ ] `aoxcmd` block production / state persistence tests pass
- [ ] rollback procedure for consensus-sensitive changes is documented

## Network security

- [ ] `aoxcnet` secure-mode tests pass
- [ ] mutual-auth assumptions are documented and reviewed
- [ ] certificate binding, attestation, replay, and handshake controls are verified
- [ ] no production profile relies on insecure-mode behavior

## Execution lanes

- [ ] `aoxcvm` multi-lane gas accounting tests pass
- [ ] lane state isolation and resource-boundary tests pass
- [ ] runtime-flow coverage exists for all supported lanes

## Keys / identity / genesis

- [ ] genesis artifacts are reproducible
- [ ] key rotation / revocation procedures are documented
- [ ] identity and genesis test coverage is reviewed before release sign-off

## Operations / response

- [ ] on-call runbook exists and is current
- [ ] incident response drill has been performed recently
- [ ] security drill / penetration baseline evidence is current
- [ ] 10-minute onboarding guide is current
- [ ] escalation owners are named for consensus, networking, execution, and keys

## Wallet / hub compatibility

- [ ] desktop wallet compatibility is verified against AOXHub
- [ ] mainnet/testnet routing and signing flows are consistent across wallet clients

## Launch gate

Mainnet readiness should be declared only if every checklist item above is either complete or has an explicit, time-bounded exception accepted by the release owner.

## Progress scoring guidance

- Treat `aoxc mainnet-readiness --format json` as the operator summary for current percentage progress.
- Review both `track_progress.testnet` and `track_progress.mainnet` before promotion decisions.
- `area_progress` should be used to identify which engineering section is furthest from 100%.
- `next_focus` should drive the next closure sprint until every section reports `ready`.
