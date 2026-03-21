use crate::telemetry::tracing::{new_context, TraceContext};

pub fn trace_for(command: &str) -> TraceContext {
    new_context(command)
}
