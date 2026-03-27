// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    config::loader::load_or_init,
    data_home::{ensure_layout, resolve_home},
    error::AppError,
};

pub fn ensure_operator_environment() -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    let _ = load_or_init()?;
    Ok(())
}
