# Quantum Cutover and Rollback Runbook Baseline

This document defines the minimum execution flow for direct PQ posture cutover and bounded rollback rehearsal.

## Cutover Baseline

1. freeze candidate config/profile inputs;
2. execute mixed-profile simulation and rejection matrix;
3. verify kernel/profile negotiation behavior against expected fail-closed outcomes;
4. perform staged activation in the designated environment track;
5. retain command logs and generated artifacts.

## Rollback Baseline

1. trigger rollback using explicit version-bounded path;
2. verify no implicit rollback by configuration drift;
3. validate settlement safety and profile policy continuity after rollback;
4. retain rollback timing and safety metrics.

## Exit Criteria

Cutover rehearsal is considered acceptable only when:

- all required checks pass without policy bypass,
- downgrade paths remain rejected,
- and evidence package requirements are fully satisfied.
