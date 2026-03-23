# AOXChain SRE / On-Call Runbook

## Purpose

This runbook gives operators a first-response path for build, runtime, consensus, networking, and execution incidents.

## Severity model

- **SEV-1:** chain safety, consensus correctness, key compromise, or widespread node outage risk
- **SEV-2:** degraded networking, RPC instability, persistent test/regression failures, or execution-lane failures without confirmed chain-safety impact
- **SEV-3:** isolated node issues, documentation drift, minor CI breakage

## First 5 minutes

1. Freeze speculative changes.
2. Capture exact failing command/output.
3. Identify affected layer:
   - `aoxcmd` = operator/runtime orchestration
   - `aoxcunity` = consensus/finality
   - `aoxcnet` = P2P/discovery/gossip/sync
   - `aoxcvm` = execution lanes and gas/resource accounting
4. Run the minimum health suite:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
```

5. Open or update the incident log.

## Decision tree

### If consensus/finality looks wrong

- Treat as **SEV-1** until disproven.
- Inspect recent changes in `crates/aoxcunity/` and `crates/aoxcmd/src/node/`.
- Re-run consensus-targeted tests first:

```bash
cargo test -p aoxcunity
cargo test -p aoxcmd
```

### If P2P/mTLS/handshake behavior looks wrong

- Treat as **SEV-1** if it could admit unauthenticated peers.
- Otherwise treat as **SEV-2**.
- Re-run network-targeted tests:

```bash
cargo test -p aoxcnet
```

- Verify mutual-auth assumptions, certificate binding, and handshake/session expectations.

### If execution lanes or gas accounting look wrong

- Treat as **SEV-2** unless state corruption or consensus divergence is suspected.
- Re-run execution-targeted tests:

```bash
cargo test -p aoxcvm
```

- Compare gas/resource behavior across lanes.

## Operator rules

- Do not bypass fmt/clippy/test gates during incident response unless explicitly authorized for an emergency triage branch.
- Do not weaken mutual-auth, certificate validation, or replay defenses as a “temporary fix.”
- Do not modify consensus behavior without a test that demonstrates the failure and the fix.

## Evidence to capture

- commit hash
- failing command
- exact stderr/stdout
- impacted crate(s)
- whether reproduction is deterministic
- whether rollback is known and safe

## Exit criteria

An incident can move to monitoring-only when:

- the failing condition is reproducible and understood,
- a fix or rollback exists,
- `cargo test` passes on the fix branch,
- docs/runbooks are updated if operator behavior changed.
