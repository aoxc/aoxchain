# PQ Authority Implementation Checklist

This checklist is the execution tracker for AOXChain's post-quantum authority architecture.

Status legend:
- `[x]` completed and merged,
- `[ ]` not completed,
- `[-]` intentionally deferred with rationale.

## A) Governance and Canonical Spec

- [x] Define the net objective: migration-safe PQ-native posture without absolute-security claims.
- [x] Define canonical authority flow: `actor -> scheme_id -> policy_root -> proof_bundle -> replay_check -> execute`.
- [x] Define mandatory authority objects (`AccountObject`, `ValidatorObject`, `GovernanceAuthorityObject`, `ReplayState`, `RotationIntent`, `RecoveryIntent`).
- [ ] Approve canonical serialization/version policy for authority objects.
- [ ] Approve governance activation/deprecation windows for each supported `scheme_id`.

## B) Cryptographic Agility and Migration Windows

- [x] Specify ML-DSA primary line and SLH-DSA fallback line in normative docs.
- [x] Specify ML-KEM or controlled hybrid mode for node/session key-establishment planning.
- [x] Document hybrid acceptance windows (classical + PQ) as policy-bounded migration mode.
- [ ] Implement dual-acceptance enforcement in consensus-visible validation path.
- [ ] Add deterministic rejection matrix tests for unsupported scheme combinations.
- [ ] Add downgrade-detection telemetry acceptance criteria.

## C) Transaction Envelope and Witness Model

- [x] Define variable-size public key and signature payload requirements.
- [x] Define proof-bundle and multi-proof container requirements.
- [ ] Finalize canonical tx envelope schema version for PQ/hybrid witnesses.
- [ ] Add compatibility tests for large witness payloads across mempool, RPC, and block inclusion.

## D) Replay, Rotation, and Recovery

- [x] Define domain/intent-scoped replay model.
- [x] Define separation between `policy_root` and `recovery_root`.
- [x] Define native transitions: key rotation, scheme migration, policy rotation, recovery invocation.
- [ ] Implement domain-scoped replay persistence and deterministic rejection logic.
- [ ] Implement recovery authority invocation path with audit evidence output.
- [ ] Rehearse recovery + emergency scheme migration and retain artifacts.

## E) Verification Kernel and Costing

- [x] Define verifier dispatch registry requirement.
- [x] Define scheme-specific deterministic cost accounting requirement.
- [ ] Implement ML-DSA verifier path.
- [ ] Implement SLH-DSA fallback verifier path.
- [ ] Implement hybrid verifier interface and policy coupling.
- [ ] Add deterministic cost-accounting tests and limits.

## F) Validator / Governance / Software Trust Surfaces

- [x] Define validator identity and consensus-signing migration as explicit scope.
- [x] Define node-auth and software/firmware signing migration as explicit scope.
- [ ] Implement validator identity migration runbook.
- [ ] Implement governance execution auth migration runbook.
- [ ] Add signed software/firmware provenance verification policy for release gates.

## G) Evidence and Release Gates

- [x] Define acceptance gates in blueprint (agility, hybrid, recovery, replay, envelope, validator, evidence).
- [ ] Create CI-visible checklist status export artifact.
- [ ] Link each gate to reproducible command set and retained artifact location.
- [ ] Add residual-risk statement template for each PQ migration candidate.

## H) Documentation and Surface Cleanup

- [x] Align quantum blueprint with architecture references.
- [x] Publish this master checklist for progress tracking.
- [ ] Review README/READ surfaces and remove or merge files that do not add operational value.
- [x] Add a repository-wide "README vs READ" policy note to prevent future documentation drift.

## Update Discipline

When a checkbox is changed:

1. include a commit message that references the changed checklist section,
2. include evidence link(s) or command output reference in the associated PR description,
3. avoid marking items complete when only partial implementation exists.
