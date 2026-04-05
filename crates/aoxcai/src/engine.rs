// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    backend::factory::BackendFactory,
    error::AiError,
    manifest::{BackendFailureAction, ModelManifest},
    model::{
        AiMode, AiTask, Assessment, DecisionAction, DecisionReport, FindingSeverity,
        InferenceFinding, InferenceRequest,
    },
    registry::ModelRegistry,
    traits::{ContextProvider, DecisionPolicy, SignalProvider},
};

include!("engine_core.rs");
include!("engine_tests.rs");
