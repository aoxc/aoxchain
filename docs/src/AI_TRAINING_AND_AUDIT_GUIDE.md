# AI Training and Audit Guide

## Objective

Define a reliable ingestion and validation method for training or evaluating AI systems on AOXChain engineering content.

## Recommended ingestion order

1. Root governance docs (`README.md`, `READ.md`, `SCOPE.md`, `ARCHITECTURE.md`, `SECURITY.md`, `TESTING.md`).
2. Crate-level READMEs and architecture/scope files under `crates/`.
3. Configuration profiles and deterministic fixtures under `configs/`.
4. Canonical models under `models/`.
5. Generated evidence under `artifacts/`.

## Labeling strategy for AI datasets

- **Normative sources:** governance docs and canonical architecture/scope files.
- **Executable sources:** Rust modules and scripts.
- **Evidence sources:** JSON/markdown artifacts generated during release and readiness workflows.

## Audit-grade traceability requirements

- Every high-level claim should map to concrete file(s) and, when applicable, generated artifacts.
- Training examples should preserve source path metadata and commit identifiers.
- Contradictions between docs and code should be flagged as quality defects.

## Quality controls

- run static checks and tests before dataset extraction,
- include version/date metadata for artifacts,
- capture unresolved blockers separately from accepted risk items.

## Compliance and liability note

Repository materials are MIT-licensed and provided "as is." They are suitable for engineering analysis but do not independently satisfy legal or regulatory compliance obligations.
