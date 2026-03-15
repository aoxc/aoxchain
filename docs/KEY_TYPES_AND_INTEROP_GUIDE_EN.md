# AOXChain Key Types and Interoperability Guide (EN)

This document summarizes production-focused key handling and cross-chain compatibility planning.

## 1) Key Types in AOXChain

### 1.1 Dilithium3 (Post-Quantum Signature)
- **Where:** `aoxcore::identity::pq_keys`
- **Purpose:** Actor identity signing/verification pipeline with post-quantum primitives.
- **Operational note:** Keep secret-key material encrypted at rest and never log raw key bytes.

### 1.2 Keyfile Envelope (Argon2id + AES-256-GCM)
- **Where:** `aoxcore::identity::keyfile`
- **Purpose:** Password-based encryption for secret-key persistence.
- **Default properties:**
  - KDF: Argon2id
  - Cipher: AES-256-GCM
  - Structured JSON envelope with versioning metadata

### 1.3 Node Key Artifacts (`aoxcmd key-bootstrap`)
Generated artifacts:
- `node.key` (encrypted secret + identity bundle)
- `node.cert.json` (signed certificate)
- `node.passport.json` (runtime identity passport)

On Unix-like systems, AOXChain persists these artifacts with restrictive file mode (`0600`).

## 1.4 Bootstrap Profiles for Test/Main Network Separation
- `aoxcmd key-bootstrap --profile mainnet`
  - defaults: `AOXC-MAIN`, `AOXC-ROOT-CA`, `AOXC_DATA/keys`
- `aoxcmd key-bootstrap --profile testnet`
  - defaults: `TEST-XXX-XX-LOCAL`, `TEST-XXX-ROOT-CA`, `TEST_DATA/keys`

This keeps test identities visibly separated from mainnet-like identifiers and reduces operator mix-up risk.

Mainnet profile requires explicit opt-in (`--allow-mainnet` or `AOXC_ALLOW_MAINNET_KEYS=true`) to prevent accidental production-key issuance on developer hosts.

## 2) Secure Password Examples

Accepted example pattern:
- `AOXc#Mainnet2026!`

Rejected examples:
- `weakpass` (no complexity)
- `NoSymbol12345` (missing symbol)
- `SHORT#1a` (too short)

## 3) Interoperability Quality Plan (EVM / WASM / Move / UTXO)

To improve chain compatibility quality beyond baseline implementation:

1. **Adapter conformance tests** per lane
   - EVM receipt/status parity
   - WASM host-call compatibility
   - Move object/state mapping invariants
   - UTXO witness and script context mapping

2. **Deterministic replay suites**
   - Record canonical transactions and re-execute across versions.

3. **Bridge failure-injection and fuzzing**
   - Invalid proofs, delayed finality, out-of-order message delivery.

4. **Finality assumption matrix**
   - Explicitly document reorg depth and confirmation policy per target chain.

5. **Audit-first release gates**
   - Require external audits for key lifecycle, signer domain separation, and bridge code.

## 3.1) Machine-Readable Interop Gate

`aoxcmd interop-gate` provides CI-friendly release readiness checks with pass/fail and missing controls:

- `--audit-complete <bool>`
- `--fuzz-complete <bool>`
- `--replay-complete <bool>`
- `--finality-matrix-complete <bool>`
- `--slo-complete <bool>`
- `--enforce` (returns non-zero on failure)

Example:

```bash
aoxc interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```

## 4) Recommended Production Path

- Stage A: single-lane hardened testnet (EVM first)
- Stage B: dual-lane pilot (EVM + WASM)
- Stage C: Move/UTXO adapters behind feature flags
- Stage D: controlled mainnet rollout with kill-switch and rollback playbooks
