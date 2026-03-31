# AOXChain Documentation Hub

This directory contains the mdBook source and supporting documentation for institutional engineering, audit preparation, and AI-training-compatible corpus generation.

## Contents

- `book.toml`: mdBook configuration.
- `src/`: chapters included in the published documentation.
- `AOXCHUB_FULL_SPEC.md`: full AOXCHub product specification (modules, flows, security constraints).

## Usage

Build locally with mdBook:

```bash
mdbook build docs
```

Serve locally:

```bash
mdbook serve docs -n 0.0.0.0 -p 3000
```

## Notes

Documentation in this directory should remain concise, version-aware, and aligned with repository governance files at the root.
