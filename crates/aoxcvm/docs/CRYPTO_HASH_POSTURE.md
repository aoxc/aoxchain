# AOXCVM Crypto Hash Posture

## Scope

This document defines the commitment-hash posture for AOXCVM runtime evidence surfaces that require deterministic, domain-separated, and crypto-agile digest construction.

## Canonical Digest Type

AOXCVM standardizes on `QuantumHardenedDigest` for high-assurance commitment paths.

- `sha3_512`: primary long-horizon preimage margin component.
- `blake3_256`: independent implementation and assumption hedge.
- Canonical byte encoding: `SHA3-512 || BLAKE3-256` (96 bytes).

The naming is intentionally conservative: it signals hardening and margin improvement without claiming absolute quantum immunity.

## Domain Separation

Both digest legs use the same fixed framing:

1. constant tag: `AOXCVM/QHASH/V1`,
2. big-endian domain length (`u32`),
3. domain bytes,
4. payload bytes.

This framing is mandatory for all call sites that rely on deterministic commitment reproducibility.

## Security Posture Statement

`QuantumHardenedDigest` is designed to:

- reduce single-primitive dependency,
- preserve deterministic replay across platforms,
- increase conservative security margin against quantum search-style degradation.

It does **not** claim unconditional resistance to all present or future quantum attacks.

## Upgrade and Agility

Digest framing is version-tagged (`.../V1`) to preserve forward migration capability. Any change to primitives, framing, or output composition requires a version bump and explicit compatibility policy update.

## Compatibility Note

Legacy symbol names are retained as deprecated aliases for migration continuity:

- `QuantumUnaffectedDigest` (type alias),
- `quantum_unaffected_digest` (function wrapper).

New integrations should use canonical `QuantumHardenedDigest` naming.
