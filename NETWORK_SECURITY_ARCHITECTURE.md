# AOXChain Network and RPC Security Architecture

This document defines the security baseline for node communication, RPC exposure, and host-level hardening.

## 1. Security Objectives

Networking and RPC surfaces must preserve:

1. consensus continuity under public-edge degradation,
2. admission asymmetry (cheap checks before expensive work),
3. trust segmentation between internet-facing and validator-signing paths,
4. profile-governed cryptographic agility,
5. operator-verifiable security evidence.

## 2. Plane Model

### Plane A — Consensus/Core (Private)

- no direct public ingress,
- allowlisted connectivity only,
- authenticated channels required for permitted peers.

### Plane B — Secure P2P (Constrained Exposure)

- authenticated transport mandatory,
- role-aware peer admission,
- bounded frame sizes and replay windows,
- protocol downgrade rejection with telemetry.

### Plane C — RPC Edge (Public Exposure)

- surface separation (public read, authenticated write, operator-only, internal),
- method-level budget and quota enforcement,
- early rejection of malformed/unsupported payloads,
- no direct bypass to consensus-critical internals.

### Plane D — Control and Operations (Restricted)

- multi-party authorization for critical actions,
- signed artifact/config verification,
- dedicated control network or bastion access.

## 3. Node Session Requirements

Every session must include:

1. pre-auth abuse suppression,
2. authenticated handshake with profile-aware identity validation,
3. capability negotiation (protocol/profile/frame/compression).

Unknown or unsupported capabilities are rejected fail-closed.

## 4. Cryptographic Profile Strategy

Versioned profile model:

- Profile v1: classical baseline,
- Profile v2: hybrid classical + PQ,
- Profile v3: PQ-preferred after governed activation.

Hybrid enforcement is mandatory during declared migration windows.

## 5. RPC Admission Pipeline

Deterministic staged admission:

1. L3/L4 source shaping,
2. schema and method validation,
3. identity and quota evaluation,
4. cost-budget enforcement,
5. bounded execution (time, memory, result size).

Requests exceeding policy are rejected before expensive validation/execution phases.

## 6. Abuse-Resistance Controls

Required controls include:

- per-IP/per-identity token buckets,
- retry tokens and anti-amplification limits,
- bounded signature-verification queue depth,
- duplicate transaction suppression,
- adaptive fee and mempool pressure controls,
- graceful degradation of non-critical API methods.

## 7. Host Hardening Baseline

- minimal and patched OS baseline,
- measured boot where available,
- seccomp plus AppArmor/SELinux confinement,
- capability dropping and cgroup quotas,
- conservative socket/conntrack limits,
- optional XDP/eBPF early-drop for high-volume abuse.

## 8. Key and Identity Segmentation

Key classes remain role-separated:

- validator consensus keys,
- transport/session keys,
- node identity keys,
- operator authentication keys,
- artifact/release signing keys.

Cross-class key reuse is prohibited.

## 9. Evidence and Readiness

No network-security readiness claim is valid without retained evidence:

- profile compatibility matrix,
- downgrade rejection telemetry,
- handshake/replay incident metrics,
- saturation and reject-reason reports,
- key-rotation and attestation logs,
- attack simulation and recovery artifacts.

## 10. Residual Risk Statement

AOXChain does not claim absolute security. Residual risk persists in third-party dependencies, infrastructure operation, undiscovered defects, and evolving cryptanalysis. The architecture reduces exposure through segmentation, fail-closed policy, and evidence-governed operation.
