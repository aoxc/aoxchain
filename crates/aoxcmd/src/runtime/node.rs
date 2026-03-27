// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{error::AppError, node::lifecycle::load_state};

pub fn health_status() -> Result<&'static str, AppError> {
    let state = load_state()?;
    if !state.initialized {
        Ok("uninitialized")
    } else if !state.key_material.operational_state.is_empty()
        && state.key_material.operational_state != "active"
    {
        Ok("degraded-key-state")
    } else if state.current_height > 0 {
        Ok("healthy")
    } else {
        Ok("bootstrapped")
    }
}
