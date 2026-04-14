# AOXC vendored `bip39`

This directory contains AOXChain's vendored copy of `bip39` `v2.2.2`,
used through a workspace-level `[patch.crates-io]` override.

## Why this exists

- Keep mnemonic parsing/encoding behavior stable inside the repository.
- Allow AOXC to review and control dependency posture for security gates.
- Avoid depending on external `rand` feature wiring for AOXC runtime paths.

## Scope

AOXC currently uses this crate for:

- mnemonic phrase parsing (`Mnemonic::parse_in`),
- entropy-to-phrase encoding (`Mnemonic::from_entropy_in`),
- deterministic restore/validation flows in `aoxcore`.

## Language support

`all-languages` is retained and includes:

- English (always enabled),
- Simplified Chinese,
- Traditional Chinese,
- Czech,
- French,
- Italian,
- Japanese,
- Korean,
- Portuguese,
- Spanish.

## Upstream provenance

Based on upstream `rust-bitcoin/rust-bip39` `v2.2.2`.
License remains CC0-1.0 (see `LICENSE`).
