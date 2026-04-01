# ROADMAP.md

Repository-level execution roadmap for AOXChain.

## Phase A — Deterministic kernel baseline
- [x] Establish AOXCLang language-family model.
- [x] Add adapter contract and relay-envelope validation surface.
- [x] Add registry-level adapter conformance entrypoint.
- [ ] Wire registry validation into runtime admission path before settlement.

## Phase B — Interoperability safety hardening
- [ ] Define proof verification interfaces per finality class.
- [ ] Add replay-ledger persistence model and conflict forensics output.
- [ ] Introduce canonical relay intent envelope versioning policy.
- [ ] Add deterministic cross-adapter conformance matrix tests.

## Phase C — Adversarial validation and quality gates
- [ ] Add CI adversarial scenarios (replay, malformed proof, delayed finality).
- [ ] Add reorg simulation matrix across representative language families.
- [ ] Add fuzz targets for relay-envelope parser/validator boundaries.
- [ ] Block releases on failed conformance and replay-safety checks.

## Phase D — Operator and production posture
- [ ] Publish readiness scorecards (kernel, adapters, proofs, replay safety).
- [ ] Add telemetry for adapter proof latency and reject causes.
- [ ] Add release evidence artifact bundle for interoperability checks.
- [ ] Finalize migration/versioning playbook for policy surface changes.

## Ongoing discipline
- [ ] Keep documentation aligned with implemented behavior in the same PR.
- [ ] Preserve deterministic behavior and compatibility-sensitive identifiers.
- [ ] Require explicit migration notes when changing protocol-facing semantics.
