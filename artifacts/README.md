# AOXC Artifacts — Advanced Production Readiness Bundle

This directory is the canonical, machine-auditable evidence bundle for AOXC releases.
It is designed for **always-on operational reading** and full compatibility validation across:

- **Mainnet**
- **Testnet**
- **Devnet**
- **Desktop interface / wallet rollout**

---

## 1) Purpose

The `artifacts/` bundle provides one source of truth for:

- release integrity (checksum, signature status, provenance)
- build and supply-chain attestations (manifest, SBOM)
- environment compatibility (mainnet/testnet/devnet)
- production closure evidence (runtime, telemetry, drills, audits)
- desktop interface compatibility and rollout readiness

---

## 2) Directory Structure

- `release-evidence/`
  - Build, provenance, SBOM, compatibility matrix, release notes, checksum/signature evidence.
- `network-production-closure/`
  - Runtime status, telemetry snapshots, production audit, security drill, soak plan,
    AOXHub rollout, desktop wallet compatibility, and alert rules.
- `index.json`
  - Machine-readable artifact index for CI/CD gates and governance reviews.

---

## 3) Mandatory Reading Order (Always)

1. `artifacts/index.json`
2. `artifacts/release-evidence/release-evidence-*.md`
3. `artifacts/release-evidence/build-manifest-*.json`
4. `artifacts/release-evidence/provenance-*.json`
5. `artifacts/release-evidence/sbom-*.json`
6. `artifacts/release-evidence/compat-matrix-*.json`
7. `artifacts/network-production-closure/runtime-status.json`
8. `artifacts/network-production-closure/telemetry-snapshot.json`
9. `artifacts/network-production-closure/desktop-wallet-compat.json`
10. `artifacts/network-production-closure/production-audit.json`
11. `artifacts/network-production-closure/security-drill.json`

---

## 4) Compatibility Completion Criteria (100%)

A release is considered **100% complete** only if all checks below are satisfied:

- Mainnet compatibility evidence present and marked pass.
- Testnet compatibility evidence present and marked pass.
- Devnet compatibility evidence present and marked pass.
- Desktop interface compatibility evidence present and marked pass.
- Provenance artifact present and verifiable.
- Checksum artifact present and matching built binary.
- Signature evidence present (`.sig` or `.sig.status`).
- Production closure package present and internally consistent.

---

## 5) Validation Checklist

- JSON artifacts parse without error.
- Required files listed in `index.json` all exist.
- Release evidence timestamp set is consistent across manifest/SBOM/provenance.
- Runtime and telemetry snapshots align with the release window.
- Desktop compatibility report references the same release target.

---

## 6) Recommended Automation Pipeline

### Pre-release
- Build + test + artifact generation (manifest, SBOM, provenance).

### Release gate
- Checksum + signature + provenance verification.
- Compatibility gate for mainnet/testnet/devnet + desktop.

### Post-release closure
- Runtime/telemetry collection.
- Security drill and production audit sign-off.
- Final bundle lock and retention.

---

## 7) Security Note

Artifacts are evidence surfaces, not standalone proof of ownership.
Final trust requires signature verification and provenance verification together.
