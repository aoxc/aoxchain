# AOXChain

AOXChain is a deterministic Layer-1 engineering program focused on policy-governed authority, cryptographic agility, and evidence-gated release discipline.

## Current Repository Posture

This repository has undergone a planning reset to remove ambiguous roadmap drafts and establish one canonical execution path.

Canonical root documents:

- `ROADMAP.md` — single program plan with phase gates and checklists,
- `READ.md` — technical contract and invariants,
- `ARCHITECTURE.md` — architecture baseline and trust boundaries,
- `SCOPE.md` — in-scope/out-of-scope and sensitive change classes,
- `TESTING.md` — mandatory validation and evidence policy,
- `SECURITY.md` — disclosure process, threat-priority classes, and security gates,
- `KEY_MANAGEMENT.md` — quantum-resistant wallet/node key model, lifecycle, and domain separation.

## Program Objective

Build and operate AOXChain as:

- classical-secure today,
- post-quantum-primary by governed transition,
- migration-safe by protocol design.

No readiness claim is valid without reproducible evidence.

## Operator and Developer Quick Start

### Build

```bash
cargo build -p aoxcmd --release
```

### Baseline gates

```bash
make build
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

## Architecture-Change Rule

If a change affects authority model, consensus boundaries, migration semantics, profile policy, replay/recovery rules, or compatibility behavior, update in the same change:

1. `ARCHITECTURE.md`
2. `ROADMAP.md`
3. `TESTING.md`

Implementation-only architecture drift is not allowed.

## License and Liability

AOXChain is distributed under the MIT License on an "as is" basis, without warranty or liability assumptions by maintainers or contributors except where restricted by applicable law.
