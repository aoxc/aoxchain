//! Audit-grade invocation records for AI extension calls.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::capability::{AiActionClass, AiCapability, KernelZone};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationDisposition {
    Allowed,
    Denied,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiInvocationAuditRecord {
    pub invocation_id: String,
    pub caller_crate: String,
    pub caller_component: String,
    pub requested_action: String,
    pub provider_name: String,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
    pub kernel_zone: KernelZone,
    pub policy_id: String,
    pub input_class: String,
    pub output_class: String,
    pub confidence_bps: u16,
    pub duration_ms: u64,
    pub timeout_hit: bool,
    pub side_effect_intent: bool,
    pub approval_state: String,
    pub final_disposition: InvocationDisposition,
    pub recorded_at: DateTime<Utc>,
}

impl AiInvocationAuditRecord {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        invocation_id: impl Into<String>,
        caller_crate: impl Into<String>,
        caller_component: impl Into<String>,
        requested_action: impl Into<String>,
        provider_name: impl Into<String>,
        capability: AiCapability,
        action_class: AiActionClass,
        kernel_zone: KernelZone,
        policy_id: impl Into<String>,
    ) -> Self {
        Self {
            invocation_id: invocation_id.into(),
            caller_crate: caller_crate.into(),
            caller_component: caller_component.into(),
            requested_action: requested_action.into(),
            provider_name: provider_name.into(),
            capability,
            action_class,
            kernel_zone,
            policy_id: policy_id.into(),
            input_class: "structured".into(),
            output_class: "advisory_artifact".into(),
            confidence_bps: 0,
            duration_ms: 0,
            timeout_hit: false,
            side_effect_intent: false,
            approval_state: "not_required".into(),
            final_disposition: InvocationDisposition::Allowed,
            recorded_at: Utc::now(),
        }
    }
}
