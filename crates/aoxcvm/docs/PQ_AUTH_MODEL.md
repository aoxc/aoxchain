# PQ AUTH MODEL

This document defines the AOXCVM vNext authentication baseline for quantum-resilient operation.

## Objectives

- keep signature-policy behavior deterministic and serializable,
- permit staged migration from classical signatures to post-quantum signatures,
- preserve governance control over profile transitions.

## Policy Profiles

AOXCVM defines three execution-time authentication profiles:

- `Legacy`: classical algorithms only (`ed25519`, `ecdsa-p256`);
- `HybridMandatory`: at least one classical signer and one post-quantum signer;
- `PostQuantumStrict`: post-quantum signers only.

`HybridMandatory` is the default runtime profile. This allows incremental migration while ensuring every signed action already carries post-quantum evidence.

## Signature Families

Currently modeled algorithms:

- `ed25519`
- `ecdsa-p256`
- `ml-dsa-65`
- `ml-dsa-87`

`ml-dsa-*` algorithms are treated as post-quantum in policy validation.

## Operational Guardrails

- empty signer sets are always invalid;
- profile checks must not depend on host-local entropy or wall-clock state;
- key-rotation flows should require at least one post-quantum signer by default.
- auth envelopes should enforce bounded sizes (default: at most 16 signatures, at most 4096 bytes per signature witness).
