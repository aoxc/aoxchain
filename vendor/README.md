# Vendor Dependencies

This directory contains third-party crates vendored into AOXChain for deterministic builds and controlled patching.

## Current contents

- `bip39/` — vendored `bip39` implementation patched into the workspace via `[patch.crates-io]`.

## Policy

- Treat vendored crates as upstream mirrors with minimal local divergence.
- Prefer workspace-level integration changes over modifying vendored internals.
- Any required local patch must be narrowly scoped and documented in PR context.
