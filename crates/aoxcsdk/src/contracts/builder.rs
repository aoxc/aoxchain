// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use thiserror::Error;

use aoxcontract::{
    ArtifactDigest, ArtifactFormat, ArtifactLocationKind, CapabilityProfile, Compatibility,
    ContractArtifactRef, ContractCapability, ContractClass, ContractDescriptor, ContractError,
    ContractManifest, ContractMetadata, ContractPolicy, ContractVersion, Entrypoint,
    ExecutionProfile, Integrity, NetworkClass, PolicyProfile, RuntimeFamily, SourceTrustLevel,
    Validate, VmTarget,
};

include!("builder_types.rs");
include!("builder_impl.rs");
include!("builder_tests.rs");
