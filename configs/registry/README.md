# AOXC Registry Policy

This directory contains the authoritative policy layer for AOXC network identity and binary compatibility.

These files are intended to be stable, governance-controlled, and audit-relevant.

## Files

### `network-registry.toml`
Defines the canonical AOXC network identity model, including:
- family identity,
- `chain_id` derivation,
- `network_serial` policy,
- `network_id` policy,
- canonical environment names,
- reserved ranges,
- governance constraints.

This file is the root identity policy for all AOXC environment manifests.

### `binary-compatibility.toml`
Defines the canonical binary compatibility and provenance expectations for AOXC node artifacts.

This includes:
- single-binary multi-network expectations,
- manifest compatibility requirements,
- genesis compatibility requirements,
- build and provenance expectations,
- runtime rejection rules.

## Governance Expectations

Changes to files in this directory must be treated as release-impacting changes.

Such changes must not be introduced silently and must be accompanied by:
- release evidence,
- audit log updates,
- manifest compatibility review,
- operational review where applicable.

## Stability Expectations

These files should change rarely.

Routine deployment operations should not require modifications here unless:
- identity policy changes,
- compatibility policy changes,
- canonical naming changes,
- new reserved ranges are introduced,
- a new environment class becomes officially supported.
