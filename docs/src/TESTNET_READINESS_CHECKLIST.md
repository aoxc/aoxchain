# TESTNET READINESS CHECKLIST

This checklist is the operator-facing closure surface for testnet release promotion.
Mark each item only after attaching evidence for the same commit SHA and runtime profile.

## Identity and Genesis

- [x] `aoxc network-identity-gate --enforce --env testnet --format json` passed.
- [x] `aoxc genesis-validate --strict` passed.
- [x] `aoxc genesis-production-gate` passed.
- [x] `identity/genesis.json`, `identity/validators.json`, `identity/bootnodes.json`, and `identity/certificate.json` are present in active node homes.

## Runtime and Observability

- [x] `aoxc node status` reports healthy runtime on target nodes.
- [x] `aoxc diagnostics-doctor` reports no blocking findings for the release candidate.
- [x] JSON structured logging is enabled in active operator configuration.
- [x] Metrics endpoint is enabled and scrapeable.

## Release Evidence Bundle (`artifacts/release-evidence/`)

- [x] `release-evidence-*.md`
- [x] `build-manifest-*.json`
- [x] `compat-matrix-*.json`
- [x] `production-audit-*.json`
- [x] `sbom-*.json`
- [x] `provenance-*.json`
- [x] `aoxc-*.sig` or `aoxc-*.sig.status`

## Production Closure (`artifacts/network-production-closure/`)

- [x] `production-audit.json`
- [x] `runtime-status.json`
- [x] `soak-plan.json`
- [x] `telemetry-snapshot.json`
- [x] `aoxhub-rollout.json`
- [x] `alert-rules.md`
- [x] `security-drill.json` includes `penetration-baseline`, `rpc-authz`, and `session-replay`.
- [x] `desktop-wallet-compat.json` includes `desktop-wallet`, `aoxhub`, `mainnet`, and `testnet`.

## Final Go/No-Go

- [x] `aoxc testnet-readiness --enforce --format json` returns no blockers.
- [x] Checksum/signature verification logs are archived with release evidence.
- [x] Announcement approval recorded with operator/date/commit SHA.

If any required item is not closed, release status remains **NOT_READY**.
