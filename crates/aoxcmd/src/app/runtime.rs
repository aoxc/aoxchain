use crate::app::bootstrap::bootstrap;
use crate::error::app_error::AppError;

pub fn run_once(password: &str) -> Result<(), AppError> {
    let (_context, _handles) = bootstrap(password)?;
    Ok(())
}
