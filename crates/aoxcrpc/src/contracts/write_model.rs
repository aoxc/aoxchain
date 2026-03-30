// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use aoxcontract::{ApprovalMarker, ContractManifest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSubmission {
    pub manifest: ContractManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRegistrationInput {
    pub manifest_submission: ManifestSubmission,
    pub review: Option<ApprovalMarker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLifecycleInput {
    pub contract_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBindingResolutionInput {
    pub manifest_submission: ManifestSubmission,
}
