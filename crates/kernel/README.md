# Kernel Crates Layout

This directory hosts AOXChain's kernel-layer crates.

## Included crates

- `aoxcore` — canonical protocol and identity domain models, deterministic state/configuration primitives.
- `aoxcunity` — consensus/finality/safety engine and validator voting mechanics.

## Boundary intent

`crates/kernel/*` defines protocol truth and consensus-critical behavior.
Service, execution, and operations crates may depend on these crates, but must not redefine kernel truth.

## Change discipline

Changes under this directory are architecture-sensitive and should preserve:

- deterministic behavior,
- explicit validation boundaries,
- compatibility intent stated in release/change notes.
