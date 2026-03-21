use crate::ai::{context::default_context, signals::baseline_signals};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AiRuntimeReport {
    pub context_role: &'static str,
    pub anomaly_score: f64,
    pub recommendation: &'static str,
}

pub fn report() -> AiRuntimeReport {
    let context = default_context();
    let signals = baseline_signals();
    AiRuntimeReport {
        context_role: context.role,
        anomaly_score: signals.anomaly_score,
        recommendation: signals.recommendation,
    }
}
