// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXChain canonical contract manifest, identity, and governance-adjacent domain model.
//!
//! # Purpose
//! This crate defines the canonical, deterministic, and validation-first domain model
//! for contract descriptions within AOXChain.
//!
//! It establishes the authoritative structures and rules for:
//! - contract manifests,
//! - artifact references and integrity metadata,
//! - compatibility constraints,
//! - policy declarations,
//! - review-facing status records,
//! - registry-facing contract records,
//! - runtime-binding descriptors, and
//! - deterministic contract identity derivation.
//!
//! # Design Objectives
//! This crate is designed to serve as a stable domain boundary with the following
//! properties:
//! - deterministic serialization for identity-sensitive objects,
//! - audit-grade validation for externally supplied and internally persisted records,
//! - explicit separation between descriptive contract data and execution concerns,
//! - long-term schema stability for registry, review, and runtime integration, and
//! - safe composition by higher-layer AOXChain subsystems.
//!
//! # Architectural Scope
//! This crate is the canonical source of truth for contract definition semantics.
//! It may be consumed by registry, operator, SDK, RPC, and runtime-adapter layers,
//! but it does not itself perform those responsibilities.
//!
//! Specifically, this crate:
//! - defines contract manifests, metadata, policy, compatibility, and artifact references,
//! - provides canonical serialization and deterministic contract identity derivation,
//! - defines registry-facing, review-facing, and runtime-binding-facing contract records, and
//! - performs validation for audit-grade domain objects.
//!
//! # Explicit Non-Goals
//! This crate is intentionally non-executable and non-transactional.
//!
//! It does not:
//! - execute contracts,
//! - deploy contracts,
//! - mutate chain state,
//! - persist records by itself,
//! - expose RPC handlers,
//! - perform network operations,
//! - or participate in consensus processing.
//!
//! Those responsibilities belong to higher-layer AOXChain subsystems such as state,
//! runtime, transport, operator, and API surfaces.
//!
//! # Security Posture
//! All identity-affecting and registry-relevant structures defined by this crate
//! must be treated as security-sensitive domain objects.
//!
//! Implementations consuming this crate are expected to:
//! - validate untrusted input before acceptance,
//! - preserve canonical encoding rules,
//! - avoid alternate identity derivation logic outside this crate, and
//! - treat policy and compatibility declarations as authoritative contract metadata.

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
pub mod registry_types;
pub mod review;
pub mod runtime_binding;
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
pub use registry_types::{
    ContractActivationMode, ContractRecordVersion, ContractStatus, RegisteredContract,
};
pub use review::{ApprovalMarker, ContractReviewStatus, ReviewRequirement};
pub use runtime_binding::{ExecutionProfileRef, LaneBinding, RuntimeBindingDescriptor};
pub use validate::Validate;
