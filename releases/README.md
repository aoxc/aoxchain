# Releases Surface (Repository-Governed)

This directory is the canonical release publication surface for AOXChain.

It is intentionally split into:

1. **Permanent repository-level assets** (`releases/` root)
2. **Version-scoped release payloads** (`releases/v<workspace-version>/`)

---

## 1) Permanent Assets (`releases/` root)

These files/directories are long-lived and version-agnostic:

- `README.md` (this contract)
- `RELEASE_TEMPLATE.json` (manifest shape template)
- `CERTIFICATE_TEMPLATE.json` (release certificate shape template)
- `SBOM_TEMPLATE.spdx.json` (SPDX document template)
- `PROVENANCE_TEMPLATE.intoto.jsonl` (in-toto provenance template)
- `signing/` (repository release signing material and policies)

These assets define format contracts and process policy, not a specific shipped version.

---

## 2) Versioned Assets (`releases/v<workspace-version>/`)

Each published version must have exactly one version directory, for example:

- `releases/v0.2.0-alpha.1/`

A prepared version directory contains:

- `README.md`
- `manifest.json`
- `checksums.sha256`
- `compatibility.toml`
- `sbom.spdx.json`
- `provenance.intoto.jsonl`
- `binaries/`
  - `README.md`
  - `<target-label>/...` binary files
- `signatures/`
  - `README.md`
  - `*.sig` detached signatures (publication-time material)
- `certificates/`
  - `README.md`
  - `release-certificate.json`

---

## Operational Rules

- Prepare with `make repo-release-prepare`.
- Validate with `make repo-release-validate`.
- For one-command preparation + validation + summary output, use `make repo-release-full`.
- You may scope release artifacts by binary and network:
  - `make repo-release-full RELEASE_BINARIES="aoxc" REPO_RELEASE_NETWORK=testnet`
  - `make repo-release-full RELEASE_BINARIES="aoxc aoxchub aoxckit" REPO_RELEASE_NETWORK=mainnet`
- Do not mutate a published version after release finalization.
- If fixes are required after publication, publish a new version directory.
- Always verify:
  1. signature status,
  2. checksum integrity,
  3. compatibility constraints,
  4. runtime `aoxc version` parity with manifest.

---

## Governance Notes

- Release metadata must remain aligned with:
  - `Cargo.toml` workspace version
  - `configs/version-policy.toml`
  - `configs/registry/network-registry.toml`
- Compatibility declarations must be explicit and auditable.
- Generated release surfaces are engineering artifacts, not placeholder documentation.
