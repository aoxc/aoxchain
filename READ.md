# AOXChain Canonical Technical Reference

## 1) Purpose and Operational Intent

AOXChain is a modular Layer-1 engineering workspace designed to produce deterministic protocol behavior, operator-auditable runtime workflows, and evidence-backed release decisions.

This document defines the canonical technical contract of the repository at a system level:
- what the system is expected to do,
- how to operate it through stable repository surfaces,
- which controls protect determinism and safety,
- and which command paths are expected for engineering and operations.

---

## 2) Canonical System Layers

### 2.1 Consensus and State Kernel
**Primary crates:** `aoxcore`, `aoxcunity`  
**Responsibility:** canonical transaction/state semantics, block validity, quorum/finality behavior, and safety boundaries.

### 2.2 Execution and Economic Controls
**Primary crates:** `aoxcexec`, `aoxcvm`, `aoxcenergy`  
**Responsibility:** deterministic execution, lane behavior, metering, and execution-policy enforcement.

### 2.3 Service and Data Plane
**Primary crates:** `aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`  
**Responsibility:** networking, RPC/API surfaces, persistence plumbing, and configuration materialization.

### 2.4 Operator and Runtime Surface
**Primary crates and paths:** `aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`, `Makefile`  
**Responsibility:** runtime installation/verification, health diagnostics, lifecycle orchestration, and audit evidence generation.

---

## 3) Determinism, Safety, and Control Invariants

The following invariants are repository-critical:

1. **Deterministic execution:** equivalent canonical inputs must produce equivalent canonical outputs.
2. **Fail-closed validation:** invalid configuration, malformed payloads, checksum drift, or policy violations must halt progression.
3. **Boundary discipline:** non-kernel surfaces must not bypass consensus/state validation controls.
4. **Evidence traceability:** readiness assertions must be supported by reproducible checks and retained artifacts.
5. **Operational reproducibility:** runtime identity material must remain checksum-verifiable and profile-consistent.

---

## 4) Canonical Network Profiles and Runtime Source Material

AOXChain maintains profile-defined runtime bundles under:

```text
configs/environments/<network-kind>/
```

Canonical network kinds include `localnet`, `devnet`, `testnet`, `validation`, and `mainnet`.

A canonical runtime source bundle is expected to include, at minimum:
- `manifest.v1.json`
- `genesis.v1.json`
- `genesis.v1.sha256`
- `validators.json`
- `bootnodes.json`
- `certificate.json`
- `profile.toml`
- `release-policy.toml`

For `testnet`, operator-facing metadata is additionally maintained in:
- `network-metadata.json`

---

## 5) Command Reference (Current Make Surface)

The following command sets represent the canonical operating paths.

### 5.1 Repository orientation and baseline quality
```bash
make help
make build
make test
make check
make fmt
make clippy
make quality
```

### 5.2 Runtime source validation and activation (example: `devnet`)
```bash
make runtime-source-check AOXC_NETWORK_KIND=devnet
make runtime-install AOXC_NETWORK_KIND=devnet
make runtime-verify AOXC_NETWORK_KIND=devnet
make runtime-activate AOXC_NETWORK_KIND=devnet
make runtime-status AOXC_NETWORK_KIND=devnet
```

### 5.3 Operator lifecycle (single-runtime flow)
```bash
make ops-prepare
make ops-start
make ops-status
make ops-logs
make ops-stop
```

### 5.4 High-level guided workflows
```bash
make demo
make localnet
make devnet
make testnet
make doctor
make audit-chain
```

### 5.5 Testnet readiness gate (persistent flow)
Use the dedicated testnet gate before testnet-affecting rollout work:

```bash
make testnet-gate
```

The gate validates:
- required `configs/environments/testnet` artifacts,
- cross-environment identity/checksum consistency,
- runtime-source integrity via `runtime-source-check`,
- identity alignment of `network-metadata.json` with `manifest.v1.json`.

### 5.6 Release and evidence surfaces
```bash
make package-versioned-bin
make package-versioned-archive
make publish-release
make audit
make db-health
```

---

## 6) Identity and Key Derivation Contract

AOXChain uses a canonical BIP44-style path envelope:

```text
m/44/2626/<chain>/<role>/<zone>/<index>
```

Policy constraints:
- canonical persisted path components are numeric and unhardened,
- variable components are bounded to `0 ..= 0x7FFF_FFFF`,
- hardened handling is represented as derivation policy/projection behavior, not serialized marker text.

Deterministic derivation uses domain-separated hashing over:
- canonical purpose fields (`44`, `2626`),
- master seed material,
- canonical path fields (`chain`, `role`, `zone`, `index`).

Node key-bundle controls enforce:
- role/path consistency,
- algorithm and encoding validity,
- deterministic fingerprint integrity,
- explicit validation errors suitable for telemetry and audit processing.

---

## 7) Engineering Governance References

For authoritative repository governance and operational policy, use:
- `README.md` for orientation and quick-start execution,
- `SCOPE.md` for in-scope boundaries and compatibility posture,
- `ARCHITECTURE.md` for dependency and control-flow boundaries,
- `SECURITY.md` for vulnerability handling and disclosure expectations,
- `TESTING.md` for mandatory validation and evidence requirements.

---

## 8) License and Liability Context

AOXChain is distributed under the MIT License.

Repository materials are provided on an **"as is"** basis without warranties or implied liability by maintainers or contributors, except where such limitations are prohibited by applicable law.
