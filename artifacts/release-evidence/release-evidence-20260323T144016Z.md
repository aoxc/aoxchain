# AOXC Release Evidence 20260323T144016Z

- reproducible_build: cargo fmt --all --check && cargo test -p aoxcmd && cargo build --release -p aoxcmd --bin aoxc
- artifact_checksum: aoxc-20260323T144016Z.sha256
- artifact_signature: aoxc-20260323T144016Z.sig or .sig.status
- provenance_attestation: provenance-20260323T144016Z.json
- compatibility_matrix: compat-matrix-20260323T144016Z.json
- production_audit: production-audit-20260323T144016Z.json
- enforcement_rule: release is blocked if signature or provenance remains missing
