# AOXChain Network and RPC Security Architecture

## Purpose

This document defines AOXChain's target security architecture for node communication, RPC exposure, and host-level hardening. It is intended as an implementation and review baseline for changes affecting:

- validator and sentry topology,
- peer transport and handshake policy,
- RPC admission controls and abuse resistance,
- cryptographic profile agility (classical, hybrid, post-quantum),
- kernel and operating-system isolation controls.

The goal is not to claim absolute protection. The engineering target is fail-closed behavior, bounded blast radius, and deterministic chain continuity under hostile network conditions.

## Security Objectives

AOXChain networking and RPC layers must preserve the following objectives:

1. **Consensus continuity:** public API degradation must not halt consensus.
2. **Admission asymmetry:** cheap checks must precede expensive work.
3. **Trust segmentation:** internet-facing services cannot share trust boundaries with validator signing paths.
4. **Crypto agility:** cryptographic suites are profile-driven and upgradeable.
5. **Operator verifiability:** all critical controls produce audit-capable evidence.

## Layered Plane Model

### Plane A — Consensus/Core (Private)

Contains validator signing paths, consensus logic, deterministic execution, and canonical state transition boundaries.

Policy:

- no direct public internet ingress;
- no co-location with public RPC edge processes;
- allowlisted connectivity only (sentry and control-plane sources);
- strict mTLS or equivalent authenticated channel enforcement for permitted peers.

### Plane B — Secure P2P (Constrained Exposure)

Contains peer discovery and inter-node message propagation surfaces.

Policy:

- authenticated transport mandatory;
- role-aware peer admission (validator, sentry, archival, observer, bootstrap);
- bounded frame sizes, replay windows, anti-amplification limits;
- protocol downgrade rejection with telemetry.

### Plane C — RPC Edge (Public Exposure)

Contains read and write API ingress.

Policy:

- split by trust and workload: public read, authenticated write, operator-only, internal service;
- per-method cost controls and request budget enforcement;
- aggressive early rejection of malformed or unsupported payloads;
- no direct RPC bypass to consensus-critical internals.

### Plane D — Control and Operations (Restricted)

Contains release, attestation, incident response, and key lifecycle tooling.

Policy:

- multi-party authorization for critical actions;
- signed config and artifact verification;
- operator access via dedicated bastion or equivalent private control network.

## Reference Deployment Topology

Minimum production-safe topology:

- **Validators:** private subnet only; no public ingress.
- **Sentries:** public P2P exposure; validator-facing allowlist path.
- **RPC read gateways:** horizontally scaled, cache-backed, rate-limited.
- **RPC write gateways:** authenticated ingress with stricter quotas and replay controls.
- **Read replicas and archive nodes:** isolated from consensus-critical execution path.
- **Control-plane services:** attestation, release signing, and key ceremony workflows isolated from public ingress.

## Node Communication Model

### Session Establishment Requirements

Node sessions must include:

1. **Pre-auth gate:** stateless retry/cookie, per-source shaping, and early abuse suppression.
2. **Authenticated handshake:** profile-aware key exchange and peer identity validation.
3. **Capability negotiation:** protocol version, crypto profile, frame limits, and compression policy.

Unknown or unsupported capability profiles are rejected fail-closed.

### Cryptographic Profile Strategy

All network cryptography is governed by versioned profiles:

- **Profile v1 (classical):** current classical baseline.
- **Profile v2 (hybrid):** classical + PQ primitives simultaneously.
- **Profile v3 (PQ-preferred):** PQ-dominant transport and signing paths after governance activation.

During migration windows, hybrid validation is mandatory where declared by policy.

## RPC Security Model

### Service Separation

RPC ingress is segmented into distinct surfaces:

- **Public Read RPC:** query-only methods with strict method allowlist and cache-first response policy.
- **Authenticated Write RPC:** transaction submission with identity-bound quotas.
- **Operator RPC:** non-public administration, lifecycle, and emergency controls.
- **Internal RPC:** indexer/analytics/service-mesh-only methods.

### Admission Pipeline

Every RPC request follows deterministic staged admission:

1. L3/L4 source shaping and connection controls.
2. L7 schema and method validation.
3. Identity and quota policy evaluation.
4. Method cost-budget enforcement.
5. Sandbox execution with strict time/memory/result ceilings.

Requests that exceed policy are rejected before VM simulation, signature-heavy validation, or expensive storage traversal.

## DDoS and Abuse-Resistance Controls

Required controls include:

- anycast/scrubbing support where available,
- per-IP and per-identity token buckets,
- handshake retry tokens and anti-amplification policy,
- bounded queue depth for signature verification,
- duplicate transaction suppression,
- adaptive fee floor and mempool pressure controls under attack mode,
- graceful degradation for non-critical API methods.

The design objective is to protect consensus and validator liveness even when public read surfaces are degraded.

## Host and Kernel Hardening Baseline

Node and gateway hosts must apply:

- minimal and patch-managed OS baseline,
- secure/measured boot where hardware permits,
- seccomp + AppArmor/SELinux confinement,
- strict Linux capabilities dropping,
- cgroup CPU/memory/IO quotas per process class,
- socket and conntrack tuning with conservative resource ceilings,
- optional XDP/eBPF early-drop policies for high-volume abuse.

Host hardening changes affecting runtime assumptions must be versioned and auditable.

## Key and Identity Segmentation

AOXChain key usage must remain role-separated:

- validator consensus signing keys,
- transport session keys,
- node identity keys,
- operator/authentication keys,
- artifact and release signing keys.

A single key class must never be reused across consensus signing and public ingress duties.

## Evidence and Operational Readiness

No readiness claim is valid without retained evidence. At minimum, operations must produce:

- profile compatibility matrices,
- downgrade rejection telemetry,
- handshake failure and replay-attempt metrics,
- RPC saturation and reject-reason reports,
- key rotation and attestation records,
- attack simulation and recovery drill outcomes.

Artifacts must be reproducible and retained under repository evidence workflows.

## Compatibility and Change Discipline

Changes impacting network protocol behavior, RPC admission policy, cryptographic profile negotiation, or host hardening controls are compatibility-sensitive and require:

1. explicit documentation updates,
2. deterministic test coverage,
3. operational migration notes,
4. rollback guidance.

## Residual Risk Statement

AOXChain does not claim absolute security. Residual risk remains in third-party dependencies, infrastructure operators, undiscovered protocol defects, and emerging cryptanalytic results. This architecture reduces exposure through segmentation, fail-closed policy, and evidence-governed operations.
