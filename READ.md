# AOXChain Canonical Technical Definition

## 1) Mission

AOXChain is a modular Layer-1 engineering platform for deterministic execution, auditable operations, and evidence-backed release governance.

## 2) Canonical architecture layers

1. **Consensus and state kernel** (`aoxcore`, `aoxcunity`)
   - Canonical state transition primitives, block semantics, safety/finality logic.
2. **Execution and economics** (`aoxcexec`, `aoxcvm`, `aoxcenergy`)
   - Lane-specific execution, deterministic metering, and host resource policies.
3. **Service layer** (`aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`)
   - Networking, API surfaces, persistence, and configuration control plane.
4. **Operator layer** (`aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`)
   - Runtime installation, readiness checks, lifecycle orchestration, evidence output.

## 3) Determinism and safety invariants

- **Determinism:** equivalent canonical inputs must produce equivalent canonical outputs.
- **Fail-closed policy:** invalid inputs/policies/configurations must block progression.
- **Traceability:** operational claims must map to reproducible tests or artifacts.
- **Boundary discipline:** non-kernel surfaces must not bypass consensus/state validation boundaries.

## 4) Installation and operation with Make

### 4.1 Engineering bootstrap
```bash
make help
make build
make test
make check
make clippy
```

### 4.2 Runtime bootstrap
```bash
make runtime-source-check AOXC_NETWORK_KIND=devnet
make runtime-install AOXC_NETWORK_KIND=devnet
make runtime-verify
make runtime-activate
make runtime-status
```

### 4.3 Continuous operations
```bash
make ops-prepare
make ops-start
make ops-status
make ops-logs
make ops-stop
```

### 4.4 Release and evidence
```bash
make package-versioned-bin
make package-versioned-archive
make audit
```

### 4.5 Full four-node local system bring-up
```bash
make aoxc-full-4nodes-plan
make aoxc-full-4nodes
make aoxc-full-4nodes-docker
```

## 5) Canonical key and identity model

### 5.1 HD path contract
AOXChain uses a canonical BIP44-style path contract:

`m/44/2626/<chain>/<role>/<zone>/<index>`

- `44` = BIP44 purpose field.
- `2626` = AOXC canonical coin-type field.
- `chain`, `role`, `zone`, `index` are validated as canonical variable components.

### 5.2 Hardened/unhardened policy
- Canonical serialized paths are persisted as unhardened numeric components.
- Each variable component is bounded to `0 ..= 0x7FFF_FFFF`.
- Hardened behavior is available via projection helpers (for downstream derivation policy), not by storing hardened markers in canonical path text.

### 5.3 Deterministic derivation policy
- Key material is derived through domain-separated hashing over:
  - AOXC key derivation domain,
  - BIP44 purpose (`44`),
  - AOXC purpose (`2626`),
  - master seed,
  - canonical `chain`, `role`, `zone`, `index` components.
- Role-scoped seeds and bundle fingerprints are derived through additional explicit domain separators.

### 5.4 Node key bundle control
Node key bundles enforce:
- required role presence,
- canonical algorithm/encoding constraints,
- deterministic fingerprint consistency,
- expected HD path matching per role/profile,
- explicit validation error codes suitable for telemetry and audit workflows.

## 6) Environment and release posture

Environment definitions in `configs/` cover localnet, devnet, testnet, mainnet, and deterministic templates. Mainnet progression is governed by readiness checks and closure evidence under `artifacts/`.

## 7) Governance references

- Repository navigation and quick start: `README.md`
- Scope and compatibility policy: `SCOPE.md`
- Component and data-flow boundaries: `ARCHITECTURE.md`
- Security reporting and posture: `SECURITY.md`
- Validation policy and required checks: `TESTING.md`

## 8) License and disclaimer

This project is distributed under MIT. All materials are provided "as is" without warranties or implied liability by maintainers or contributors.
