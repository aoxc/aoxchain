//! Thin contract HTTP-style handlers.
//!
//! These functions emulate route handlers by parsing requests, performing
//! transport validation, delegating to services, and returning typed response
//! envelopes without embedding contract/business logic.

use aoxcdata::contracts::store::ContractStore;
use aoxconfig::contracts::ContractsConfig;
use aoxcore::contract::registry::ContractRegistry;

use crate::contracts::error::ContractRpcError;
use crate::contracts::mapper::{
    map_activation_request_to_contract_id, map_receipt_to_envelope, map_record_to_contract_detail,
    map_record_to_contract_summary, map_register_request_to_descriptor,
    map_validate_request_to_manifest,
};
use crate::contracts::service::{
    ContractCommandService, ContractQueryService, ContractRuntimeBindingService,
    ContractValidationService,
};
use crate::contracts::types::*;
use crate::contracts::validation::{
    validate_get_request, validate_list_request, validate_manifest_request,
    validate_register_request, validate_runtime_binding_request,
};

#[derive(Debug, Default)]
pub struct ContractHttpApi {
    pub contracts_config: ContractsConfig,
    pub registry: ContractRegistry,
    pub store: ContractStore,
}

impl ContractHttpApi {
    pub fn validate_manifest(
        &self,
        request: ValidateManifestRequest,
    ) -> Result<ValidateManifestResponse, ContractRpcError> {
        validate_manifest_request(&request)?;
        let manifest = map_validate_request_to_manifest(&request.submission);
        let descriptor = ContractValidationService::validate_manifest(&manifest)?;
        let runtime_binding = ContractRuntimeBindingService::resolve_runtime_binding(
            &descriptor,
            &self.contracts_config,
        )
        .ok();
        let record = aoxcore::contract::record::OnChainContractRecord {
            contract_id: descriptor.contract_id.clone(),
            manifest,
            status: aoxcontract::ContractStatus::Draft,
            manifest_digest: aoxcore::contract::record::ManifestDigest("preflight".into()),
            registered_at_height: aoxcore::contract::record::RegisteredAtHeight(0),
            updated_at: chrono::Utc::now(),
        };
        Ok(map_receipt_to_envelope(
            &request.request_id,
            Some(descriptor.contract_id.0.clone()),
            None,
            map_record_to_contract_detail(&record, None, runtime_binding),
        ))
    }

    pub fn register_contract(
        &mut self,
        request: RegisterContractRequest,
    ) -> Result<RegisterContractResponse, ContractRpcError> {
        validate_register_request(&request)?;
        let descriptor = map_register_request_to_descriptor(&request.input)?;
        let receipt = ContractCommandService::register_contract(
            &mut self.registry,
            Some(&mut self.store),
            descriptor.clone(),
            1,
        )?;
        let record = self
            .registry
            .get_contract(&descriptor.contract_id)
            .cloned()
            .ok_or_else(|| ContractRpcError::RegistryError("registered contract missing".into()))?;
        let runtime_binding = ContractRuntimeBindingService::resolve_runtime_binding(
            &descriptor,
            &self.contracts_config,
        )
        .ok();
        Ok(map_receipt_to_envelope(
            &request.request_id,
            Some(descriptor.contract_id.0),
            Some(&receipt),
            map_record_to_contract_detail(&record, request.input.review.as_ref(), runtime_binding),
        ))
    }

    pub fn get_contract(
        &self,
        request: GetContractRequest,
    ) -> Result<GetContractResponse, ContractRpcError> {
        validate_get_request(&request)?;
        let contract_id = aoxcontract::ContractId(request.contract_id.clone());
        let record = ContractQueryService::get_contract(&self.registry, &contract_id, None, None)?;
        Ok(map_receipt_to_envelope(
            &request.request_id,
            Some(request.contract_id),
            None,
            map_record_to_contract_detail(&record, None, None),
        ))
    }

    pub fn list_contracts(
        &self,
        request: ListContractsRequest,
    ) -> Result<ListContractsResponse, ContractRpcError> {
        validate_list_request(
            &request,
            self.contracts_config.limits.max_entrypoints.max(100),
        )?;
        let records = ContractQueryService::list_contracts(&self.registry);
        let start = request.page.saturating_mul(request.page_size);
        let data = records
            .into_iter()
            .skip(start)
            .take(request.page_size)
            .map(|r| map_record_to_contract_summary(&r))
            .collect();
        Ok(map_receipt_to_envelope(
            &request.request_id,
            None,
            None,
            data,
        ))
    }

    pub fn activate_contract(
        &mut self,
        request: ActivateContractRequest,
    ) -> Result<ActivateContractResponse, ContractRpcError> {
        let contract_id = map_activation_request_to_contract_id(&request.input);
        let receipt = ContractCommandService::activate_contract(&mut self.registry, &contract_id)?;
        let record = self
            .registry
            .get_contract(&contract_id)
            .cloned()
            .ok_or_else(|| ContractRpcError::NotFound(contract_id.0.clone()))?;
        Ok(map_receipt_to_envelope(
            &request.request_id,
            Some(contract_id.0),
            Some(&receipt),
            map_record_to_contract_detail(&record, None, None),
        ))
    }

    pub fn deprecate_contract(
        &mut self,
        request: DeprecateContractRequest,
    ) -> Result<DeprecateContractResponse, ContractRpcError> {
        let contract_id = map_activation_request_to_contract_id(&request.input);
        let receipt = ContractCommandService::deprecate_contract(&mut self.registry, &contract_id)?;
        let record = self
            .registry
            .get_contract(&contract_id)
            .cloned()
            .ok_or_else(|| ContractRpcError::NotFound(contract_id.0.clone()))?;
        Ok(map_receipt_to_envelope(
            &request.request_id,
            Some(contract_id.0),
            Some(&receipt),
            map_record_to_contract_detail(&record, None, None),
        ))
    }

    pub fn revoke_contract(
        &mut self,
        request: RevokeContractRequest,
    ) -> Result<RevokeContractResponse, ContractRpcError> {
        let contract_id = map_activation_request_to_contract_id(&request.input);
        let receipt = ContractCommandService::revoke_contract(&mut self.registry, &contract_id)?;
        let record = self
            .registry
            .get_contract(&contract_id)
            .cloned()
            .ok_or_else(|| ContractRpcError::NotFound(contract_id.0.clone()))?;
        Ok(map_receipt_to_envelope(
            &request.request_id,
            Some(contract_id.0),
            Some(&receipt),
            map_record_to_contract_detail(&record, None, None),
        ))
    }

    pub fn resolve_runtime_binding(
        &self,
        request: ResolveRuntimeBindingRequest,
    ) -> Result<ResolveRuntimeBindingResponse, ContractRpcError> {
        validate_runtime_binding_request(&request)?;
        let manifest = map_validate_request_to_manifest(&request.input.manifest_submission);
        let descriptor = ContractValidationService::validate_manifest(&manifest)?;
        let binding = ContractRuntimeBindingService::resolve_runtime_binding(
            &descriptor,
            &self.contracts_config,
        )?;
        Ok(map_receipt_to_envelope(
            &request.request_id,
            Some(descriptor.contract_id.0),
            None,
            binding,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::write_model::{
        ContractLifecycleInput, ContractRegistrationInput, ManifestSubmission,
        RuntimeBindingResolutionInput,
    };
    use aoxcontract::{
        ApprovalMarker, ArtifactDigest, ArtifactDigestAlgorithm, ContractMetadata,
        ContractReviewStatus, Entrypoint, VmTarget,
    };
    use aoxcsdk::contracts::builder::ContractManifestBuilder;

    fn manifest() -> aoxcontract::ContractManifest {
        ContractManifestBuilder::new()
            .with_name("rpc_native_contract")
            .with_package("aox.rpc")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(VmTarget::Wasm)
            .with_artifact_digest(ArtifactDigest {
                algorithm: ArtifactDigestAlgorithm::Sha256,
                value: "9f4dcc3b5aa765d61d8327deb882cf9922222222222222222222222222222222".into(),
            })
            .with_artifact_location("ipfs://rpc/contract.wasm")
            .with_metadata(ContractMetadata {
                display_name: "RPC Contract".into(),
                description: None,
                author: None,
                organization: None,
                source_reference: None,
                tags: vec![],
                created_at: None,
                updated_at: None,
                audit_reference: None,
                notes: None,
            })
            .add_entrypoint(Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap())
            .build()
            .unwrap()
    }

    #[test]
    fn valid_manifest_validate_succeeds() {
        let api = ContractHttpApi::default();
        let response = api
            .validate_manifest(ValidateManifestRequest {
                request_id: "req-1".into(),
                submission: ManifestSubmission {
                    manifest: manifest(),
                },
            })
            .unwrap();
        assert_eq!(response.status, ResponseStatus::Ok);
        assert_eq!(response.data.summary.status, "draft");
    }

    #[test]
    fn invalid_manifest_returns_422() {
        let api = ContractHttpApi::default();
        let mut bad = manifest();
        bad.name = " ".into();
        let err = api
            .validate_manifest(ValidateManifestRequest {
                request_id: "req-2".into(),
                submission: ManifestSubmission { manifest: bad },
            })
            .unwrap_err();
        assert_eq!(err.http_status(), 422);
    }

    #[test]
    fn register_get_list_and_lifecycle_flow_work() {
        let mut api = ContractHttpApi::default();
        let register = api
            .register_contract(RegisterContractRequest {
                request_id: "req-3".into(),
                input: ContractRegistrationInput {
                    manifest_submission: ManifestSubmission {
                        manifest: manifest(),
                    },
                    review: Some(ApprovalMarker {
                        reviewer: "sec".into(),
                        status: ContractReviewStatus::Approved,
                        note: None,
                    }),
                },
            })
            .unwrap();
        let contract_id = register.contract_id.clone().unwrap();
        assert_eq!(register.data.summary.status, "registered");

        let get = api
            .get_contract(GetContractRequest {
                request_id: "req-4".into(),
                contract_id: contract_id.clone(),
            })
            .unwrap();
        assert_eq!(get.data.summary.contract_id, contract_id);

        let list = api
            .list_contracts(ListContractsRequest {
                request_id: "req-5".into(),
                status: None,
                vm_target: None,
                page: 0,
                page_size: 10,
            })
            .unwrap();
        assert_eq!(list.data.len(), 1);

        let active = api
            .activate_contract(ActivateContractRequest {
                request_id: "req-6".into(),
                input: ContractLifecycleInput {
                    contract_id: contract_id.clone(),
                },
            })
            .unwrap();
        assert_eq!(active.data.summary.status, "active");

        let deprecated = api
            .deprecate_contract(DeprecateContractRequest {
                request_id: "req-7".into(),
                input: ContractLifecycleInput {
                    contract_id: contract_id.clone(),
                },
            })
            .unwrap();
        assert_eq!(deprecated.data.summary.status, "deprecated");

        let revoked = api
            .revoke_contract(RevokeContractRequest {
                request_id: "req-8".into(),
                input: ContractLifecycleInput { contract_id },
            })
            .unwrap();
        assert_eq!(revoked.data.summary.status, "revoked");
    }

    #[test]
    fn duplicate_register_returns_409() {
        let mut api = ContractHttpApi::default();
        let request = RegisterContractRequest {
            request_id: "req-9".into(),
            input: ContractRegistrationInput {
                manifest_submission: ManifestSubmission {
                    manifest: manifest(),
                },
                review: None,
            },
        };
        api.register_contract(request.clone()).unwrap();
        let err = api.register_contract(request).unwrap_err();
        assert_eq!(err.http_status(), 409);
    }

    #[test]
    fn get_unknown_contract_returns_404() {
        let api = ContractHttpApi::default();
        let err = api
            .get_contract(GetContractRequest {
                request_id: "req-10".into(),
                contract_id: "missing".into(),
            })
            .unwrap_err();
        assert_eq!(err.http_status(), 404);
    }

    #[test]
    fn resolve_runtime_binding_returns_lane() {
        let api = ContractHttpApi::default();
        let response = api
            .resolve_runtime_binding(ResolveRuntimeBindingRequest {
                request_id: "req-11".into(),
                input: RuntimeBindingResolutionInput {
                    manifest_submission: ManifestSubmission {
                        manifest: manifest(),
                    },
                },
            })
            .unwrap();
        assert!(response.data.lane.contains("wasm"));
    }
}
