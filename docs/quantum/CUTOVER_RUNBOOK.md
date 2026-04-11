# Quantum Cutover Runbook

## Scope
Operational sequence for activating the quantum-ready cryptographic profile in a controlled release window.

## Procedure
1. Freeze release inputs and verify artifact checksums.
2. Stage canary validators with hybrid compatibility enabled.
3. Observe handshake and consensus telemetry for one full checkpoint window.
4. Promote PQ-primary profile after compatibility and rollback gates pass.
5. Publish cutover evidence and governance decision record.

## Rollback Rule
If any mandatory invariant fails (consensus validity, signature verification, or handshake safety), immediately revert to last signed stable profile and open an incident record.
