# Quantum Evidence Package Requirements

## Mandatory Artifacts
- Policy fingerprint and signed governance decision.
- Network compatibility matrix for mainnet, testnet, and devnet.
- Runtime performance envelope with declared thresholds and observed values.
- Operator drill logs for cutover and rollback scenarios.
- Gate summary report with blocker count equal to zero for promotion.

## Evidence Integrity Rules
- Every artifact must include commit identifier and UTC timestamp.
- Checksums and signatures must be machine-verifiable.
- Placeholder, synthetic, or missing-generator values are not acceptable for promotion.
