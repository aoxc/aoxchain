# AOXChain P2P Security & Audit Guide (EN)

This document defines how to use and audit the AOXChain secure p2p shell.

## 1. Security model

AOXChain networking exposes three security modes:

- `Insecure`: local testing only.
- `MutualAuth`: certificate validation + session gating.
- `AuditStrict`: production profile with strict admission policy.

## 2. Peer identity and certificates

Each peer contains:

- `id`
- `address`
- `NodeCertificate { subject, issuer, valid_from_unix, valid_until_unix, serial }`
- certificate fingerprint (`SHA-256`)

Admission in secure modes rejects certificates that are expired or not yet valid.

## 3. Session establishment flow

1. Register peer.
2. Validate certificate.
3. Establish session ticket.
4. Allow secure broadcasts only for peers with active session tickets.

This flow blocks anonymous message injection in secure modes.

## 4. CLI/operator integration

`aoxcmd network-smoke` is backward-compatible and demonstrates gossip plumbing.
For strict secure path testing, use crate-level tests in `aoxcnet::p2p`.

## 5. Audit checklist

- [ ] `SecurityMode::AuditStrict` set for production nodes.
- [ ] Certificate validity windows monitored.
- [ ] Session admission/revocation telemetry exported.
- [ ] Replay and partition simulation tests in CI.
- [ ] Incident response runbook includes cert rotation and peer quarantine.

## 6. Next hardening steps

- Replace in-memory queue with QUIC/TLS transport.
- Integrate CA trust store and CRL/OCSP strategy.
- Add message signing and anti-replay nonce persistence.
- Add Byzantine test scenarios (drop/reorder/equivocation).
