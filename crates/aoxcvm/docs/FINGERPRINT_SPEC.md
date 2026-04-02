# AOXCVM Fingerprint Specification

## Scope

This specification defines canonical execution fingerprint generation for AOXCVM evidence and determinism workflows.

## Canonical Type and API

- Function: `canonical_execution_fingerprint(namespace, payload) -> String`.
- Digest backend: `quantum_hardened_digest`.
- Output encoding: uppercase hexadecimal string.
- Output size: 192 characters (96 digest bytes).

## Encoding Contract

Fingerprint output is derived from canonical digest bytes in this order:

1. SHA3-512 bytes,
2. BLAKE3-256 bytes.

The resulting 96-byte buffer is encoded using uppercase hex alphabet (`0-9`, `A-F`) with no separators, prefix, or whitespace.

## Determinism Requirements

For the same `(namespace, payload)` input tuple, implementations must produce byte-for-byte identical fingerprint output across:

- toolchains,
- architectures,
- optimization profiles.

## Namespace Rules

`namespace` is a domain-separation input and is part of consensus-relevant framing for fingerprint production. Distinct namespaces are required for distinct evidence planes (for example: receipt vs state).

## Versioning

This specification is bound to digest framing version `AOXCVM/QHASH/V1`. Any future framing or primitive migration must be introduced through a new version tag and compatibility rollout plan.
