# AOXChain Naming and Versioning Standard (Operational Baseline)

## 1. Objective

Provide a permanent, low-ambiguity naming and versioning policy for AOXChain so that:

- operators can identify network and release surfaces without confusion,
- maintainers can evolve protocol and runtime safely,
- reviewers can audit identity and compatibility changes deterministically.

## 2. Canonical Terminology

Use the following terms as non-interchangeable dimensions:

1. **Brand name**: `AOXChain` (product/system identity).
2. **Asset ticker**: `AOXC` (native asset symbol).
3. **Release line label**: human-facing stream name (for example `AOXC-QTR-V1`).
4. **Workspace/software version**: repository release metadata (`configs/version-policy.toml`).
5. **Network identity tuple**: `chain_id`, `network_id`, `network_serial`.
6. **Crypto profile version**: consensus-visible cryptographic policy stage.

## 3. Core Rules

1. Never use brand/ticker as a substitute for network identity.
2. Never use release-line labels as `chain_id` or `network_id`.
3. Never rely on Git tags alone as runtime truth.
4. Keep network identity tuple registry-derived and policy-enforced.
5. Keep crypto-profile evolution explicit, auditable, and independent from brand naming.

## 4. Recommended Naming Model

### 4.1 Mainnet/Testnet/Validation

- `network_id` pattern: `aoxc-<network-class>-<family-serial>`
- `chain_id` pattern source: registry-derived numeric format (`FFFFCCNNNN`)
- `display_name`: human-facing operational label only

Suggested display labels:

- `AOXC Mainnet (AKDENIZ)`
- `AOXC Testnet (PUSULA)`
- `AOXC Validation (MIZAN)`

### 4.2 Node Names

Role-first and environment-scoped naming is mandatory for new assets:

```text
<env>-<role>-<ordinal>
```

Examples:

- `mainnet-validator-01`
- `testnet-rpc-02`
- `validation-sentry-01`

Legacy fixture aliases may remain temporarily, but they must be explicitly documented as fixture compatibility names.

### 4.3 Layer/Role Extensions

If a new layer, role family, or service plane is introduced:

1. define canonical role slug,
2. define activation policy surface,
3. define trust boundary and validation ownership,
4. define readiness evidence requirement.

No new layer/role should ship without synchronized docs and policy files.

## 5. Source-of-Truth Hierarchy

For identity and compatibility decisions, apply this authority order:

1. `configs/registry/network-registry.toml` (identity derivation policy),
2. `configs/version-policy.toml` (workspace/release governance),
3. `configs/environments/<env>/release-policy.toml` (environment enforcement),
4. `configs/environments/<env>/profile.toml` (runtime profile tuple),
5. `configs/environments/<env>/genesis.v1.json` (runtime instantiation artifact),
6. release notes and Git tags (communication surface only).

## 6. Decision Rule for `AOXC-QTR-V1`

Using `AOXC-QTR-V1` is correct and recommended for:

- release communication,
- Git tag strategy,
- migration narrative.

It is **not** sufficient as the only versioning system. In-repo machine-readable policy files remain mandatory for deterministic runtime checks and auditability.

## 7. Migration Plan (Low-Risk, Permanent)

1. **Terminology freeze**
   - adopt canonical vocabulary in root docs and operator docs.
2. **Identity freeze**
   - preserve registry-derived tuple and override prohibitions.
3. **Node naming normalization**
   - apply role-first naming to new environments and fixtures.
4. **Legacy alias containment**
   - keep only documented compatibility aliases; prevent new alias families.
5. **Policy-backed CI checks**
   - reject naming drift and identity mismatch in validation gates.

## 8. Repository Cleanup Map

- `README.md`: quick glossary and operator-facing naming summary.
- `READ.md`: invariant-level rules for identity and version-axis separation.
- `ARCHITECTURE.md`: layer/role dependency and trust boundary rules.
- `configs/README.md`: runtime configuration authority and naming enforcement.
- `docs/GENESIS_IDENTITY_CHECKLIST.md`: practical identity verification flow.

## 9. Expected Outcome

This policy keeps the system:

- simple in language,
- strict in identity,
- auditable in compatibility,
- safe for long-lived mainnet/testnet operations.
