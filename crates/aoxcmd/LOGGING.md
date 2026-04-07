# AOXC Node Logging Baseline

This document defines a practical, operator-first logging baseline for AOXC node execution.

## Scope

- Runtime terminal logs (`node-run` live output).
- Persistent node logs (file/JSON sinks).
- Audit/security event logs.

## Why previous output feels weak

A node log surface is weak when it only shows block height progression. Production operators also need:

- consensus context,
- network exposure and policy state,
- key material posture,
- endpoint readiness,
- fault domains (storage, RPC, p2p, consensus, policy),
- and direct next actions during incidents.

## Terminal Live Log (Operator HUD)

`node-run` terminal output should always expose:

1. **Boot context panel**
   - profile, bind host, p2p/rpc/metrics ports,
   - policy toggles (key material requirement, genesis requirement, remote peer policy),
   - current boot state (height, produced blocks, network id, last round),
   - key operational state and fingerprint summary,
   - canonical runtime DB path,
   - active RPC and metrics URLs.

2. **Round stream table**
   - round index,
   - timestamp,
   - height,
   - produced block count,
   - section count,
   - consensus round,
   - short block hash,
   - tx identifier.

3. **Debug expansion**
   - parent hash,
   - raw unix timestamp,
   - any additional correlation metadata required for incident triage.

## Persistent Node Logs

Structured persistent logs should include event groups:

- `consensus` (proposal/vote/commit/finality transitions),
- `p2p` (peer connect/disconnect/reject and policy reason),
- `rpc` (method, latency, status, caller class),
- `storage` (state write/read failures, compaction/snapshot events),
- `runtime` (command execution boundaries, round lifecycle),
- `security` (key posture changes, signature/validation failures).

Each record should carry:

- UTC timestamp,
- severity,
- subsystem,
- event name,
- height/round where relevant,
- correlation id,
- deterministic machine-readable fields (JSON-safe keys and values).

## Audit Log

Security-relevant events should be mirrored to a dedicated audit stream:

- key-material state transitions,
- policy override attempts,
- consensus integrity failures,
- invalid block or signature evidence,
- unauthorized RPC operation attempts.

Audit records must be append-only and human-reviewable.

## Operational Rule

Terminal output is for rapid situational awareness.
Persistent and audit logs are for forensics, reliability engineering, and compliance review.
