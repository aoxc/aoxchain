# AOXCVM Phase-3 Runtime Constitution

## Status and intent

Phase-3 is the **Sovereign Runtime Evolution Layer**.

Phase-1 established kernel determinism and safety boundaries.
Phase-2 established execution law and contract admission law.
Phase-3 establishes AOXC protocol identity as a long-horizon constitutional runtime.

The target is not "more features." The target is constitutional continuity.

## Constitutional objective

Phase-3 is complete when AOXCVM can enforce a protocol-native constitutional order
across authentication, governance authority, execution capability, package lifecycle,
upgrade control, and settlement/interop boundaries.

## Constitutional pillars

1. **Canonical Auth Law**
   - Typed `AuthProfileId`.
   - Canonical auth profile registry with explicit versioning.
   - Signer-class model and threshold/quorum policy surfaces.
   - Distinct constitutional paths for governance, operations, and system signers.

2. **Governance Constitution**
   - Governance action taxonomy.
   - Governance-only runtime transitions.
   - Protected change lanes: constitutional, operational, emergency.
   - Class-scoped authority over system contracts and upgrades.

3. **Native Execution Direction**
   - AOXC execution identity is protocol-native.
   - Canonical native execution family is specified.
   - Foreign VM compatibility is integration scope, not identity source.

4. **Constitutional Runtime Enforcement**
   - Capability gates bind at syscall and host boundaries.
   - Governance hooks and registry mutations are lane-gated at runtime.
   - Upgrade authority is enforced as runtime law, not convention.

5. **Long-Horizon Continuity**
   - Upgrade constitution, migration law, and compatibility law are explicit.
   - State transition constitution separates protocol-owned and contract-owned state.
   - Law-aware receipts, witnesses, and audit traces remain replayable and reviewable.

## Delivery structure

### Phase-3A: Constitutional Identity

Scope:
- auth constitution,
- profile constitution,
- governance lane model,
- canonical chain identity semantics.

Primary outputs:
- `AUTH_CONSTITUTION.md`
- `GOVERNANCE_CONSTITUTION.md`

### Phase-3B: Sovereign Runtime

Scope:
- native execution direction,
- runtime capability gate binding,
- constitutional system contracts,
- package publication and upgrade law.

Primary outputs:
- `PACKAGE_LAW.md`
- `UPGRADE_CONSTITUTION.md`

### Phase-3C: External Horizon

Scope:
- settlement and interoperability constitution,
- witness and provenance expansion,
- long-horizon continuity controls.

Primary outputs:
- `INTEROP_SETTLEMENT_LAW.md`


## Implementation status snapshot

Current runtime implementation now defines a canonical constitutional telemetry surface in
`src/vm/constitutional_audit.rs`, including deterministic allowed/denied decision records
for policy, governance lane, and active auth-profile decisions.

This directly satisfies the requirement that constitutional telemetry and audit logs are
explicitly defined for policy/governance/profile decisions, while broader Phase-3 delivery
remains gated by the full done criteria listed below.

## Final done criteria

Phase-3 is done only when all are true:

1. Canonical auth law and profile registry model are constitutional and enforced.
2. Governance constitution and lane separation are explicit and runtime-binding.
3. Native execution direction is declared in canonical protocol documentation.
4. Capability gates are enforced at execution time (syscall + host + registry + metadata + upgrade authority).
5. Constitutional system contracts are defined with authority boundaries.
6. Package law is explicit for publication, dependencies, compatibility, and promotion lifecycle.
7. Interop/settlement law is constitutional, governed, and identity-preserving.
8. Upgrade constitution defines mutable surfaces, veto/quorum, compatibility, and migration rules.
9. Identity layer semantics are explicit for account, actor, system, and governed actors.
10. Receipts/proof/witness surfaces carry law-aware provenance.
11. Constitutional telemetry and audit logs are defined for policy/governance/profile decisions.
12. Test matrix includes constitutional, hostile, fuzz, and fail-closed invariant coverage.

If any item above is missing, Phase-3 remains in progress.
