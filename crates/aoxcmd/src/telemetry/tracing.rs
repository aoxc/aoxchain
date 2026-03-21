use chrono::Utc;
use serde::Serialize;
use sha3::{Digest, Sha3_256};

#[derive(Debug, Clone, Serialize)]
pub struct TraceContext {
    pub correlation_id: String,
    pub recorded_at: String,
}

pub fn new_context(command: &str) -> TraceContext {
    let recorded_at = Utc::now().to_rfc3339();
    let mut hasher = Sha3_256::new();
    hasher.update(command.as_bytes());
    hasher.update(recorded_at.as_bytes());
    let correlation_id = hex::encode(hasher.finalize())[..16].to_string();
    TraceContext {
        correlation_id,
        recorded_at,
    }
}
