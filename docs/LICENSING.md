# AOXChain Licensing Strategy

## 1) Chosen repository license model (implemented now)

**Current repository code license:** `AGPL-3.0-only`.

This repository currently applies a single top-level license (`LICENSE`) to avoid contradictory legal messaging during architecture normalization.

## 2) Why AGPL-3.0-only was chosen now

- Aligns with reciprocity goals for sovereign protocol, runtime, and operator stack components.
- Preserves source inspectability and modification rights while requiring network-use sharing obligations.
- Removes prior permissive-MIT mismatch with stated anti-extractive goals.

## 3) Surface coverage under current model

Under current implementation, AGPL-3.0-only applies to repository code including:

- kernel and consensus crates,
- runtime crates,
- system service crates,
- operator/control-plane crates,
- SDK and peripheral crates.

## 4) Planned split-license direction (target state, not implemented yet)

A future split model may be considered after contributor-provenance/legal review:

- core protocol/runtime/system/operator: `AGPL-3.0-only`,
- SDK-only external integration surfaces: potential `Apache-2.0` subset where strategically justified.

This split is **target-state discussion only** until implemented with explicit per-surface license files, SPDX metadata, and contributor sign-off.

## 5) Contribution expectations

- Contributions are accepted under repository license terms (`AGPL-3.0-only`) unless explicit alternative terms are later documented.
- Contributors should not submit code they cannot license under these terms.
- Pull requests should preserve SPDX and licensing clarity when adding new files.

## 6) What code license does not cover

- Trademarks, logos, and brand identity are not granted by AGPL code rights.
- Name/logo usage is governed by `docs/TRADEMARK_POLICY.md`.
- Separate legal agreements may be required for official branding, endorsements, or certification claims.

## 7) Legal ambiguity and history note

This document is an engineering-facing normalization artifact, not legal advice. Historical contributions authored under prior terms may require formal legal review before downstream relicensing assertions beyond this repository state.
