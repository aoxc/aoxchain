// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::telemetry::tracing::{new_context, TraceContext};

pub fn trace_for(command: &str) -> TraceContext {
    new_context(command)
}
