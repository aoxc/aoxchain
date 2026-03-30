// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    error::AiError,
    manifest::ModelManifest,
    model::{
        Assessment, InferenceContext, InferenceFinding, InferenceRequest, InferenceSignal,
        ModelOutput,
    },
};

#[async_trait::async_trait]
pub trait SignalProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn collect(
        &self,
        task: crate::model::AiTask,
        subject_id: &str,
    ) -> Result<Vec<InferenceSignal>, AiError>;
}

#[async_trait::async_trait]
pub trait ContextProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn build(
        &self,
        task: crate::model::AiTask,
        subject_id: &str,
    ) -> Result<InferenceContext, AiError>;
}

#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    fn name(&self) -> &'static str;

    async fn infer(
        &self,
        manifest: &ModelManifest,
        request: &InferenceRequest,
    ) -> Result<ModelOutput, AiError>;
}

#[async_trait::async_trait]
pub trait DecisionPolicy: Send + Sync {
    fn name(&self) -> &'static str;

    async fn decide(
        &self,
        manifest: &ModelManifest,
        request: &InferenceRequest,
        output: &ModelOutput,
        findings: &[InferenceFinding],
    ) -> Result<Assessment, AiError>;
}
