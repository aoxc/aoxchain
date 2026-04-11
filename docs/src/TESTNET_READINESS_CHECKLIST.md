# TESTNET READINESS CHECKLIST

This checklist is the operator-facing closure surface for testnet release promotion.
Mark each item only after attaching evidence for the same commit SHA and runtime profile.

## Identity and Genesis

- [ ] `aoxc network-identity-gate --enforce --env testnet --format json` passed.
- [ ] `aoxc genesis-validate --strict` passed.
- [ ] `aoxc genesis-production-gate` passed.
- [ ] `identity/genesis.json`, `identity/validators.json`, `identity/bootnodes.json`, and `identity/certificate.json` are present in active node homes.

## Runtime and Observability

- [ ] `aoxc node status` reports healthy runtime on target nodes.
- [ ] `aoxc diagnostics-doctor` reports no blocking findings for the release candidate.
- [ ] JSON structured logging is enabled in active operator configuration.
- [ ] Metrics endpoint is enabled and scrapeable.

## Release Evidence Bundle (`artifacts/release-evidence/`)

- [ ] `release-evidence-*.md`
- [ ] `build-manifest-*.json`
- [ ] `compat-matrix-*.json`
- [ ] `production-audit-*.json`
- [ ] `sbom-*.json`
- [ ] `provenance-*.json`
- [ ] `aoxc-*.sig` or `aoxc-*.sig.status`

## Production Closure (`artifacts/network-production-closure/`)

- [ ] `production-audit.json`
- [ ] `runtime-status.json`
- [ ] `soak-plan.json`
- [ ] `telemetry-snapshot.json`
- [ ] `aoxhub-rollout.json`
- [ ] `alert-rules.md`
- [ ] `security-drill.json` includes `penetration-baseline`, `rpc-authz`, and `session-replay`.
- [ ] `desktop-wallet-compat.json` includes `desktop-wallet`, `aoxhub`, `mainnet`, and `testnet`.

## Final Go/No-Go

- [ ] `aoxc testnet-readiness --enforce --format json` returns no blockers.
- [ ] Checksum/signature verification logs are archived with release evidence.
- [ ] Announcement approval recorded with operator/date/commit SHA.
- [ ] Optional live verification completed with `NODE_HOME=<node-home> ./scripts/validation/testnet_live_smoke.sh`.

If any required item is not closed, release status remains **NOT_READY**.
