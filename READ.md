# AOXChain Canonical Technical Reference (v2)

This document is the system-level engineering contract for AOXChain. It defines what the system must do, what it must not do, and how correctness/readiness claims are validated.

## 1) System Intent

AOXChain is engineered as a deterministic protocol stack with:
- kernel-first consensus and settlement controls,
- AOXChain-owned deterministic VM execution,
- crypto-agile and post-quantum migration capability,
- operator-verifiable evidence for release posture.

## 2) Canonical Layers

### 2.1 Kernel and consensus layer
Responsibilities:
- transaction/block validity,
- finality policy and replay safety,
- cryptographic profile enforcement at consensus boundary.

### 2.2 VM and execution layer
Responsibilities:
- deterministic execution semantics,
- bounded resource accounting,
- deterministic cryptographic syscall behavior under active profile policy.

### 2.3 Service and transport layer
Responsibilities:
- p2p networking and peer/session controls,
- RPC/API surfaces,
- persistence and configuration materialization.

### 2.4 Operator and evidence layer
Responsibilities:
- lifecycle orchestration,
- quality and readiness command workflows,
- artifact generation for audits and closure decisions.

## 3) Non-Negotiable Invariants

1. **Determinism:** equal canonical inputs produce equal canonical outputs.
2. **Fail-closed validation:** unknown or malformed critical inputs are rejected before state transition.
3. **Boundary integrity:** non-kernel surfaces cannot override consensus truth.
4. **Crypto-profile explicitness:** every consensus-critical cryptographic behavior is profile-bound and versioned.
5. **Evidence traceability:** operational or readiness claims require reproducible command evidence.

## 4) VM Contract Principles

The AOXChain VM is a protocol surface, not a convenience runtime.

Mandatory properties:
- deterministic instruction behavior,
- explicit metering and bounded execution,
- admission checks for bytecode and syscalls,
- profile-aware cryptographic verification paths,
- no nondeterministic external dependencies in consensus-critical execution.

## 5) Post-Quantum Migration Contract

AOXChain must implement a staged migration strategy:
- profile versioning in consensus-visible structures,
- hybrid operation during migration windows,
- deterministic deprecation and rollback controls,
- artifact-backed verification of performance and safety impact.

Reference execution documents:
- `QUANTUM_ROADMAP.md`
- `QUANTUM_CHECKLIST.md`

## 6) Canonical Command Surface

Engineering baseline:

```bash
make build
make test
make quality
make audit
make os-compat-gate
make quantum-readiness-gate
```

Runtime lifecycle example:

```bash
make runtime-source-check AOXC_NETWORK_KIND=devnet
make runtime-install AOXC_NETWORK_KIND=devnet
make runtime-verify AOXC_NETWORK_KIND=devnet
make runtime-activate AOXC_NETWORK_KIND=devnet
```

## 7) Change Impact Rules

A change must be accompanied by documentation and validation updates when it touches:
- consensus/finality rules,
- VM semantics or gas behavior,
- cryptographic profiles and key handling,
- serialization/storage formats,
- operator controls and release gates.

## 8) License and Liability

AOXChain is distributed under the MIT License and provided **"as is"** without warranty or liability assumptions by maintainers or contributors, except where prohibited by law.

## 9) Protocol Naming Contract

For external and internal consistency, AOXChain naming should remain explicit and stable:

- **Ecosystem / chain name:** `AOXChain`
- **Protocol name:** `AOXC Constitutional Protocol`
- **Network IDs:** `AOXC-DEVNET`, `AOXC-TESTNET`, `AOXC-MAINNET`

Rationale:
- preserves constitutional governance and kernel-boundary semantics,
- keeps protocol branding concise for explorer/RPC/release surfaces,
- keeps network identity deterministic across genesis, runtime, and tooling.

## 10) Advanced Genesis Program (Operator Baseline)

AOXChain genesis hardening is treated as an operational security surface.

Mandatory controls before promotion:

1. deterministic serialization is enabled,
2. validator quorum policy is explicit and non-empty,
3. binding references (`validators`, `bootnodes`, `certificate`) are populated,
4. profile/environment alignment is verified,
5. testnet validator-account threshold is reviewed.

CLI surfaces:

```bash
aoxc genesis-template-advanced --profile testnet --out ./genesis.testnet.advanced.example.json
aoxc genesis-security-audit --profile testnet --genesis ./genesis.testnet.advanced.example.json
aoxc genesis-security-audit --profile testnet --genesis ./genesis.testnet.advanced.example.json --enforce
```

These commands provide a secure starting template and an enforceable security-audit gate for custom genesis workflows.
