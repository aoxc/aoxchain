//! Mappers between contract transport models and workspace domain models.

use chrono::Utc;

use aoxcontract::{ApprovalMarker, ContractDescriptor, ContractId, ContractManifest};
use aoxcore::contract::receipt::ContractReceipt;
use aoxcore::contract::record::OnChainContractRecord;

use crate::contracts::error::ContractRpcError;
use crate::contracts::read_model::{
    ContractArtifactView, ContractCompatibilityView, ContractDetailView, ContractPolicyView,
    ContractReviewView, ContractRuntimeBindingView, ContractSummaryView,
};
use crate::contracts::types::{ResponseEnvelope, ResponseStatus};
use crate::contracts::write_model::{
    ContractLifecycleInput, ContractRegistrationInput, ManifestSubmission,
};

pub fn map_validate_request_to_manifest(submission: &ManifestSubmission) -> ContractManifest {
    submission.manifest.clone()
}

pub fn map_register_request_to_descriptor(
    input: &ContractRegistrationInput,
) -> Result<ContractDescriptor, ContractRpcError> {
    ContractDescriptor::new(input.manifest_submission.manifest.clone())
        .map_err(|err| ContractRpcError::ValidationFailed(err.to_string()))
}

pub fn map_activation_request_to_contract_id(input: &ContractLifecycleInput) -> ContractId {
    ContractId(input.contract_id.clone())
}

pub fn map_record_to_contract_summary(record: &OnChainContractRecord) -> ContractSummaryView {
    ContractSummaryView {
        contract_id: record.contract_id.0.clone(),
        status: format!("{:?}", record.status).to_lowercase(),
        package: record.manifest.package.clone(),
        vm_target: format!("{:?}", record.manifest.vm_target).to_lowercase(),
    }
}

pub fn map_record_to_contract_detail(
    record: &OnChainContractRecord,
    review: Option<&ApprovalMarker>,
    runtime_binding: Option<ContractRuntimeBindingView>,
) -> ContractDetailView {
    ContractDetailView {
        summary: map_record_to_contract_summary(record),
        artifact: ContractArtifactView {
            location: record.manifest.artifact.artifact_path_or_uri.clone(),
            format: format!("{:?}", record.manifest.artifact.artifact_format).to_lowercase(),
            size: record.manifest.artifact.artifact_size,
        },
        policy: ContractPolicyView {
            allowed_vm_targets: record
                .manifest
                .policy
                .allowed_vm_targets
                .iter()
                .map(|vm| format!("{:?}", vm).to_lowercase())
                .collect(),
            allowed_artifact_formats: record
                .manifest
                .policy
                .allowed_artifact_formats
                .iter()
                .map(|fmt| format!("{:?}", fmt).to_lowercase())
                .collect(),
            review_required: record.manifest.policy.review_required,
            signature_required: record.manifest.policy.signature_required,
        },
        compatibility: ContractCompatibilityView {
            minimum_schema_version: record.manifest.compatibility.minimum_schema_version,
            supported_schema_versions: record
                .manifest
                .compatibility
                .supported_schema_versions
                .clone(),
        },
        review: ContractReviewView {
            status: review.map(|r| format!("{:?}", r.status).to_lowercase()),
            note: review.and_then(|r| r.note.clone()),
        },
        runtime_binding,
    }
}

pub fn map_runtime_binding_to_response(
    lane: String,
    execution_profile: String,
) -> ContractRuntimeBindingView {
    ContractRuntimeBindingView {
        lane,
        execution_profile,
    }
}

pub fn map_receipt_to_envelope<T>(
    request_id: &str,
    contract_id: Option<String>,
    receipt: Option<&ContractReceipt>,
    data: T,
) -> ResponseEnvelope<T> {
    ResponseEnvelope {
        request_id: request_id.to_string(),
        status: ResponseStatus::Ok,
        contract_id,
        warnings: vec![],
        errors: vec![],
        timestamp: Utc::now(),
        receipt: receipt.map(|r| format!("{:?}", r)),
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoxcontract::{
        ArtifactDigest, ArtifactDigestAlgorithm, ArtifactFormat, ArtifactLocationKind,
        Compatibility, ContractArtifactRef, ContractMetadata, ContractPolicy, ContractVersion,
        Entrypoint, Integrity, NetworkClass, RuntimeFamily, SourceTrustLevel, VmTarget,
    };
    use aoxcore::contract::record::{ManifestDigest, RegisteredAtHeight};

    fn record() -> OnChainContractRecord {
        let digest = ArtifactDigest {
            algorithm: ArtifactDigestAlgorithm::Sha256,
            value: "abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abcd".into(),
        };
        let artifact = ContractArtifactRef::new(
            digest.clone(),
            64,
            ArtifactFormat::WasmModule,
            ArtifactLocationKind::Uri,
            "ipfs://contract",
            None,
            Some("application/wasm".into()),
            VmTarget::Wasm,
        )
        .unwrap();
        let manifest = ContractManifest::new(
            "rpc_contract",
            "aox.rpc",
            "1.0.0",
            ContractVersion("1.0.0".into()),
            VmTarget::Wasm,
            artifact,
            vec![Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap()],
            digest.clone(),
            ContractMetadata {
                display_name: "RPC".into(),
                description: None,
                author: None,
                organization: None,
                source_reference: None,
                tags: vec![],
                created_at: None,
                updated_at: None,
                audit_reference: None,
                notes: None,
            },
            ContractPolicy::new(
                vec![VmTarget::Wasm],
                vec![ArtifactFormat::WasmModule],
                1024,
                vec![],
                vec![],
                true,
                true,
                SourceTrustLevel::ReviewRequired,
            )
            .unwrap(),
            Compatibility::new(
                1,
                vec![1],
                vec![RuntimeFamily::Wasm],
                vec![NetworkClass::Mainnet],
                vec![],
                false,
            )
            .unwrap(),
            Integrity {
                digest,
                artifact_size: 64,
                artifact_format: ArtifactFormat::WasmModule,
                media_type: Some("application/wasm".into()),
                signature_required: true,
                source_trust_level: SourceTrustLevel::ReviewRequired,
            },
            1,
        )
        .unwrap();
        let descriptor = ContractDescriptor::new(manifest.clone()).unwrap();
        OnChainContractRecord {
            contract_id: descriptor.contract_id,
            manifest,
            status: aoxcontract::ContractStatus::Registered,
            manifest_digest: ManifestDigest("digest".into()),
            registered_at_height: RegisteredAtHeight(1),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn record_mapping_is_complete() {
        let detail = map_record_to_contract_detail(&record(), None, None);
        assert_eq!(detail.summary.package, "aox.rpc");
        assert_eq!(detail.artifact.size, 64);
    }
}
