# READ.md

> Scope: `crates/aoxcvm`
> System role: AOXChain execution/interoperability kernel surface

## 1) Purpose and system alignment
`aoxcvm` is the execution-kernel surface that normalizes lane execution and language-policy behavior under deterministic host controls.

System alignment principles:
- deterministic execution and replay behavior,
- explicit proof-gated relay admission,
- stable lane-to-language policy mapping,
- auditable error and evidence boundaries.

This scope is implementation-critical for runtime correctness and interoperability safety.

## 2) In-scope kernel responsibilities
- Route canonical transactions to lane executors.
- Preserve deterministic gas/resource accounting across lanes.
- Enforce language-family policy metadata used by adapters.
- Validate relay envelopes before settlement path admission.
- Reject replay collisions through deterministic message-id checks.

## 3) Core runtime surfaces
- `vm_kind`: lane identity and lane-to-language mapping.
- `language`: language-family profile model (ABI/state/finality requirements).
- `language_adapter`: adapter contract, replay tag model, relay envelope checks, conformance harness.
- `adapter_registry`: default adapter selection and batch conformance facade.
- `routing`: dispatcher-based lane execution routing.
- `host`: deterministic storage, gas charging, event emission surface.

## 4) Invariants (must not regress)
1. Same input + same host state => same output and gas profile.
2. No relay settlement without finality proof payload.
3. Duplicate relay `message_id` values are rejected deterministically.
4. Lane storage namespaces remain isolated (no cross-lane key collisions).
5. Policy IDs and replay-domain tags remain stable across patch updates.

## 5) Security boundary notes
- Adapters are admission gates, not trust shortcuts.
- Proof-kind declaration is mandatory for adapter profiles.
- Replay-domain derivation must be deterministic and non-empty.
- Source/target chain separation is required for relay envelopes.

## 6) Compatibility contract
When modifying this crate, preserve:
- public enum stability for active policy/lane surfaces,
- serialized/canonical identifiers used in replay and policy selection,
- deterministic validation outcomes for identical input batches.

If incompatibility is intentional, include explicit migration and versioning notes in the same change.

## 7) Operational readiness checklist (crate-local)
- [ ] all lane adapters expose proof-kind + replay-domain metadata,
- [ ] kernel relay precheck is wired into scheduler/dispatcher boundaries,
- [ ] adversarial replay/reorg cases are covered in CI,
- [ ] regression budget is defined for gas/resource drift,
- [ ] release evidence includes adapter conformance results.

## 8) Current status summary (April 1, 2026)
- Deterministic multi-lane skeleton is present and tested.
- Language-first policy and adapter abstractions are present.
- Registry-level conformance validation exists.
- Production-grade all-chain light-client/finality coverage remains pending.
