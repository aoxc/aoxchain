use crate::{
    config::loader::load_or_init, error::AppError, node::lifecycle::load_state,
    runtime::context::RuntimeContext,
};

pub fn runtime_context() -> Result<RuntimeContext, AppError> {
    let settings = load_or_init()?;
    let node_state = load_state().ok();
    Ok(RuntimeContext {
        settings,
        node_state,
    })
}
