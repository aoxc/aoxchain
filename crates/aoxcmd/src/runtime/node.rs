use crate::{error::AppError, node::lifecycle::load_state};

pub fn health_status() -> Result<&'static str, AppError> {
    let state = load_state()?;
    if state.initialized && state.current_height > 0 {
        Ok("healthy")
    } else if state.initialized {
        Ok("bootstrapped")
    } else {
        Ok("uninitialized")
    }
}
