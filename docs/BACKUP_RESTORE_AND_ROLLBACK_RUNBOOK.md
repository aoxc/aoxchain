# AOX Chain Backup, Restore, and Rollback Runbook

**Operational Version:** `aoxc.v.0.1.0-testnet.1`

## Purpose
This runbook defines the minimum recovery expectations before public testnet or production promotion.

## Backup Scope
Operators must define and test backups for:
- node home and configuration,
- genesis and identity material,
- certificate and revocation artifacts,
- storage snapshots and state exports,
- release manifests and rollback references.

## Restore Procedure
1. Identify the target version and the trusted restore point.
2. Validate backup integrity using checksums.
3. Restore configuration, identity artifacts, and state snapshots into a clean node home.
4. Re-run deterministic bootstrap checks.
5. Verify node health and compare state commitments to the expected restore reference.

## Rollback Procedure
1. Declare rollback authority.
2. Freeze new deployments.
3. Select the previous trusted release artifact and manifest.
4. Restore the prior binary, configuration, and state reference.
5. Execute smoke and health checks before rejoining the network.
6. Record the reason, timing, and approving owner in the incident log.

## Mandatory Drill Frequency
- backup verification: every release candidate,
- restore drill: at least once per release line,
- rollback drill: before public testnet promotion and after any consensus-critical change.
