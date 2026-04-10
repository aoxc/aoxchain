# AOXChain Cross-Platform Compatibility Baseline

This document defines the repository-level compatibility contract for host operating systems and containerized execution.

It is an engineering baseline for review and operations, not a marketing claim. Production-readiness continues to require reproducible evidence.

## 1) Supported Execution Targets

| Target | Support posture | Notes |
|---|---|---|
| Linux (glibc/musl) | Primary | Canonical operator and CI surface. |
| NixOS | Supported (Linux profile) | Requires explicit package provisioning for toolchain/runtime dependencies. |
| macOS | Best effort | Build and operator flows are expected to work; service-management details differ from Linux. |
| Windows (PowerShell + Git Bash/MSYS2/WSL) | Best effort | GNU Make and bash-compatible shell are required for repository Make targets. |
| Docker / docker-compose | Primary for reproducible packaging and isolated flow rehearsal | Containerized flows are required for release rehearsal parity. |

## 2) Required Tooling Baseline

Minimum command surfaces expected across hosts:

- `bash`
- `make`
- `cargo` / Rust toolchain
- `git`
- `sha256sum` (or equivalent)
- `tar`

For Docker flows:

- `docker`
- `docker compose` (or `docker-compose`)

## 3) Compatibility Validation Surfaces

Use the following gates for platform and runtime contract checks:

```bash
make env-check
make docker-check
make os-compat-gate
```

For deeper production-grade rehearsal:

```bash
make production-full
make quantum-full
```

## 4) Target-Specific Operational Notes

### Linux / NixOS
- AOXC runtime root and profile materialization remain controlled via the Make runtime contract.
- NixOS operators should pin toolchain derivations and ensure shell compatibility for repository scripts.

### macOS
- Prefer GNU tooling where BSD variants differ in flags or behavior.
- Keep runtime path and permission assumptions explicit during operator rehearsals.

### Windows
- Use a bash-compatible execution surface (Git Bash, MSYS2, or WSL) for Make targets.
- Validate path translation and line-ending behavior for scripts and generated artifacts.

### Docker
- Treat container-based runs as reproducibility anchors for release and readiness rehearsal.
- Keep image/runtime manifests aligned with repository tag/version metadata.

## 5) Evidence Expectations

A compatibility readiness claim is valid only if:

1. gate command outputs are retained,
2. command environment (OS/tool versions) is recorded,
3. any platform-specific deviations are documented with remediation ownership.

Claims without retained evidence should be treated as non-authoritative.
