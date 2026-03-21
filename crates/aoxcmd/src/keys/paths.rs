use crate::{data_home::resolve_home, error::AppError};
use std::path::PathBuf;

pub fn operator_key_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("keys").join("operator_key.json"))
}
