# Advanced Node Role and Layer Blueprint

This document defines the full AOXChain multi-role node topology requested for future-ready deployments.

The blueprint is implementation-oriented: all roles can be defined immediately, while activation remains policy-gated by environment and validator governance.

---

## Design Goals

- preserve deterministic consensus safety and finality guarantees;
- isolate internet-facing services from validator key authority;
- support staged post-quantum migration without disruptive protocol rewrites;
- keep role expansion explicit, auditable, and reversible.

---

## Seven-Layer Runtime Model

1. **L0 Trust Root Layer**
   - hardware-backed key custody (HSM/TPM where available),
   - node attestation hooks,
   - immutable identity anchors.

2. **L1 Secure Transport Layer**
   - QUIC-based transport lanes,
   - mTLS role certificates,
   - replay-safe session management.

3. **L2 Identity and Policy Layer**
   - role-scoped node identity,
   - ACL/RBAC enforcement,
   - environment-level policy admission controls.

4. **L3 Data Availability and Gossip Layer**
   - block and state-chunk propagation,
   - bounded gossip admission,
   - anti-eclipse and peer-diversity constraints.

5. **L4 Deterministic Execution Layer**
   - deterministic state transition engine,
   - bounded metering,
   - replay equivalence enforcement.

6. **L5 Consensus and Finality Layer**
   - validator committee participation,
   - finality certificate handling,
   - slashing and evidence ingestion.

7. **L6 Service and Interop Layer**
   - RPC/API services,
   - index and archive surfaces,
   - bridge and oracle integrations.

---

## Full Role Set (Defined Now, Activated by Policy)

| Role ID | Canonical Name | Primary Responsibility | Default Activation |
|---|---|---|---|
| `core-val` | Consensus Validator | Vote, sign, and execute finality protocol | enabled in validator environments |
| `core-prop` | Block Proposer Scheduler | Proposal assembly and proposer rotation orchestration | disabled |
| `core-guard` | Consensus Rule Guard | Equivocation detection and evidence synthesis | disabled |
| `net-relay` | Relay Mesh Node | Peer relay and gossip fanout resilience | enabled in public networks |
| `net-gate` | Ingress Gateway | Transaction admission, QoS, anti-spam | enabled |
| `data-arch` | Archive Node | Full chain history and forensic replay data | disabled |
| `data-da` | Data Availability Node | Chunk persistence and availability responses | disabled |
| `serv-rpc` | RPC Service Node | External API and client connectivity surface | enabled |
| `serv-idx` | Index Service Node | Query index materialization and event lookup | disabled |
| `sec-sent` | Sentinel Security Node | Threat detection and operational anomaly signaling | disabled |
| `x-orcl` | Oracle Attestation Node | External data attestation ingress | disabled |
| `x-bridge` | Cross-Chain Bridge Node | External chain verification and settlement relay | disabled |

---

## Plane-Based Connectivity Model

AOXChain should maintain four network planes with strict role separation:

- **control plane** — lifecycle and policy orchestration;
- **consensus plane** — block proposal/vote/finality traffic;
- **data plane** — gossip, chunk, and replication movement;
- **service plane** — RPC and query traffic.

Each role must only communicate on explicitly allowed planes. All unspecified role-to-role traffic is denied by default.

---

## Quantum-Ready Consensus Hardening

AOXChain should implement post-quantum readiness as a staged capability, not a one-step replacement:

1. **Crypto agility baseline**
   - profile-selected signature suite identifiers,
   - explicit negotiation of supported cryptographic suites,
   - deterministic fail-closed downgrade handling.

2. **Hybrid signature transition**
   - combined classical + PQ signatures during migration windows,
   - certificate and block-header suite identifiers,
   - explicit validator policy gates for activation.

3. **Evidence-first safety posture**
   - slashing evidence retained for equivocation and invalid finality claims,
   - replay-resistant message domain separation,
   - governance-visible activation/rollback telemetry.

This approach aligns with repository quantum roadmap intent while preserving deterministic operation.

---

## Implementation Discipline

- define all role configurations and trust boundaries now;
- keep non-essential roles disabled until operational validation is complete;
- activate new roles only with validator policy and retained readiness evidence;
- never co-host validator signing authority with internet-facing RPC or bridge surfaces.

This document is normative for advanced role expansion planning and should be kept synchronized with environment profiles under `configs/`.

---

## AOXC-Q Consensus (AOXChain-Özel Tasarım)

Bu bölüm, klasik BFT davranışını AOXChain'e özgü bir karar hattıyla güçlendirir.

### Fazlar

1. **Q-Prepare**
   - proposer blok + state root + execution digest yayınlar,
   - komite üyeleri alan ayrılmış imza etiketleriyle (suite id) ön-oy üretir.

2. **Q-Lock**
   - `2f+1` eşiği görülmeden commit geçişi yasaktır,
   - lock sertifikası hem payload hem profil karmasına bağlanır.

3. **Q-Commit**
   - finality sertifikası canonical header içine gömülür,
   - geçerli sertifika olmadan yürütme katmanı blok final kabul etmez.

4. **Q-Seal**
   - canonical evidence hash zincire yazılır,
   - çapraz doğrulama düğümleri (guard/sentinel) hata kanıtı penceresinde gözlem yapar.

### AOXC-Q Özgün Güçlendirmeler

- **Dual-certificate mode:** lock ve commit sertifikaları ayrı tutulur; tek adım bypass engellenir.
- **Profile-bound finality:** finality sertifikası aktif kripto profil kimliğine bağlanır.
- **Deterministic evidence window:** her yükseklik için sabit kanıt toplama penceresi zorunludur.
- **Governance-gated crypto shift:** hibrit/PQC geçişleri yalnızca zincir üstü politika kapısıyla açılır.

---

## Rol Genişletme (AOXC-Q için ek roller)

Aşağıdaki roller tam aktivasyon öncesi tanımlanmalı, varsayılan durumda kapalı kalmalıdır:

| Role ID | Name | Responsibility |
|---|---|---|
| `core-qc` | Quantum Certificate Aggregator | Lock/commit sertifika birleştirme ve doğrulama |
| `core-qv` | PQ Verifier Node | Hibrit/PQC imza doğrulama hızlandırma yüzeyi |
| `sec-evd` | Evidence Custodian | Equivocation, invalid vote ve replay kanıt arşivi |
| `net-pacer` | Consensus Pacemaker | Deterministic round/timeout yönetimi |

Bu roller, çekirdek validator anahtarıyla asla aynı güvenlik alanında koşturulmamalıdır.
