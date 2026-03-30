# AOXChain

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="180" />
</p>

AOXChain is a modular Layer-1 engineering workspace focused on deterministic execution, auditable operations, and production-grade tooling.

> **Project status:** Experimental and under active development. Interfaces, behavior, and operational procedures can change at any time.

## Start here

- Canonical system definition: [READ.md](./READ.md)
- Architecture map: [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
- Security model: [docs/SECURITY_MODEL.md](./docs/SECURITY_MODEL.md)
- Licensing policy: [docs/LICENSING.md](./docs/LICENSING.md)
- Experimental + liability disclaimer: [docs/EXPERIMENTAL_NOTICE.md](./docs/EXPERIMENTAL_NOTICE.md)

## Repository map

| Scope | Purpose | Entry doc |
|---|---|---|
| `crates/` | Rust workspace crates for protocol, runtime, network, RPC, SDK, and tooling | [crates/README.md](./crates/README.md) |
| `configs/` | Environment profiles and deterministic network materials | [configs/README.md](./configs/README.md) |
| `docs/` | Architecture, security, state, execution, and operations documentation | [docs/src/READ.md](./docs/src/READ.md) |
| `scripts/` | Automation for local runs, validation, release evidence, and operations | [scripts/READ.md](./scripts/READ.md) |
| `tests/` | Integration and readiness validation workspace | [tests/READ.md](./tests/READ.md) |
| `models/` | Readiness, risk, and network profile model artifacts | [models/README.md](./models/README.md) |
| `contracts/` | System contract references and deployment matrices | [contracts/README.md](./contracts/README.md) |
| `artifacts/` | Generated release and production-closure evidence snapshots | [artifacts/README.md](./artifacts/README.md) |

## Core subsystem quick links

- Consensus and safety kernel: [crates/aoxcunity/READ.md](./crates/aoxcunity/READ.md)
- Core types and state primitives: [crates/aoxcore/READ.md](./crates/aoxcore/READ.md)
- Runtime and operator CLI: [crates/aoxcmd/READ.md](./crates/aoxcmd/READ.md)
- Multi-VM execution: [crates/aoxcvm/READ.md](./crates/aoxcvm/READ.md)
- Networking and resilience: [crates/aoxcnet/READ.md](./crates/aoxcnet/READ.md)
- RPC/API services: [crates/aoxcrpc/READ.md](./crates/aoxcrpc/READ.md)
- Desktop control plane: [crates/aoxchub/README.md](./crates/aoxchub/README.md)

## License

This repository is licensed under the [MIT License](./LICENSE).

## Important notice

This repository and its documents are provided for engineering and educational information. They do not provide legal, financial, or operational guarantees, and they do not create liability assumptions by maintainers or contributors.
