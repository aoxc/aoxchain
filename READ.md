# AOXChain Developer and Operator Readbook

This document is the practical companion to `README.md`.
It focuses on:

- developers building AOXChain locally,
- operators bootstrapping deterministic nodes,
- reviewers validating that the repository behaves like a real chain project.

---

## 1. Chain identity summary

AOXChain should be treated as:

- an **experimental sovereign chain**,
- with a **constitutional local core**,
- and **remote execution domains** attached through policy and settlement.

The shortest architecture commands are:

```bash
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- sovereign-core
cargo run -p aoxcmd -- module-architecture
```

---

## 2. Canonical address / key derivation rule

AOXC HD derivation format:

```text
m/44/2626/<chain>/<role>/<zone>/<index>
```

Examples:

```text
m/44/2626/1/1/1/0
m/44/2626/1001/2/4/7
```

This repo uses `2626` as the AOXC coin-type namespace.

Do not document or implement alternate local derivation roots unless there is an explicit migration plan.

---

## 3. Clean local development flow

### Step 1 — format, check, test

```bash
make fmt
make check
make test
```

### Step 2 — lint and quality gates

```bash
make clippy
make quality-quick
```

### Step 3 — release-style validation

```bash
make quality-release
make package-bin
```

---

## 4. Build metadata and release policy

Inspect build identity:

```bash
make version
make manifest
make policy
```

Direct CLI equivalents:

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- build-manifest
cargo run -p aoxcmd -- node-connection-policy
```

These outputs are important because they expose:

- version,
- commit,
- dirty/clean state,
- build profile,
- release channel,
- attestation hash,
- embedded certificate fingerprint status.

---

## 5. Deterministic node bootstrap

```bash
export AOXC_HOME="$PWD/.aoxc-devhome"
umask 077
mkdir -p "$AOXC_HOME"
```

### Key material

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --home "$AOXC_HOME" \
  --profile testnet \
  --name validator-01 \
  --password 'TEST#Secure2026!'
```

### Genesis

```bash
cargo run -p aoxcmd -- genesis-init \
  --home "$AOXC_HOME" \
  --chain-num 1001 \
  --block-time 6 \
  --treasury 1000000000000
```

### Node bootstrap

```bash
cargo run -p aoxcmd -- node-bootstrap --home "$AOXC_HOME"
```

### Single block

```bash
cargo run -p aoxcmd -- produce-once --home "$AOXC_HOME" --tx 'hello-aoxc'
```

### Short run

```bash
cargo run -p aoxcmd -- node-run \
  --home "$AOXC_HOME" \
  --rounds 15 \
  --sleep-ms 1000 \
  --tx-prefix AOXC_DEV
```

---

## 6. Make targets for developers

### Fast day-to-day

```bash
make help
make fmt
make check
make test
make clippy
```

### Packaging

```bash
make build-release
make package-bin
```

### Chain loop

```bash
make real-chain-run-once
make real-chain-run
make real-chain-tail
make real-chain-health
```

---

## 7. What should be improved next for ~75% readiness?

If the target is “closer to 75%”, I would add these next:

### A. Test coverage

1. multi-node deterministic consensus simulation
2. replay fixtures for settlement and receipts
3. message-envelope serialization/compatibility tests
4. policy tests for official-release-only peering
5. genesis/state migration tests

### B. Release and security tooling

1. cert issue / rotate / revoke commands
2. signed release manifest
3. SBOM generation
4. provenance attestation verification
5. peer handshake enforcement using attestation hash + cert fingerprint

### C. Operator UX

1. better `make help`
2. rich terminal dashboard
3. structured JSON logs + pretty logs together
4. summary command for runtime health
5. peer / cert / finality / settlement status panels

These additions would move AOXChain from “experimental but structured” toward “serious testnet candidate”.

---

## 8. Rule of thumb

When in doubt:

- keep the local chain small,
- keep the authority local,
- keep execution remote,
- and keep every critical decision auditable.
