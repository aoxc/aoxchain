# aoxcsdk

## Purpose

`aoxcsdk` provides the SDK-facing integration surface for applications and services that connect to AOXChain.

## Target Use Cases

- building chain clients,
- operational automation (CI/CD, health checks, validation workflows),
- typed integrations with AOXChain node and RPC layers.

## SDK Design Principles

1. **Deterministic, explicit behavior**: avoid hidden defaults.
2. **Typed error surface**: enable precise error classification for integrators.
3. **Security-oriented integration**: enforce clear boundaries around key and identity flows.
4. **Documented examples**: provide quick-start and verifiable usage patterns.

## Local Development

From repository root:

```bash
cargo check -p aoxcsdk
cargo test -p aoxcsdk
```

## Workspace Integrations

- Node command surface: [`../aoxcmd/README.md`](../aoxcmd/README.md)
- RPC layer: [`../aoxcrpc/README.md`](../aoxcrpc/README.md)
- Network security layer: [`../aoxcnet/README.md`](../aoxcnet/README.md)

## Production and Risk Note

The SDK alone is not a production security guarantee. Before going live, validate:
- independent security audits,
- threat model verification,
- operational runbook and rollback plan,
- version upgrade tests.
