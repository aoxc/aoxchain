# Quantum Near-Closure Tracks

## 1) Kernel Policy Closure

- unknown/unsupported profile payloads are rejected before settlement;
- activation/deprecation windows are explicit and version-bounded;
- classical-only acceptance paths are removed or explicitly windowed.

## 2) Network and Handshake Closure

- peer profile negotiation fails closed on mismatch;
- downgrade attempts are rejected and surfaced in telemetry;
- negotiated behavior is deterministic across supported node roles.

## 3) Migration Closure

- validator/operator key transition path is deterministic and documented;
- persisted consensus artifacts have deterministic migration or controlled reset rules;
- migration rehearsals are retained per release candidate.

## 4) Rollback Closure

- rollback path is explicit, rehearsed, and version-bounded;
- rollback-by-config-drift is blocked by policy and validation checks;
- rollback drill outputs include safety and timing outcomes.

## 5) Evidence and Gate Closure

- required gate commands are reproducible;
- mixed-profile rejection proofs are generated for candidate cutovers;
- final readiness package links commit, commands, artifacts, and residual risk statement.
