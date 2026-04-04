use super::*;

pub fn cmd_vm_trace(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmTraceStep {
        index: u64,
        op: &'static str,
        gas: u64,
    }

    #[derive(serde::Serialize)]
    struct VmTraceView {
        tx_hash: Option<String>,
        trace: Vec<VmTraceStep>,
        source: &'static str,
    }

    let response = VmTraceView {
        tx_hash: arg_value(args, "--tx-hash").and_then(|value| normalize_text(&value, false)),
        trace: vec![
            VmTraceStep {
                index: 0,
                op: "BEGIN",
                gas: 21_000,
            },
            VmTraceStep {
                index: 1,
                op: "END",
                gas: 0,
            },
        ],
        source: "deterministic-local",
    };
    emit_serialized(&response, output_format(args))
}
