// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AiSignalSet {
    pub anomaly_score: f64,
    pub recommendation: &'static str,
}

pub fn baseline_signals() -> AiSignalSet {
    AiSignalSet {
        anomaly_score: 0.0,
        recommendation: "No anomaly detected in the current local workflow surface",
    }
}
