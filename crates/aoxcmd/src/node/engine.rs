use crate::{
    error::AppError,
    node::{lifecycle::{load_state, persist_state}, state::NodeState},
};

pub fn produce_once(tx: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;
    state.current_height += 1;
    state.produced_blocks += 1;
    state.last_tx = tx.to_string();
    state.touch();
    persist_state(&state)?;
    Ok(state)
}

pub fn run_rounds(rounds: u64, tx_prefix: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;
    for index in 0..rounds {
        state.current_height += 1;
        state.produced_blocks += 1;
        state.last_tx = format!("{tx_prefix}-{index}");
    }
    state.touch();
    persist_state(&state)?;
    Ok(state)
}
