// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

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
