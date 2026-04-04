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
