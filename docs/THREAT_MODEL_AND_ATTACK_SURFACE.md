# AOX Chain Threat Model and Attack Surface

**Threat Model Version:** `aoxc.v.0.1.0-testnet.1`

## System Boundary
AOX Chain exposes the following primary attack surfaces:
- consensus and validator messaging,
- P2P networking and discovery,
- RPC and client APIs,
- multi-lane execution runtimes,
- key and certificate lifecycle management,
- release pipeline and artifact distribution,
- operator automation and local helper scripts.

## Trust Boundaries
1. Validator-to-validator communication.
2. Node-to-client RPC interaction.
3. Operator workstation to node control plane.
4. Release pipeline to distributed artifacts.
5. Mobile/edge signer to gateway or relay.
6. AI-governance backend to operator decision support.

## Highest-Risk Threat Classes
- equivocation and consensus safety failure,
- replay or stale-state injection,
- P2P eclipse or Sybil admission,
- RPC abuse or authentication bypass,
- cross-lane execution isolation failure,
- compromised release artifact or malicious dependency introduction,
- key exfiltration or revocation bypass,
- operational error during rollback or emergency patching.

## Required Countermeasure Documents
For a production-ready decision, each threat class must map to:
- preventive controls,
- detective controls,
- recovery procedure,
- owning team or role,
- test or audit evidence.

## Current Gap Note
This document establishes the structure of the threat model, but the full control-to-threat mapping still needs to be continuously maintained as a living artifact for every major subsystem release.
