// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Service layer orchestrating contract RPC operations.
//!
//! These services keep HTTP handlers thin and avoid duplicating domain logic.

use aoxcdata::contracts::store::ContractStore;
use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{ApprovalMarker, ContractDescriptor, ContractManifest};
use aoxcore::contract::receipt::ContractReceipt;
use aoxcore::contract::record::OnChainContractRecord;
use aoxcore::contract::registry::ContractRegistry;
use aoxcvm::contracts::resolver::resolve_runtime_binding;

use crate::contracts::error::ContractRpcError;
use crate::contracts::mapper::{map_record_to_contract_detail, map_runtime_binding_to_response};
use crate::contracts::read_model::ContractRuntimeBindingView;

pub struct ContractValidationService;

impl ContractValidationService {
    pub fn validate_manifest(
        manifest: &ContractManifest,
    ) -> Result<ContractDescriptor, ContractRpcError> {
        ContractDescriptor::new(manifest.clone())
            .map_err(|err| ContractRpcError::ValidationFailed(err.to_string()))
    }
}

pub struct ContractQueryService;

impl ContractQueryService {
    pub fn get_contract(
        registry: &ContractRegistry,
        contract_id: &aoxcontract::ContractId,
        review: Option<&ApprovalMarker>,
        runtime_binding: Option<ContractRuntimeBindingView>,
    ) -> Result<OnChainContractRecord, ContractRpcError> {
        registry
            .get_contract(contract_id)
            .cloned()
            .ok_or_else(|| ContractRpcError::NotFound(contract_id.0.clone()))
            .inspect(|record| {
                let _ = map_record_to_contract_detail(record, review, runtime_binding.clone());
            })
    }

    pub fn list_contracts(registry: &ContractRegistry) -> Vec<OnChainContractRecord> {
        registry.all_contracts().into_iter().cloned().collect()
    }
}

pub struct ContractCommandService;

impl ContractCommandService {
    pub fn register_contract(
        registry: &mut ContractRegistry,
        store: Option<&mut ContractStore>,
        descriptor: ContractDescriptor,
        height: u64,
    ) -> Result<ContractReceipt, ContractRpcError> {
        let receipt = registry
            .register_contract(descriptor.clone(), height)
            .map_err(|err| ContractRpcError::Conflict(err.to_string()))?;
        if let Some(store) = store {
            let record = registry
                .get_contract(&descriptor.contract_id)
                .cloned()
                .ok_or_else(|| {
                    ContractRpcError::RegistryError(
                        "registered contract missing from registry".into(),
                    )
                })?;
            store.put(record);
        }
        Ok(receipt)
    }

    pub fn activate_contract(
        registry: &mut ContractRegistry,
        contract_id: &aoxcontract::ContractId,
    ) -> Result<ContractReceipt, ContractRpcError> {
        registry
            .activate_contract(contract_id)
            .map_err(|err| ContractRpcError::Conflict(err.to_string()))
    }

    pub fn deprecate_contract(
        registry: &mut ContractRegistry,
        contract_id: &aoxcontract::ContractId,
    ) -> Result<ContractReceipt, ContractRpcError> {
        registry
            .deprecate_contract(contract_id)
            .map_err(|err| ContractRpcError::Conflict(err.to_string()))
    }

    pub fn revoke_contract(
        registry: &mut ContractRegistry,
        contract_id: &aoxcontract::ContractId,
    ) -> Result<ContractReceipt, ContractRpcError> {
        registry
            .revoke_contract(contract_id)
            .map_err(|err| ContractRpcError::Conflict(err.to_string()))
    }
}

pub struct ContractRuntimeBindingService;

impl ContractRuntimeBindingService {
    pub fn resolve_runtime_binding(
        descriptor: &ContractDescriptor,
        config: &ContractsConfig,
    ) -> Result<ContractRuntimeBindingView, ContractRpcError> {
        let binding = resolve_runtime_binding(descriptor, config)
            .map_err(|err| ContractRpcError::RuntimeResolutionError(err.to_string()))?;
        Ok(map_runtime_binding_to_response(
            format!("{:?}", binding.lane_binding).to_lowercase(),
            binding.execution_profile.0,
        ))
    }
}
