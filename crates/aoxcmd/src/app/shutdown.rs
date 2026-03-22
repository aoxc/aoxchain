use crate::{
    error::AppError,
    node::lifecycle::{load_state, persist_state},
};

pub fn graceful_shutdown() -> Result<(), AppError> {
    let mut state = load_state()?;
    state.running = false;
    state.touch();
    persist_state(&state)?;
    Ok(())
}
