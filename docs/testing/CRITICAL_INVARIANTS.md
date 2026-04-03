# Critical Invariants

This document records validation-critical invariants that must remain true as the codebase evolves.

## Consensus and Finality

- Quorum threshold calculations remain deterministic and monotonic for a fixed validator set.
- Finalized branches reject stale or conflicting votes.
- Validator set transitions cannot silently lower safety thresholds.

## Deterministic Execution and State

- Identical transaction inputs and ordering produce identical post-state outputs.
- Signature-agnostic intent hashing remains stable across signature rotation.
- Failed execution paths do not leak partial state mutations.
- Phase-1 closure requires deterministic agreement across block production canonicalization, fork-choice tie-breaking for equal-height siblings, and AOXCVM replay execution output.
- Phase-4 closure requires tampered state/body commitments to fail closed and stale branches to be rejected after finality.

## Genesis and Runtime Material Integrity

- Canonical genesis and runtime material produce stable fingerprints from identical inputs.
- Runtime install/verify/activate flows fail closed when required artifacts are missing or malformed.
- Runtime receipts remain reproducible for equivalent source bundles.
- Testnet environment bundle identity (`chain_id`, `network_id`, `network_serial`) remains consistent across manifest, validators, bootnodes, and operator metadata files.
- Published `genesis.v1.sha256` remains equal to the digest of the shipped `genesis.v1.json` payload.

## Persistence and Recovery

- Persistence serialization/deserialization round-trips preserve domain meaning.
- Schema migrations preserve backward compatibility guarantees declared by policy.
- Corruption and truncation paths fail closed with actionable diagnostics.

## API and Operator Surfaces

- CLI stderr/error contracts remain deterministic and sanitized.
- Public API contract changes are versioned and regression-tested.
- Unauthorized, malformed, and oversized inputs are rejected without unsafe side effects.
- Transaction ingress continues to fail closed for empty payloads, oversized payloads, malformed sender keys, and structurally invalid signatures.
- Protocol-envelope verification rejects framing corruption, chain/protocol identity mismatches, and payload/frame hash tampering.
- Peer/session ingress denies duplicate peer admission, unknown-session broadcast attempts, and banned-peer traffic.
- Phase-3 closure requires deterministic discovery ordering, spam-resistant mempool admission, and consistent snapshot persistence/restore for synchronization paths.
- Phase-5 closure requires metrics visibility, health/readiness endpoint continuity, and operator alert baseline availability.

## Key and Trust Boundaries

- Signature verification rejects invalid or replayed signed objects.
- Key derivation and role/path semantics remain canonical.
- Certificate and identity validation must fail closed on malformed structures.
- Phase-2 closure requires signature-admission verification, key-rotation continuity checks, hybrid PQ policy enforcement, and domain-separated PQ signature verification to remain deterministic and fail-closed.
- Phase-6 closure requires adversarial and fuzz test surfaces to remain deterministic and fail closed under malformed or conflicting inputs.
- Phase-7 closure requires testnet genesis/topology/public-endpoint integrity checks to pass before launch promotion.
