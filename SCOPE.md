# AOXChain Scope Statement

This document defines the authoritative engineering scope for repository changes.

## 1. In Scope

- deterministic Layer-1 protocol engineering,
- policy-based authority architecture and migration-safe cryptographic agility,
- validator, governance, and account authority control surfaces,
- replay-domain and recovery-domain semantics,
- readiness gates and evidence production,
- environment and release controls required for disciplined promotion.

## 2. Explicitly Out of Scope

- absolute or permanent security guarantees,
- implicit production-readiness claims without evidence,
- undocumented compatibility promises for experimental surfaces,
- governance or operations paths that bypass protocol validation.

## 3. Sensitive Change Classes

The following require explicit rationale, tests, and synchronized documentation:

- consensus admission and finality semantics,
- authority model and policy evaluation logic,
- profile activation/deprecation and migration rules,
- replay/recovery semantics,
- serialization and storage compatibility behavior,
- validator/governance authorization paths,
- wallet/node key hierarchy and lifecycle controls,
- release/readiness gate definitions.

## 4. Compatibility Policy

Compatibility is managed through explicit release policy and migration documentation.
Breaking changes may be accepted when required for determinism, safety, or architectural integrity, but must include operator guidance and rollback context.

## 5. Evidence Policy

A scope or readiness claim is authoritative only when supported by reproducible commands and retained artifacts.
Claims without evidence are non-authoritative.

## 6. License and Liability Context

AOXChain is distributed under the MIT License on an **"AS IS"** basis, without warranties or liability assumptions by maintainers and contributors except where prohibited by law.
