use super::*;

pub fn cmd_vm_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct VmStatus {
        vm_enabled: bool,
        execution_plane: &'static str,
        execution_mode: &'static str,
        latest_height: u64,
        last_executed_block: u64,
        latest_tx_marker: String,
        last_execution_status: &'static str,
        total_tx_in_last_block: u64,
        executed_tx_count: u64,
        failed_tx_count: u64,
        runtime_running: bool,
        state_root: String,
        updated_at: String,
    }

    let state = lifecycle::load_state()?;
    let state_root = derive_state_root(&state)?;
    let has_last_tx = state.last_tx != "none";
    let status = VmStatus {
        vm_enabled: true,
        execution_plane: "deterministic-local",
        execution_mode: "local-snapshot",
        latest_height: state.current_height,
        last_executed_block: state.current_height,
        latest_tx_marker: state.last_tx,
        last_execution_status: if has_last_tx { "ok" } else { "idle" },
        total_tx_in_last_block: u64::from(has_last_tx),
        executed_tx_count: state.produced_blocks,
        failed_tx_count: 0,
        runtime_running: state.running,
        state_root,
        updated_at: state.updated_at,
    };

    emit_serialized(&status, output_format(args))
}

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
