# AOXChain 10-Minute Operator Onboarding

## Goal

Get a new engineer or on-call operator to a safe local validation baseline in about 10 minutes.

## 0. Prerequisites

- Rust toolchain installed (`rustup`, `cargo`)
- Git
- A Unix-like shell
- Enough free disk for a full workspace build/test run

## 1. Clone and enter the workspace

```bash
git clone <repo-url> aoxchain
cd aoxchain
```

## 2. Confirm toolchain and formatting gates

```bash
cargo --version
rustc --version
cargo fmt --all --check
```

If `cargo fmt --all --check` fails, stop and normalize the working tree before doing anything else.

## 3. Run the mandatory local safety gates

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
```

These are the baseline CI-equivalent checks expected before touching operator-sensitive code.

## 4. Inspect the local chain model

```bash
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
```

Expected outcome:

- You understand the constitutional-core model.
- You can identify which crate owns which layer.
- You know whether a change belongs in protocol, networking, execution, or operations.

## 5. Read the minimum operator docs

Read these in order:

1. [`ONCALL_RUNBOOK.md`](./ONCALL_RUNBOOK.md)
2. [`MAINNET_READINESS_CHECKLIST.md`](./MAINNET_READINESS_CHECKLIST.md)
3. [`INCIDENT_RESPONSE_DRILL.md`](./INCIDENT_RESPONSE_DRILL.md)

## 6. Know the first escalation questions

Before changing or deploying anything, answer:

- Is this protocol, runtime, network, or operator-only scope?
- Does it affect deterministic state, consensus, or security posture?
- Does it need rollback guidance?
- Does it require new tests or updated runbook steps?

## 7. Definition of “onboarded enough”

A new operator is considered minimally onboarded when they can:

- run fmt/clippy/test successfully,
- identify the relevant crate for an incident,
- explain where consensus, P2P, and execution responsibilities live,
- follow the on-call runbook without improvising.
