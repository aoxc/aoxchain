# Security

## Sensitive Surface
This crate can influence system-level correctness, data integrity, or operator trust boundaries.

## Rules
- Preserve deterministic and auditable behavior.
- Validate untrusted inputs at module boundaries.
- Keep privilege and authority checks explicit.

## Forbidden
- Silent security bypasses.
- Implicit trust of external inputs.
- Hidden behavior changes that alter safety assumptions.

## Reporting
Potential security issues should be handled privately through the repository security disclosure process.
