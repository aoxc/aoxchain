// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractArtifactView {
    pub location: String,
    pub format: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractPolicyView {
    pub allowed_vm_targets: Vec<String>,
    pub allowed_artifact_formats: Vec<String>,
    pub review_required: bool,
    pub signature_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractCompatibilityView {
    pub minimum_schema_version: u32,
    pub supported_schema_versions: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractReviewView {
    pub status: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractRuntimeBindingView {
    pub lane: String,
    pub execution_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractSummaryView {
    pub contract_id: String,
    pub status: String,
    pub package: String,
    pub vm_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractDetailView {
    pub summary: ContractSummaryView,
    pub artifact: ContractArtifactView,
    pub policy: ContractPolicyView,
    pub compatibility: ContractCompatibilityView,
    pub review: ContractReviewView,
    pub runtime_binding: Option<ContractRuntimeBindingView>,
}
