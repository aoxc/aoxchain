use crate::{
    config::loader::load_or_init,
    data_home::{ensure_layout, resolve_home},
    error::AppError,
};

pub fn bootstrap_operator_home() -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    let _ = load_or_init()?;
    Ok(())
}
