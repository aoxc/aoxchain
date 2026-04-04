use super::*;

pub fn cmd_vm_storage_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmStorageView {
        address: String,
        key: String,
        value: String,
        found: bool,
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
    let key = arg_value(args, "--key")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --key must not be blank",
            )
        })?;
    let response = VmStorageView {
        address,
        key,
        value: "0x".to_string(),
        found: false,
        source: "local-snapshot",
    };
    emit_serialized(&response, output_format(args))
}
