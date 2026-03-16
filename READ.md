# AOXChain Audit-Grade Operational Readbook

This document is a **clean, chronological, audit-oriented runbook** for operating AOXChain.
It is written for operators, reviewers, and security teams who need a deterministic sequence from environment preparation to post-run evidence collection.

---

## 0) Scope and Intent

This readbook defines a minimal, reliable flow for:

1. Preparing a controlled environment.
2. Building and validating binaries.
3. Creating operator identity material.
4. Initializing deterministic genesis.
5. Bootstrapping node state.
6. Running controlled production loops.
7. Capturing verifiable audit evidence.

Use this file as the **primary execution order**. Use other docs only for deep-dive references.

---

## 1) Security Preconditions (Before Any Command)

- Use a dedicated host or isolated CI runner.
- Ensure shell history handling is compliant with your secret policy.
- Never commit private keys, passwords, or generated runtime data.
- Keep a timestamped command log for audit trails.
- For mainnet-sensitive actions, require multi-party review approval.

Recommended environment isolation:

```bash
export AOXC_HOME="$PWD/.aoxc-audit-home"
umask 077
mkdir -p "$AOXC_HOME"
```

---

## 2) Build and Integrity Gate

Run these commands in order and do not continue if any step fails.

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

Optional release-style packaging:

```bash
make quality-quick
make package-bin
```

Audit note: archive stdout/stderr of these commands as build evidence.

---

## 3) Identity Bootstrap (Wallet-Like Operator Material)

Create cryptographic identity material for the node operator.

### Testnet profile (recommended first)

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --home "$AOXC_HOME" \
  --profile testnet \
  --name validator-01 \
  --password "TEST#Secure2026!"
```

### Mainnet profile (explicit safety gate)

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --home "$AOXC_HOME" \
  --profile mainnet \
  --allow-mainnet \
  --name validator-01 \
  --password "AOXc#Mainnet2026!"
```

Alternative policy switch:

```bash
AOXC_ALLOW_MAINNET_KEYS=true cargo run -p aoxcmd -- key-bootstrap \
  --home "$AOXC_HOME" \
  --profile mainnet \
  --name validator-01 \
  --password "AOXc#Mainnet2026!"
```

---

## 4) Deterministic Genesis Initialization

Initialize a reproducible genesis configuration.

```bash
cargo run -p aoxcmd -- genesis-init \
  --home "$AOXC_HOME" \
  --chain-num 1001 \
  --block-time 6 \
  --treasury 1000000000000
```

Expected outcome:

- Genesis file is written under `$AOXC_HOME/identity/genesis.json` (unless `--path` is provided).
- Output includes deterministic fields such as `chain_id`, `total_supply`, and `state_hash`.

Audit note: persist the resulting `state_hash` into your change/control ticket.

---

## 5) Node Bootstrap and State Activation

Bootstrap the local node runtime from generated identity + genesis.

```bash
cargo run -p aoxcmd -- node-bootstrap --home "$AOXC_HOME"
```

This step validates readiness of core runtime components (mempool, validators, and quorum-visible state).

---

## 6) Controlled Block Production Sequence

### 6.1 One-block deterministic smoke

```bash
cargo run -p aoxcmd -- produce-once --home "$AOXC_HOME" --tx "boot-sequence-1"
```

### 6.2 Short bounded run

```bash
cargo run -p aoxcmd -- node-run \
  --home "$AOXC_HOME" \
  --rounds 20 \
  --sleep-ms 1000 \
  --tx-prefix AOXC_RUN
```

### 6.3 Network probe sequence

```bash
cargo run -p aoxcmd -- real-network \
  --home "$AOXC_HOME" \
  --rounds 10 \
  --timeout-ms 3000 \
  --pause-ms 250 \
  --bind-host 127.0.0.1 \
  --port 0
```

---

## 7) Operational Status and Release Gates

Capture runtime posture:

```bash
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
```

Check interoperability readiness:

```bash
cargo run -p aoxcmd -- interop-readiness
```

Enforce explicit release gate:

```bash
cargo run -p aoxcmd -- interop-gate \
  --audit-complete true \
  --fuzz-complete true \
  --replay-complete true \
  --finality-matrix-complete true \
  --slo-complete true \
  --enforce
```

---

## 8) Audit Evidence Checklist (Mandatory)

For each execution window, archive the following artifacts:

1. Build evidence (`fmt/check/test` command outputs).
2. Key bootstrap summary output (without exposing secrets).
3. Genesis output JSON including `state_hash`.
4. Node bootstrap output.
5. Produce-once output (`height`, `hash`, `finalized`).
6. Node-run and real-network outputs.
7. Runtime-status and interop-gate outputs.

Store these with:

- UTC timestamp,
- operator identity,
- git commit hash,
- environment fingerprint (OS/toolchain).

---

## 9) Failure Handling Protocol

If any command fails:

1. Stop the sequence immediately.
2. Record failing command + full stderr/stdout.
3. Classify failure as config, dependency, runtime, or policy gate.
4. Apply fix in a new tracked change.
5. Re-run from the **nearest safe checkpoint** (typically Section 2 or 4).

Never skip failed gates in production workflows.

---

## 10) Minimal Daily Command Set

For routine operator confidence checks:

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace minimal --tps 5.0 --peers 3 --error-rate 0.0
```

---

## 11) Reference Links

- Primary repository overview: `README.md`
- Crate-level map: `crates/README.md`
- Security and risk posture: `docs/SECURITY_AND_RISK_NOTICE_TR.md`
- Interoperability and key controls: `docs/KEY_TYPES_AND_INTEROP_GUIDE_EN.md`
- Audit readiness operations: `docs/AUDIT_READINESS_AND_OPERATIONS.md`

---

## Final Note

This readbook is intentionally strict, chronological, and audit-friendly.
If you keep evidence at each stage and enforce gate discipline, AOXChain operations remain reproducible, reviewable, and safer for production progression.
