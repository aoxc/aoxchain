# AOXChain Security Model

This document defines trust assumptions and failure domains for the current repository state.

## 1) Trust assumptions

1. **Validator trust boundary:** validators run kernel and runtime binaries correctly and protect signing keys.
2. **Deterministic execution assumption:** all consensus-relevant execution paths are deterministic for identical canonical inputs.
3. **Operator environment assumption:** operator tools (`aoxcmd`, `aoxchub`, `aoxckit`) are control-plane tools only, not consensus authority.
4. **Service boundary assumption:** networking/RPC/storage services can be byzantine or unavailable without redefining consensus rules.
5. **AI boundary assumption:** AI outputs from `aoxcai` are advisory and untrusted until validated by native deterministic controls.

## 2) Attacker classes

- **Remote network attacker:** attempts partitioning, eclipse behavior, gossip flooding, malformed protocol messages.
- **RPC-facing attacker:** sends malformed/high-rate requests to induce availability or parsing failures.
- **Compromised operator endpoint:** attempts unauthorized mutating control actions through CLI/desktop surfaces.
- **Key compromise attacker:** steals validator/operator keys and submits validly signed but malicious operations.
- **Storage adversary:** tampers with local data files/indexes/snapshots.
- **Supply-chain attacker:** introduces malicious dependency or build artifact drift.

## 3) Compromise scenarios

### Validator compromise
- **Impact:** highest; can affect consensus votes/finality safety depending on quorum share.
- **Current controls:** identity/key material and consensus safety modules are present in `aoxcore`/`aoxcunity`.
- **Gap:** full operational key-rotation incident runbooks and measurable RTO/RPO evidence are not yet centralized.

### Operator compromise
- **Impact:** medium-to-high; can mutate local runtime/service state but should not directly redefine kernel rules.
- **Current controls:** explicit operator crates and command surfaces.
- **Gap:** complete signed evidence for every control action path is partially implemented.

## 4) Network partition considerations

- Consensus and network are distinct failure domains; partitions should degrade liveness before safety.
- `aoxcnet` includes resilience/discovery/sync surfaces, but comprehensive partition-healing evidence remains partially implemented.
- Release blocking implication: partition recovery and fork convergence evidence should be required before production claims.

## 5) Storage corruption and recovery

- `aoxcdata` provides integrity-checked envelope/index patterns and error categories.
- Corruption must be detected as explicit integrity/recovery errors; silent fallback is unsafe.
- Snapshot/archive lifecycle exists as a service concern, but repository-wide deterministic recovery proof bundles are not yet fully evidenced.

## 6) Key compromise considerations

- Key material appears across identity and toolkit surfaces (`aoxcore::identity`, `aoxckit`, `aoxcmd::keys`).
- Key compromise must be modeled as expected incident class, not exceptional impossibility.
- Required operational policy: revoke/rotate/re-attest flows must remain externally auditable and scriptable via authoritative CLI surfaces.

## 7) Operator misuse and control-plane risks

- `aoxchub` is operator-facing only and must remain command-transparent.
- UI convenience must never bypass backend authorization and validation.
- Destructive operations must remain explicit, reviewable, and exportable as evidence.

## 8) UI/control-plane safety boundaries

1. UI is **outside** consensus trusted boundary.
2. UI actions must map to explicit backend APIs/commands.
3. Kernel state transitions must remain impossible from hidden UI-only code paths.
4. `aoxcmd` remains the authoritative operational shell unless governance formally changes that model.

## 9) Current-state honesty summary

- **Currently implemented:** explicit crate separation and many security-oriented modules.
- **Partially implemented:** full-system incident/recovery proofs and control-plane evidence coverage.
- **Target state:** formalized threat-driven release gates with reproducible evidence artifacts.
- **Not yet evidenced:** complete repository-level red-team/fault-injection result set tied to specific release tags.
