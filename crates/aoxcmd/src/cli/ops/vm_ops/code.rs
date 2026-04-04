use super::*;

pub fn cmd_vm_code_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmCodeView {
        address: String,
        code: String,
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
    let response = VmCodeView {
        address,
        code: "0x".to_string(),
        source: "local-snapshot",
    };
    emit_serialized(&response, output_format(args))
}
