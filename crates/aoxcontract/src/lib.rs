//! Canonical contract manifest and identity domain model for AOXChain.
//!
//! # Scope
//! - Defines contract manifests, metadata, policy, compatibility, and artifact references.
//! - Provides canonical serialization and deterministic contract identity derivation.
//! - Performs validation for audit-grade domain objects.
//!
//! # Non-goals
//! - Not a runtime crate.
//! - Not a contract execution engine.
//! - Does not deploy, mutate state, perform RPC, or interact with consensus/network layers.

pub mod artifact;
pub mod canonical;
pub mod capability;
pub mod compatibility;
pub mod descriptor;
pub mod entrypoint;
pub mod error;
pub mod id;
pub mod manifest;
pub mod metadata;
pub mod policy;
pub mod validate;

pub use artifact::{
    ArtifactDigest, ArtifactDigestAlgorithm, ArtifactFormat, ArtifactLocationKind,
    ContractArtifactRef, Integrity, SourceTrustLevel,
};
pub use capability::ContractCapability;
pub use compatibility::{Compatibility, NetworkClass, RuntimeFamily};
pub use descriptor::ContractDescriptor;
pub use entrypoint::Entrypoint;
pub use error::{
    ArtifactValidationError, CanonicalizationError, CompatibilityError, ContractError,
    IdentityDerivationError, ManifestValidationError, PolicyValidationError,
};
pub use id::{CONTRACT_ID_DOMAIN_SEPARATOR, ContractId};
pub use manifest::{ContractManifest, ContractVersion, VmTarget};
pub use metadata::ContractMetadata;
pub use policy::ContractPolicy;
pub use validate::Validate;
