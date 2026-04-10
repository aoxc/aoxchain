# AOXChain Bootstrap Runbook

Deterministic bootstrap execution reference for operators and release governance.

This section decomposes bootstrap into explicit phases so operators can run, audit,
and recover the process without hidden assumptions.

### Bootstrap Phase 0 — Environment and artifact boundary

Purpose:

- Establish the exact environment profile (`localnet`, `devnet`, `testnet`,
  `validation`, or `mainnet`)
- Freeze artifact inputs before node activation

Required inputs:

- `profile.toml`
- `manifest.v1.json`
- `release-policy.toml`
- `certificate.json`

Expected controls:

1. Verify all required files exist and are readable.
2. Validate manifest identity fields are internally consistent.
3. Verify certificate and release policy compatibility with target profile.
4. Record immutable fingerprints (for example SHA-256 digests) in operator logs.

Fail-closed behavior:

- If any required artifact is missing or malformed, bootstrap stops before key
  loading or process startup.

### Bootstrap Phase 1 — Genesis and topology integrity

Purpose:

- Guarantee deterministic chain identity and peer layout before any runtime action

Required inputs:

- `genesis.v1.json`
- `validators.json`
- `bootnodes.json`
- Topology policy files under `topology/`

Expected controls:

1. Validate genesis schema and chain/network identifiers.
2. Validate validator set structure, uniqueness, and identity linkage.
3. Validate bootnode records and endpoint formatting.
4. Validate topology matrix and role mapping constraints.
5. Confirm hash compatibility between genesis and manifest references.

Fail-closed behavior:

- Any mismatch across genesis/validator/bootnode/topology surfaces aborts
  bootstrap and prevents partial startup.

### Bootstrap Phase 2 — Node identity and local trust material

Purpose:

- Confirm each node starts with deterministic and non-conflicting local identity

Required inputs:

- Node identity material in environment-specific node homes
- Local host mapping and socket matrix policy

Expected controls:

1. Verify node home layout and permissions.
2. Verify seed/key files are present for all required nodes.
3. Validate local endpoint uniqueness (no port collisions in role topology).
4. Verify node-role assignments match consensus policy expectations.

Fail-closed behavior:

- Missing identity artifacts, permission violations, or endpoint collisions block
  process launch.

### Bootstrap Phase 3 — Controlled process activation

Purpose:

- Activate processes in a deterministic order with immediate health feedback

Expected controls:

1. Start required bootstrap node set first.
2. Start remaining validator/observer nodes according to role topology.
3. Perform bounded readiness checks after each start stage.
4. Abort and roll back startup if critical readiness thresholds are not met.

Fail-closed behavior:

- Startup halts when required quorum or core service readiness is not achieved
  within policy-defined limits.

### Bootstrap Phase 4 — Post-start verification and smoke signals

Purpose:

- Prove network operability immediately after activation

Expected controls:

1. Run chain/rpc health checks.
2. Run finality smoke checks.
3. Run transfer smoke checks where required by environment policy.
4. Export deterministic status and evidence artifacts.

Fail-closed behavior:

- A network that starts but fails post-start smoke checks is treated as not
  bootstrap-complete.

### Bootstrap Phase 5 — Audit closure and handoff

Purpose:

- Create a reviewable closure package for operations and governance consumers

Expected controls:

1. Export fingerprints, health summaries, and smoke outcomes.
2. Record bootstrap timestamp window and operator command trace.
3. Persist closure artifacts under `artifacts/` or the environment evidence path.
4. Mark bootstrap state as complete only after evidence export succeeds.

Fail-closed behavior:

- Bootstrap success is not declared until audit closure artifacts are durable and
  reviewable.

### Minimal acceptance definition for "bootstrap complete"

Bootstrap is complete only when all of the following are true:

1. Artifact validation passed (Phase 0 and Phase 1).
2. Node identity and topology checks passed (Phase 2).
3. Deterministic startup reached required readiness thresholds (Phase 3).
4. Health/finality/smoke checks passed (Phase 4).
5. Audit closure artifacts were exported and persisted (Phase 5).

Any missing condition must be reported as "bootstrap incomplete" in operator output.

### Full bootstrap execution checklist (operator runbook)

Use this checklist as the exact execution order for a production bootstrap event.
Each item should produce an auditable signal (stdout log line, structured status
record, or stored artifact).

#### Stage A — Pre-execution guards

1. Confirm approved change window and operator identity.
2. Confirm target environment (`localnet`/`devnet`/`testnet`/`validation`/`mainnet`).
3. Confirm host clock synchronization and monotonic time health.
4. Confirm required binaries and expected binary fingerprint.
5. Confirm writable evidence directory and retention policy.

#### Stage B — Artifact lock and fingerprinting

6. Lock `profile.toml`, `manifest.v1.json`, `release-policy.toml`, and `certificate.json`.
7. Compute and store SHA-256 fingerprint for each locked artifact.
8. Validate chain identity tuple consistency (`chain_id`, `network_id`, `network_serial`).
9. Validate certificate subject and policy binding for the selected environment.
10. Abort immediately if any identity or signature mismatch is detected.

#### Stage C — Genesis and topology verification

11. Validate `genesis.v1.json` schema and deterministic ordering expectations.
12. Validate `validators.json` membership uniqueness and key material format.
13. Validate `bootnodes.json` endpoint scheme, host, and port formatting.
14. Validate `topology/` policy compatibility (`role-topology`, `socket-matrix`, consensus policy).
15. Verify manifest references resolve to the exact genesis/validator/bootnode fingerprints.

#### Stage D — Node identity readiness

16. Verify each planned node home exists with expected permissions.
17. Verify required identity files are present and readable per node.
18. Verify no duplicate node identity appears across configured nodes.
19. Verify endpoint uniqueness and absence of role-port collisions.
20. Abort on first identity or endpoint conflict; do not partially continue.

#### Stage E — Deterministic activation and readiness

21. Start bootstrap-critical nodes in defined order.
22. Start remaining nodes by role tier and record per-stage readiness.
23. Enforce bounded wait windows for quorum and required internal services.
24. Trigger rollback/stop routine if readiness gate is missed.

#### Stage F — Post-start proof and closure

25. Run health, finality, and transfer smoke checks according to environment policy.
26. Export runtime status, fingerprints, smoke outputs, and command trace.
27. Produce a bootstrap closure record with start/end timestamps and operator identity.
28. Mark status as `bootstrap_complete=true` only after closure artifacts persist successfully.
29. Mark status as `bootstrap_complete=false` if any gate fails or evidence export is incomplete.

### Bootstrap evidence package (minimum required files)

A bootstrap event should publish at least the following evidence outputs:

- artifact fingerprint report (all locked inputs)
- startup order and readiness gate log
- health/finality/transfer smoke outcomes
- failure record (if any), including first failing gate and stop reason
- closure manifest referencing all evidence files and hashes

Without this evidence package, bootstrap must be treated as operationally incomplete.

### Bootstrap gate matrix (must-pass)

| Gate ID | Gate description | Required evidence | Failure classification | Required action |
| --- | --- | --- | --- | --- |
| G0 | Environment and artifact boundary validation | Artifact fingerprints + manifest identity check output | Configuration integrity failure | Stop before any node start |
| G1 | Genesis/validator/bootnode/topology coherence | Cross-file consistency report | Determinism boundary failure | Stop and regenerate validated inputs |
| G2 | Node identity and endpoint uniqueness | Node-home and endpoint collision report | Local trust or routing failure | Stop and fix identity or topology conflicts |
| G3 | Deterministic activation readiness | Staged startup logs + readiness thresholds | Runtime readiness failure | Trigger rollback/controlled stop |
| G4 | Post-start health/finality/smoke verification | Health/finality/transfer smoke records | Functional verification failure | Mark bootstrap incomplete and hold rollout |
| G5 | Audit closure durability | Closure manifest with file hashes and timestamps | Evidence durability failure | Keep bootstrap incomplete until persistence succeeds |

A bootstrap attempt is valid only when **all gates G0-G5 pass** in order.

### Failure taxonomy and operator response

Use the following failure classes to keep incident and recovery handling deterministic:

- `F-CONFIG`: manifest/profile/policy/certificate inconsistency (maps to G0)
- `F-GENESIS`: genesis/validator/bootnode/topology mismatch (maps to G1)
- `F-IDENTITY`: node identity artifact or endpoint uniqueness failure (maps to G2)
- `F-READINESS`: quorum/readiness threshold miss during activation (maps to G3)
- `F-SMOKE`: post-start health/finality/transfer verification failure (maps to G4)
- `F-EVIDENCE`: closure artifact export or durability failure (maps to G5)

Required response contract:

1. Emit first failing gate and failure class as a machine-parseable status line.
2. Emit operator-visible summary with immediate next action.
3. Persist partial evidence generated before failure.
4. Mark bootstrap state as incomplete and block promotion.

### Environment-specific bootstrap strictness

All environments follow the same gate model, but strictness differs by risk posture:

- `localnet`: permissive for developer velocity; still requires identity and topology checks.
- `devnet`: moderate strictness; smoke and evidence export required for team handoff.
- `testnet`: high strictness; full gate pass required before external test activity.
- `validation`: production-like strictness with explicit audit package retention.
- `mainnet`: maximum strictness; no gate bypasses, no partial promotion, mandatory closure durability.

Any policy override must be explicit, reviewable, and documented in environment release governance.

### Bootstrap completion and sign-off ("is it finished?")

Operational answer is **yes** only when the following sign-off checks are true:

1. Gate sequence `G0` through `G5` completed with pass status.
2. Closure manifest exists and all referenced files are present and hash-verified.
3. Final status record explicitly states `bootstrap_complete=true`.
4. No open failure class (`F-*`) remains unresolved for the same bootstrap window.
5. Responsible operator and reviewer identities are recorded in the closure record.

If any condition above is false, the only valid status is `bootstrap_complete=false`.

### Machine-parseable bootstrap status contract

Bootstrap tooling should emit a deterministic status payload for every run so
operators, automation, and governance reviewers consume the same closure model.

Recommended fields:

- `bootstrap_id`: unique run identifier
- `environment`: target environment name
- `started_at` / `finished_at`: RFC3339 timestamps
- `gate_results`: ordered map for `G0`-`G5`
- `failure_class`: nullable `F-*` classification
- `bootstrap_complete`: boolean completion flag
- `evidence_manifest_path`: closure manifest location
- `operator_id` and `reviewer_id`: accountability identities

Illustrative payload shape:

```json
{
  "bootstrap_id": "2026-04-04T12:00:00Z-mainnet-001",
  "environment": "mainnet",
  "started_at": "2026-04-04T12:00:00Z",
  "finished_at": "2026-04-04T12:14:32Z",
  "gate_results": {
    "G0": "pass",
    "G1": "pass",
    "G2": "pass",
    "G3": "pass",
    "G4": "pass",
    "G5": "pass"
  },
  "failure_class": null,
  "bootstrap_complete": true,
  "evidence_manifest_path": "artifacts/bootstrap/closure-manifest.json",
  "operator_id": "ops-01",
  "reviewer_id": "release-approver-01"
}
```

If `bootstrap_complete=false`, `failure_class` must be non-null and the first
failing gate must be explicitly identified in the emitted status payload.

### Stage-to-command mapping (implementation-aligned)

This mapping keeps the runbook operationally aligned with current repository
entrypoints while preserving the rule that critical validation belongs in `aoxc`.

| Runbook stage | Primary command surface | Expected output |
| --- | --- | --- |
| Stage A-B | `make doctor` + `aoxc` validation subcommands | preflight and artifact validation status |
| Stage C-D | `aoxc genesis-*`, `aoxc config-*`, `aoxc validator-*` | deterministic config and identity checks |
| Stage E | `make network-start` (or environment launch wrapper) | staged startup and readiness evidence |
| Stage F | `make audit-chain`, smoke commands, and closure export | post-start verification and closure artifacts |

Shell wrappers may orchestrate call order, but pass/fail truth must come from
`aoxc` validations and deterministic status outputs.
