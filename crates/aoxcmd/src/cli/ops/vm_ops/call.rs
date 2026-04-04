use super::*;

pub fn cmd_vm_call(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmCallView {
        to: String,
        from: Option<String>,
        data: Option<String>,
        read_only: bool,
        status: &'static str,
        return_data: String,
        source: &'static str,
    }

    let to = arg_value(args, "--to")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --to must not be blank",
            )
        })?;
    let from = arg_value(args, "--from").and_then(|value| normalize_text(&value, false));
    let data = arg_value(args, "--data").and_then(|value| normalize_text(&value, false));
    let response = VmCallView {
        to,
        from,
        data,
        read_only: true,
        status: "simulated-local",
        return_data: "0x".to_string(),
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}
