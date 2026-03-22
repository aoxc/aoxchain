# AOXChain Incident Response Drill

## Objective

Practice a full incident loop before mainnet readiness: detect, classify, reproduce, mitigate, verify, and document.

## Recommended cadence

- Run monthly during active development.
- Run again before any mainnet-readiness review.

## Drill scenarios

### Scenario A — Consensus regression

Simulate a failing `aoxcunity` or `aoxcmd` test related to block production or finality.

Success criteria:

- incident is classified correctly,
- consensus-sensitive crates are isolated quickly,
- rollback/fix decision is documented.

### Scenario B — Mutual-auth / P2P regression

Simulate an `aoxcnet` failure involving handshake rejection, peer admission, or replay behavior.

Success criteria:

- operator treats security regressions as high-severity,
- secure-mode assumptions are checked before any workaround,
- targeted network tests are re-run and captured.

### Scenario C — Multi-lane gas inconsistency

Simulate a failing `aoxcvm` test involving gas accounting or lane state isolation.

Success criteria:

- gas/resource issue is isolated to execution scope unless broader safety impact is proven,
- comparative lane tests are run,
- blast radius is documented.

## Drill steps

1. Assign incident commander and scribe.
2. Start timer.
3. Present one scenario without advance warning.
4. Require the responder to:
   - classify severity,
   - name affected crates,
   - run the minimum checks,
   - propose mitigation,
   - define verification steps.
5. End with a written retrospective.

## Retrospective template

- What was detected first?
- Was severity classification correct?
- Were the right tests run first?
- Was rollback/fix reasoning sound?
- Which docs were missing or unclear?
- What should change before the next drill?
