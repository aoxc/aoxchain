use serde::{Deserialize, Serialize};

use crate::{ArtifactValidationError, ContractError, Validate, VmTarget};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactDigestAlgorithm {
    Sha256,
    Sha3_256,
    Blake3,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArtifactDigest {
    pub algorithm: ArtifactDigestAlgorithm,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFormat {
    EvmBytecode,
    WasmModule,
    Archive,
    ManifestLinked,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactLocationKind {
    FilePath,
    Uri,
    ContentAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTrustLevel {
    Trusted,
    ReviewRequired,
    Untrusted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractArtifactRef {
    pub artifact_digest: ArtifactDigest,
    pub artifact_size: u64,
    pub artifact_format: ArtifactFormat,
    pub artifact_location_kind: ArtifactLocationKind,
    pub artifact_path_or_uri: String,
    pub compression: Option<String>,
    pub media_type: Option<String>,
    pub declared_vm_target: VmTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integrity {
    pub digest: ArtifactDigest,
    pub artifact_size: u64,
    pub artifact_format: ArtifactFormat,
    pub media_type: Option<String>,
    pub signature_required: bool,
    pub source_trust_level: SourceTrustLevel,
}

impl ContractArtifactRef {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        artifact_digest: ArtifactDigest,
        artifact_size: u64,
        artifact_format: ArtifactFormat,
        artifact_location_kind: ArtifactLocationKind,
        artifact_path_or_uri: impl Into<String>,
        compression: Option<String>,
        media_type: Option<String>,
        declared_vm_target: VmTarget,
    ) -> Result<Self, ContractError> {
        let artifact = Self {
            artifact_digest,
            artifact_size,
            artifact_format,
            artifact_location_kind,
            artifact_path_or_uri: artifact_path_or_uri.into(),
            compression,
            media_type,
            declared_vm_target,
        };
        artifact.validate()?;
        Ok(artifact)
    }
}

impl Validate for ContractArtifactRef {
    fn validate(&self) -> Result<(), ContractError> {
        let digest = self.artifact_digest.value.trim();
        if digest.is_empty() {
            return Err(ArtifactValidationError::MissingArtifactDigest.into());
        }
        let expected_digest_len = match self.artifact_digest.algorithm {
            ArtifactDigestAlgorithm::Sha256 | ArtifactDigestAlgorithm::Sha3_256 => 64,
            ArtifactDigestAlgorithm::Blake3 => 64,
        };
        if digest.len() != expected_digest_len || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ArtifactValidationError::MissingArtifactDigest.into());
        }
        if self.artifact_size == 0 {
            return Err(ArtifactValidationError::ZeroArtifactSize.into());
        }
        let location = self.artifact_path_or_uri.trim();
        if location.is_empty() {
            return Err(ArtifactValidationError::MissingArtifactLocation.into());
        }
        match self.artifact_location_kind {
            ArtifactLocationKind::Uri => {
                let has_scheme = location
                    .split_once("://")
                    .map(|(scheme, rest)| !scheme.is_empty() && !rest.is_empty())
                    .unwrap_or(false);
                if !has_scheme || location.chars().any(char::is_whitespace) {
                    return Err(ArtifactValidationError::MissingArtifactLocation.into());
                }
            }
            ArtifactLocationKind::FilePath => {
                if location.contains("://") || location.chars().any(char::is_control) {
                    return Err(ArtifactValidationError::MissingArtifactLocation.into());
                }
            }
            ArtifactLocationKind::ContentAddress => {
                let is_addressed = location
                    .split_once(':')
                    .map(|(prefix, value)| !prefix.is_empty() && !value.trim().is_empty())
                    .unwrap_or(false);
                if !is_addressed || location.chars().any(char::is_whitespace) {
                    return Err(ArtifactValidationError::MissingArtifactLocation.into());
                }
            }
        }
        if let Some(media_type) = &self.media_type {
            let valid = matches!(
                (&self.artifact_format, media_type.as_str()),
                (ArtifactFormat::EvmBytecode, "application/octet-stream")
                    | (ArtifactFormat::WasmModule, "application/wasm")
                    | (ArtifactFormat::Archive, "application/vnd.aox.archive")
                    | (ArtifactFormat::ManifestLinked, "application/json")
            );
            if !valid {
                return Err(ArtifactValidationError::MediaTypeFormatMismatch.into());
            }
        }
        Ok(())
    }
}

impl Validate for Integrity {
    fn validate(&self) -> Result<(), ContractError> {
        let digest = self.digest.value.trim();
        if digest.is_empty() {
            return Err(ArtifactValidationError::MissingArtifactDigest.into());
        }
        let expected_digest_len = match self.digest.algorithm {
            ArtifactDigestAlgorithm::Sha256 | ArtifactDigestAlgorithm::Sha3_256 => 64,
            ArtifactDigestAlgorithm::Blake3 => 64,
        };
        if digest.len() != expected_digest_len || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ArtifactValidationError::MissingArtifactDigest.into());
        }
        if self.artifact_size == 0 {
            return Err(ArtifactValidationError::ZeroArtifactSize.into());
        }
        if let Some(media_type) = &self.media_type {
            let valid = matches!(
                (&self.artifact_format, media_type.as_str()),
                (ArtifactFormat::EvmBytecode, "application/octet-stream")
                    | (ArtifactFormat::WasmModule, "application/wasm")
                    | (ArtifactFormat::Archive, "application/vnd.aox.archive")
                    | (ArtifactFormat::ManifestLinked, "application/json")
            );
            if !valid {
                return Err(ArtifactValidationError::MediaTypeFormatMismatch.into());
            }
        }
        Ok(())
    }
}
