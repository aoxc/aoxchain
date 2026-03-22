use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::contracts::read_model::{
    ContractDetailView, ContractRuntimeBindingView, ContractSummaryView,
};
use crate::contracts::write_model::{
    ContractLifecycleInput, ContractRegistrationInput, ManifestSubmission,
    RuntimeBindingResolutionInput,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ok,
    Accepted,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateManifestRequest {
    pub request_id: String,
    pub submission: ManifestSubmission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterContractRequest {
    pub request_id: String,
    pub input: ContractRegistrationInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetContractRequest {
    pub request_id: String,
    pub contract_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListContractsRequest {
    pub request_id: String,
    pub status: Option<String>,
    pub vm_target: Option<String>,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateContractRequest {
    pub request_id: String,
    pub input: ContractLifecycleInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecateContractRequest {
    pub request_id: String,
    pub input: ContractLifecycleInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeContractRequest {
    pub request_id: String,
    pub input: ContractLifecycleInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveRuntimeBindingRequest {
    pub request_id: String,
    pub input: RuntimeBindingResolutionInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope<T> {
    pub request_id: String,
    pub status: ResponseStatus,
    pub contract_id: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub receipt: Option<String>,
    pub data: T,
}

pub type ValidateManifestResponse = ResponseEnvelope<ContractDetailView>;
pub type RegisterContractResponse = ResponseEnvelope<ContractDetailView>;
pub type GetContractResponse = ResponseEnvelope<ContractDetailView>;
pub type ListContractsResponse = ResponseEnvelope<Vec<ContractSummaryView>>;
pub type ActivateContractResponse = ResponseEnvelope<ContractDetailView>;
pub type DeprecateContractResponse = ResponseEnvelope<ContractDetailView>;
pub type RevokeContractResponse = ResponseEnvelope<ContractDetailView>;
pub type ResolveRuntimeBindingResponse = ResponseEnvelope<ContractRuntimeBindingView>;
