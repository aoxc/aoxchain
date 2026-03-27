# AOXC Models Directory — Production Readiness

Scope: `models/`

This directory stores YAML-based readiness and risk model manifests used for release
and operational decision support.

## Production Coverage

The directory is maintained for complete production compatibility across:

- Mainnet
- Testnet
- Devnet

## Files

- `network_profiles_v2.yaml` — canonical environment profiles and required evidence.
- `full_surface_readiness_matrix_v1.yaml` — cross-surface release readiness matrix.
- `mainnet_readiness_evidence_v1.yaml` — mainnet-focused readiness checkpoints.
- `validator-risk-v1.yaml` — validator/peer risk model manifest.
- `sample.yaml` — reference fallback model manifest template.

## Release Rules

A model set is considered 100% production-ready only when:

1. All three environments (mainnet/testnet/devnet) are explicitly represented.
2. Every required evidence path exists and is current for the target release.
3. Every blocking check is either green or formally exception-approved.
4. Ownership and verification commands are defined for every critical surface.

## Always Reading Order

1. `network_profiles_v2.yaml`
2. `full_surface_readiness_matrix_v1.yaml`
3. `mainnet_readiness_evidence_v1.yaml`
4. `validator-risk-v1.yaml`
5. `sample.yaml`
