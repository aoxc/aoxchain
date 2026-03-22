use crate::contracts::error::ContractRpcError;
use crate::contracts::types::{
    GetContractRequest, ListContractsRequest, RegisterContractRequest,
    ResolveRuntimeBindingRequest, ValidateManifestRequest,
};

pub fn validate_manifest_request(
    request: &ValidateManifestRequest,
) -> Result<(), ContractRpcError> {
    if request.request_id.trim().is_empty() {
        return Err(ContractRpcError::BadRequest(
            "request_id is required".into(),
        ));
    }
    Ok(())
}

pub fn validate_register_request(
    request: &RegisterContractRequest,
) -> Result<(), ContractRpcError> {
    if request.request_id.trim().is_empty() {
        return Err(ContractRpcError::BadRequest(
            "request_id is required".into(),
        ));
    }
    Ok(())
}

pub fn validate_get_request(request: &GetContractRequest) -> Result<(), ContractRpcError> {
    if request.contract_id.trim().is_empty() {
        return Err(ContractRpcError::BadRequest(
            "contract_id is required".into(),
        ));
    }
    Ok(())
}

pub fn validate_list_request(
    request: &ListContractsRequest,
    max_page_size: usize,
) -> Result<(), ContractRpcError> {
    if request.request_id.trim().is_empty() {
        return Err(ContractRpcError::BadRequest(
            "request_id is required".into(),
        ));
    }
    if request.page_size == 0 || request.page_size > max_page_size {
        return Err(ContractRpcError::BadRequest(
            "page_size exceeds allowed contract API limit".into(),
        ));
    }
    Ok(())
}

pub fn validate_runtime_binding_request(
    request: &ResolveRuntimeBindingRequest,
) -> Result<(), ContractRpcError> {
    if request.request_id.trim().is_empty() {
        return Err(ContractRpcError::BadRequest(
            "request_id is required".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::types::ListContractsRequest;

    #[test]
    fn invalid_list_limit_is_rejected() {
        let err = validate_list_request(
            &ListContractsRequest {
                request_id: "req".into(),
                status: None,
                vm_target: None,
                page: 0,
                page_size: 1000,
            },
            100,
        )
        .unwrap_err();
        assert!(matches!(err, ContractRpcError::BadRequest(_)));
    }
}
