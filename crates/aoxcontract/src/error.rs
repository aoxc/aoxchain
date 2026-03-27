use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error(transparent)]
    Manifest(#[from] ManifestValidationError),
    #[error(transparent)]
    Artifact(#[from] ArtifactValidationError),
    #[error(transparent)]
    Policy(#[from] PolicyValidationError),
    #[error(transparent)]
    Compatibility(#[from] CompatibilityError),
    #[error(transparent)]
    Canonicalization(#[from] CanonicalizationError),
    #[error(transparent)]
    Identity(#[from] IdentityDerivationError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestValidationError {
    #[error("contract name cannot be empty")]
    EmptyContractName,
    #[error("contract name contains invalid characters")]
    InvalidContractName,
    #[error("package cannot be empty")]
    EmptyPackage,
    #[error("version must be semantic-like and non-empty")]
    InvalidVersionFormat,
    #[error("schema version must be non-zero")]
    MissingSchemaVersion,
    #[error("duplicate entrypoint: {0}")]
    DuplicateEntrypoint(String),
    #[error("manifest must contain at least one entrypoint")]
    EmptyEntrypoints,
    #[error("declared vm target does not match artifact vm target")]
    VmTargetMismatch,
    #[error("manifest integrity digest does not match artifact digest")]
    DigestAlgorithmMismatch,
    #[error("manifest integrity does not match artifact metadata")]
    IntegrityMetadataMismatch,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArtifactValidationError {
    #[error("artifact digest cannot be empty")]
    MissingArtifactDigest,
    #[error("artifact size must be greater than zero")]
    ZeroArtifactSize,
    #[error("artifact size exceeds configured maximum")]
    ArtifactTooLarge,
    #[error("artifact path or uri cannot be empty")]
    MissingArtifactLocation,
    #[error("artifact media type is inconsistent with format")]
    MediaTypeFormatMismatch,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyValidationError {
    #[error("policy allows no vm targets")]
    EmptyAllowedVmTargets,
    #[error("policy allows no artifact formats")]
    EmptyAllowedArtifactFormats,
    #[error("policy max artifact size must be greater than zero")]
    InvalidArtifactSizeLimit,
    #[error("policy contains duplicate capability: {0}")]
    DuplicateCapability(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompatibilityError {
    #[error("minimum schema version must be non-zero")]
    MissingMinimumSchemaVersion,
    #[error("supported schema versions cannot be empty")]
    EmptySupportedSchemaVersions,
    #[error("supported runtime families cannot be empty")]
    EmptySupportedRuntimeFamilies,
    #[error("supported network classes cannot be empty")]
    EmptySupportedNetworkClasses,
    #[error("supported schema versions contain duplicates")]
    DuplicateSupportedSchemaVersions,
    #[error("minimum schema version must be listed in supported schema versions")]
    MinimumSchemaVersionNotSupported,
    #[error("manifest schema version is incompatible")]
    CompatibilityMismatch,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CanonicalizationError {
    #[error("canonical encoding failed")]
    CanonicalEncodingFailed,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentityDerivationError {
    #[error("identity derivation failed")]
    DerivationFailed,
}
