#[derive(Debug, Clone)]
pub struct CoreRuntime {
    pub chain_id: String,
}

impl CoreRuntime {
    #[must_use]
    pub fn new(chain_id: impl Into<String>) -> Self {
        Self {
            chain_id: chain_id.into(),
        }
    }
}
