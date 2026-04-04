use super::*;

pub fn cmd_vm_contract_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmContractView {
        address: String,
        exists: bool,
        code_hash: String,
        source: &'static str,
    }

    let address = arg_value(args, "--address")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --address must not be blank",
            )
        })?;
    let response = VmContractView {
        address,
        exists: false,
        code_hash: "0x0".to_string(),
        source: "local-snapshot",
    };
    emit_serialized(&response, output_format(args))
}
