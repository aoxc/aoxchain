# AOXChain Execution Model

## 1) Execution flow (current repository model)

1. **Ingress:** transactions/proposals arrive via network/RPC/operator paths.
2. **Normalization:** ingress data is parsed and validated before deterministic execution eligibility.
3. **Consensus coupling:** `aoxcunity` drives ordering/finality decisions.
4. **State-transition execution:** deterministic execution logic is applied via kernel + runtime boundaries (`aoxcore`, `aoxcexec`, `aoxcvm`).
5. **Receipt/state materialization:** outputs are represented in canonical models (`aoxcore` receipts/state; persistence in `aoxcdata`).
6. **Service propagation:** resulting state/events are exposed by RPC/network services.

## 2) Lane model

- `aoxcvm` defines multi-lane execution routing surfaces (native/system, EVM, WASM, external compatibility lanes).
- `aoxcexec` provides deterministic lane policy and execution envelope/accounting primitives.
- Lane identity and policy must be explicit and replayable.

**Current state:** lane modules and policy structures are implemented.
**Gap:** full cross-lane replay conformance evidence is partially implemented.

## 3) Scheduling and orchestration roles

- `aoxcunity` is responsible for consensus progression and admissible ordering.
- `aoxcexec` and `aoxcvm` are responsible for deterministic execution orchestration within allowed lanes.
- Operator/UI tooling may request operations but must not act as hidden schedulers for canonical consensus state.

## 4) Replay expectations

For identical canonical inputs (ordered transactions, state root assumptions, lane policy version), execution outcomes must be replay-stable:

- identical acceptance/rejection decisions,
- identical receipt semantics,
- identical deterministic hash-relevant fields,
- explicit, stable failure categories.

## 5) Deterministic execution requirements

1. No wall-clock randomness in consensus-sensitive logic.
2. No unbounded nondeterministic host calls in lane-critical paths.
3. Canonical serialization and domain-separated hashing at boundary interfaces.
4. Policy/version pinning for execution lanes.
5. Deterministic rejection of malformed envelopes before mutation.

## 6) Malformed input handling expectations

- Reject malformed payloads early with explicit errors.
- Avoid partial mutation on failed validation.
- Preserve machine-auditable rejection reasons.
- Treat malformed input floods as availability concerns (service tier), not consensus rule changes.

## 7) Runtime failure boundaries

### Consensus-safe failure
- Lane execution failure yields deterministic rejection/receipt outcome.
- System continues with preserved safety invariants.

### Availability failure
- RPC/network/storage outage reduces liveness/operability.
- Consensus rules remain unchanged.

### Operator failure
- Misuse in control plane may impact local operations.
- Must not become undisclosed consensus mutation path.

## 8) Current-state honesty

- **Currently implemented:** execution lane routing modules, policy structures, host boundaries, canonical primitives.
- **Partially implemented:** complete end-to-end replay suite across all advertised lanes.
- **Target state:** release gate requiring deterministic replay corpus pass before tier-0/tier-1 releases.
- **Not yet evidenced:** formal proof artifact demonstrating cross-client/cross-platform lane determinism at scale.
