use super::*;

pub fn cmd_vm_estimate_gas(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmEstimateGasView {
        from: Option<String>,
        to: Option<String>,
        estimated_gas: u64,
        source: &'static str,
    }

    let response = VmEstimateGasView {
        from: arg_value(args, "--from").and_then(|value| normalize_text(&value, false)),
        to: arg_value(args, "--to").and_then(|value| normalize_text(&value, false)),
        estimated_gas: 21_000,
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}
