# Podman Operational Notes

This document defines the supported Podman workflow for AOXChain local execution.

## 1. Scope

- local developer execution on Linux/macOS,
- rootless-first operation,
- parity with Docker workflows where feasible.

## 2. Prerequisites

- Podman 4.x or newer,
- Podman Compose plugin (`podman compose`),
- available host ports: `26656-26659`, `8545-8548`.

## 3. Image Build

```bash
podman build -t aoxchain-node:local .
```

## 4. Single-Node Runtime

```bash
podman run --rm \
  -p 26656:26656 \
  -p 8545:8545 \
  aoxchain-node:local
```

## 5. Multi-Node Runtime

```bash
podman compose up --build
```

Stop and remove containers:

```bash
podman compose down
```

## 6. Rootless Networking Notes

- rootless Podman uses user-space networking,
- published host ports remain explicit,
- keep unprivileged runtime by default unless a documented operational requirement justifies elevation.

## 7. Compatibility Contract

- `Dockerfile` is the canonical image build definition for Docker and Podman,
- `docker-compose.yaml` is the canonical multi-node topology for both engines,
- engine-specific runtime differences must be documented with reproducible evidence.
