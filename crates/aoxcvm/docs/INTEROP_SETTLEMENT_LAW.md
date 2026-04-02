# AOXCVM Interop and Settlement Law

## Purpose

This document defines constitutional rules for external coordination while preserving
AOXC-native protocol identity.

Interop is an integration surface. It is not AOXC's identity source.

## Constitutional position

1. AOXC-native execution semantics remain canonical.
2. External VM compatibility is secondary and policy-bounded.
3. Settlement flows are governed coordination, not implicit trust of foreign execution.

## Settlement-aware model

Settlement operations must provide:
- explicit authority path,
- explicit trust domain,
- deterministic commitment mapping,
- replay-safe witness generation,
- dispute/audit traceability.

## Interop boundaries

Interop mechanisms are bounded by:
- governance-approved lanes,
- profile-bound authority,
- restricted operation gates,
- explicit compatibility contracts,
- deny-by-default foreign capability escalation.

## Bridge and coordination law

Bridge-like functionality must be modeled as constitutional coordination actors with:
- governed route policy,
- authority-scoped execution,
- settlement finality policy,
- mandatory witness and proof emission.

## Evidence model

Interop and settlement events require receipts/witnesses carrying:
- source and destination domain identifiers,
- governing action references,
- profile and lane bindings,
- outcome status and finality stage,
- dispute references when present.

## Safety rules

- No external integration can mutate protected protocol-owned state without governed authority.
- No external execution result is authoritative without AOXC settlement confirmation.
- All unresolved or ambiguous interop states fail closed.
