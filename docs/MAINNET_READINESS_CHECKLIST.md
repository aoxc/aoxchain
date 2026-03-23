# AOXChain Mainnet Readiness Checklist

**Current Readiness Baseline:** `aoxc.v.0.1.0-testnet.1`

## Build / CI
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] CI policy in root `Cargo.toml` matches actual enforcement commands
- [ ] release artifacts are generated with reproducible checksums and signed manifests

## Consensus / state safety
- [ ] `aoxcunity` finality, fork-choice, equivocation, and quorum tests pass
- [ ] `aoxcmd` block production / state persistence tests pass
- [ ] rollback procedure for consensus-sensitive changes is documented
- [ ] multi-node adversarial simulation evidence is attached to the release record

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
- [ ] backup and restore drill evidence is attached

## Supply chain / audit
- [ ] SBOM is generated for the release candidate
- [ ] artifact signing and provenance attestation are stored with release evidence
- [ ] threat model and attack-surface review are current
- [ ] third-party security assessment report is available or formally waived with expiry

## Operations / response
- [ ] on-call runbook exists and is current
- [ ] incident response drill has been performed recently
- [ ] 10-minute onboarding guide is current
- [ ] escalation owners are named for consensus, networking, execution, keys, and release approval

## Launch gate
Mainnet readiness should be declared only if every checklist item above is either complete or has an explicit, time-bounded exception accepted by the release owner and security approver.
