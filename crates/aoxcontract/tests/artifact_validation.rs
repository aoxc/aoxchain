// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

mod common;

use aoxcontract::{ArtifactValidationError, ContractError, Validate};

#[test]
fn non_hex_digest_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.artifact.artifact_digest.value = "not_hex_digest".into();

    let err = manifest.artifact.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Artifact(ArtifactValidationError::MissingArtifactDigest)
    ));
}

#[test]
fn uri_location_requires_scheme() {
    let mut manifest = common::sample_manifest();
    manifest.artifact.artifact_path_or_uri = "artifact.wasm".into();

    let err = manifest.artifact.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Artifact(ArtifactValidationError::MissingArtifactLocation)
    ));
}

#[test]
fn integrity_media_type_mismatch_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.integrity.media_type = Some("application/json".into());

    let err = manifest.integrity.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Artifact(ArtifactValidationError::MediaTypeFormatMismatch)
    ));
}
