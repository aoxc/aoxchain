# AOXChain Relay-Chain Mainnet Blueprint

## 1. Positioning

AOXChain is architected as an interoperability-centric relay chain. The protocol intent is to coordinate heterogeneous chains and contract ecosystems under deterministic, auditable control flow.

This implies a product strategy where:
- cross-chain compatibility is a first-class property,
- deterministic settlement semantics outrank raw peak throughput,
- and operational transparency is mandatory for production readiness.

## 2. Current Implemented Operator Surface

The `aoxcmd` CLI currently exposes an executable deterministic path:

1. `vision`
2. `genesis-init`
3. `key-bootstrap`
4. `node-bootstrap`
5. `produce-once`
6. `network-smoke`

These commands provide a concrete bootstrap-to-block lifecycle for single-node deterministic verification.

## 3. Mainnet-Oriented Capability Matrix

### 3.1 Identity and key lifecycle

- Key generation and encrypted keyfile persistence are available.
- Certificate and passport artifacts are integrated into bootstrap flow.
- Actor identity model is deterministic and format-validated.

### 3.2 Consensus envelope

- Explicit validator rotation and quorum instantiation.
- Local proposal, vote admission, and finalization attempt path.
- Fork-choice insertion and block archival persistence.

### 3.3 Interop and execution lanes

- Multi-lane model exists at architecture level (`aoxcvm`).
- Full production cross-chain proof adapters remain in staged development.

## 4. Gaps to Close Before Mainnet

1. **Network transport finalization**
   - Replace gossip stub behavior with real peer transport and message queues.
2. **Distributed consensus validation**
   - Add multi-node simulation and adversarial scenarios.
3. **Persistence and replay guarantees**
   - Introduce deterministic state snapshotting and replay checks.
4. **Security program maturity**
   - Formal threat model, continuous fuzzing, external audit, incident runbooks.
5. **Release engineering**
   - Reproducible build attestations and signed release manifests.

## 5. Recommended Delivery Phases

### Phase A — Deterministic single-node hardening
- Expand CLI tests and property checks.
- Standardize structured error contracts and telemetry fields.

### Phase B — Multi-node interoperability testnet
- Integrate transport-backed gossip.
- Introduce end-to-end consensus + network integration tests.

### Phase C — Mainnet candidate
- Freeze consensus-critical APIs.
- Complete external audit closure.
- Publish reproducible release artifacts and operational SLO gates.

## 6. Operational Principle

Every new mainnet claim must be tied to:
- reproducible command path,
- deterministic test artifact,
- and explicit risk acceptance criteria.
