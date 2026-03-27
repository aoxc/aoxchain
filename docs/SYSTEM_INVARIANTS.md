# AOXChain System Invariants

These invariants are release-critical engineering rules.

1. **Consensus determinism invariant**  
   For identical canonical inputs, kernel outputs (accept/reject, finality, state-transition outputs) must be deterministic.

2. **Kernel authority invariant**  
   Only kernel-defined consensus and deterministic runtime paths may change canonical chain state.

3. **Control-plane separation invariant**  
   Operator surfaces (`aoxcmd`, `aoxchub`, `aoxckit`) must remain outside consensus authority and explicitly auditable.

4. **UI non-authority invariant**  
   `aoxchub` must never become a hidden execution path for consensus-sensitive logic.

5. **Replay stability invariant**  
   Runtime execution (`aoxcexec`/`aoxcvm`) must remain replay-stable for canonical input envelopes and policy versions.

6. **Nondeterminism normalization invariant**  
   Non-deterministic ingress (network timing, RPC order, operator events, AI advisory data) must be normalized before deterministic execution.

7. **Malformed input rejection invariant**  
   Malformed or policy-invalid inputs must be rejected before canonical state mutation.

8. **Persistence integrity invariant**  
   Storage and snapshot layers must detect corruption explicitly and avoid silent acceptance.

9. **Key custody invariant**  
   Validator/operator key custody, rotation, and revocation workflows must remain explicit and auditable.

10. **Versioned policy invariant**  
    Consensus/runtime policy changes must be versioned and activation-scoped to preserve replay explainability.

11. **Evidence traceability invariant**  
    Release claims must be backed by reproducible command/test evidence tied to commit identity.

12. **Fail-closed authority invariant**  
    On uncertainty or validation failure in consensus-sensitive paths, behavior must fail closed rather than accept ambiguous state transitions.
