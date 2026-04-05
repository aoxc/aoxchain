# Podman Operational Notes

This file defines the supported Podman workflow for AOXChain local runtime usage.

## Scope

- Local developer execution on Linux/macOS with Podman.
- Rootless-first operation.
- Parity with Docker commands where feasible.

## Prerequisites

- Podman 4.x+
- Podman Compose plugin (`podman compose`)
- Available host ports: `26656-26659`, `8545-8548`

## Build

```bash
podman build -t aoxchain-node:local .
```

## Single Node Run

```bash
podman run --rm \
  -p 26656:26656 \
  -p 8545:8545 \
  aoxchain-node:local
```

## Multi-Node Run (compose)

```bash
podman compose up --build
```

Stop and remove containers:

```bash
podman compose down
```

## Rootless Networking Notes

- Rootless Podman uses user-space networking; host port publishing remains explicit.
- If host firewall rules block access, open the mapped ports for local testing.
- Keep the default unprivileged runtime unless a specific operational requirement justifies elevation.

## Compatibility Contract

- `Dockerfile` remains the canonical build file for Docker and Podman.
- `docker-compose.yaml` remains the canonical multi-node topology for both `docker compose` and `podman compose`.
- Runtime behavior differences caused by container engine implementation must be treated as operational issues and documented with reproducible evidence.
