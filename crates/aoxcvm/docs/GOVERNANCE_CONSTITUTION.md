# AOXCVM Governance Constitution

## Purpose

This document defines governance as constitutional authority, not feature-level policy.

## Governance lanes

Governance authority is separated into explicit lanes:

1. **Constitutional lane**
   - protocol law changes,
   - protected runtime surfaces,
   - compatibility and upgrade constitution updates.

2. **Operations lane**
   - operational parameter updates within constitutional bounds,
   - temporary controls that cannot violate constitutional invariants.

3. **Emergency lane**
   - bounded emergency interventions,
   - mandatory sunset and review requirements,
   - no permanent constitutional mutation without constitutional lane ratification.

## Action taxonomy

Governance actions are typed and classed, including:
- feature activation/deactivation,
- syscall registry updates,
- package promotion and trust-domain actions,
- auth profile registry governance actions,
- upgrade schedule and migration approvals,
- settlement/interop boundary policy actions.

## Governance-only transitions

Certain state transitions are governance-only by law:
- protocol-owned registry mutation,
- protected system contract upgrades,
- canonical policy profile changes,
- compatibility floor/ceiling changes.

Any non-governance invocation of these transitions must fail closed.

## Authority boundaries

- Governance cannot bypass determinism, replay safety, or receipt integrity invariants.
- Governance cannot mutate immutable surfaces without explicit constitutional amendment flow.
- Governance decisions must be attributable to lane, profile, quorum, and action class.

## System contract boundaries

Constitutional system contracts must expose governance boundaries explicitly:
- accepted action classes,
- required signer classes,
- quorum/veto conditions,
- auditable event surfaces.

## Evidence and audit law

Each governance decision must produce audit evidence with:
- action id and class,
- lane id,
- signer profile bindings,
- quorum proof references,
- applied state transition summary,
- compatibility/migration annotations where applicable.
