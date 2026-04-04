use super::*;

pub fn cmd_vm_simulate(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmSimulateView {
        tx_hash: Option<String>,
        from: Option<String>,
        to: Option<String>,
        gas_used: u64,
        success: bool,
        revert_reason: Option<String>,
        trace_available: bool,
        source: &'static str,
    }

    let response = VmSimulateView {
        tx_hash: arg_value(args, "--tx-hash").and_then(|value| normalize_text(&value, false)),
        from: arg_value(args, "--from").and_then(|value| normalize_text(&value, false)),
        to: arg_value(args, "--to").and_then(|value| normalize_text(&value, false)),
        gas_used: 0,
        success: true,
        revert_reason: None,
        trace_available: true,
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}
