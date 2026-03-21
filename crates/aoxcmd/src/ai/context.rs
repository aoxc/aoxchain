use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AiContext {
    pub role: &'static str,
    pub mode: &'static str,
}

pub fn default_context() -> AiContext {
    AiContext {
        role: "operator-assist",
        mode: "policy-constrained",
    }
}
