//! Operator-plane adapter for policy-constrained AI assistance.
//!
//! This module is the first safe integration path between `aoxcmd` and
//! `aoxcai`. It only supports advisory and guarded-preparation outputs for the
//! operator plane. It does not mutate chain state, execute commands, or turn AI
//! output into canonical truth.

use serde::Serialize;

use aoxcai::{
    AiActionClass, AiCapability, AiInvocationAuditRecord, ExecutionBudget, ExtensionDescriptor,
    InvocationDisposition, InvocationPolicy, KernelZone,
};

#[derive(Debug, Clone, Serialize)]
pub struct OperatorAssistRequest {
    pub topic: &'static str,
    pub verdict: String,
    pub failed_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorAssistArtifact {
    pub mode: &'static str,
    pub summary: String,
    pub remediation_plan: Vec<String>,
    pub audit: AiInvocationAuditRecord,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorAssistOutcome {
    pub available: bool,
    pub artifact: Option<OperatorAssistArtifact>,
    pub trace: AiInvocationAuditRecord,
}

pub struct OperatorPlaneAiAdapter {
    policy: InvocationPolicy,
    advisory_descriptor: ExtensionDescriptor,
    guarded_descriptor: ExtensionDescriptor,
}

impl Default for OperatorPlaneAiAdapter {
    fn default() -> Self {
        Self {
            policy: InvocationPolicy::kernel_default(),
            advisory_descriptor: ExtensionDescriptor {
                id: "operator-diagnostics".into(),
                provider_name: "local-heuristic".into(),
                zone: KernelZone::Operator,
                capability: AiCapability::Explain,
                action_class: AiActionClass::Advisory,
                budget: ExecutionBudget::default(),
            },
            guarded_descriptor: ExtensionDescriptor {
                id: "operator-runbook".into(),
                provider_name: "local-heuristic".into(),
                zone: KernelZone::Operator,
                capability: AiCapability::RunbookGenerate,
                action_class: AiActionClass::GuardedPreparation,
                budget: ExecutionBudget::default(),
            },
        }
    }
}

impl OperatorPlaneAiAdapter {
    #[must_use]
    pub fn diagnostics_assistance(&self, request: OperatorAssistRequest) -> OperatorAssistOutcome {
        if std::env::var("AOXC_AI_DISABLE").ok().as_deref() == Some("1") {
            return OperatorAssistOutcome {
                available: false,
                artifact: None,
                trace: denied_trace(
                    "operator-diagnostics-disabled",
                    AiCapability::Explain,
                    AiActionClass::Advisory,
                    "AI disabled by AOXC_AI_DISABLE",
                ),
            };
        }

        match self.advisory_descriptor.attempt_authorize(
            &self.policy,
            "aoxcmd",
            "diagnostics-doctor",
            request.topic,
        ) {
            Ok(authorized) => {
                let artifact = OperatorAssistArtifact {
                    mode: "advisory",
                    summary: if request.failed_checks.is_empty() {
                        format!(
                            "Operator diagnostics are healthy. Verdict '{}' requires no remediation.",
                            request.verdict
                        )
                    } else {
                        format!(
                            "Operator diagnostics detected {} failed checks. AI assistance remains advisory only.",
                            request.failed_checks.len()
                        )
                    },
                    remediation_plan: request
                        .failed_checks
                        .iter()
                        .map(|check| format!("Review and remediate operator check '{check}' before state-changing actions."))
                        .collect(),
                    audit: authorized.audit_record.clone(),
                };
                OperatorAssistOutcome {
                    available: true,
                    artifact: Some(artifact),
                    trace: authorized.audit_record,
                }
            }
            Err(trace) => OperatorAssistOutcome {
                available: false,
                artifact: None,
                trace: *trace,
            },
        }
    }

    #[must_use]
    pub fn remediation_runbook(&self, request: OperatorAssistRequest) -> OperatorAssistOutcome {
        match self.guarded_descriptor.attempt_authorize(
            &self.policy,
            "aoxcmd",
            "diagnostics-doctor",
            request.topic,
        ) {
            Ok(authorized) => {
                let artifact = OperatorAssistArtifact {
                    mode: "guarded_preparation",
                    summary: "Prepared operator runbook draft. Manual approval remains required.".into(),
                    remediation_plan: request
                        .failed_checks
                        .iter()
                        .map(|check| format!("Draft remediation step for '{check}' prepared for operator review."))
                        .collect(),
                    audit: authorized.audit_record.clone(),
                };
                OperatorAssistOutcome {
                    available: true,
                    artifact: Some(artifact),
                    trace: authorized.audit_record,
                }
            }
            Err(trace) => OperatorAssistOutcome {
                available: false,
                artifact: None,
                trace: *trace,
            },
        }
    }
}

fn denied_trace(
    requested_action: &str,
    capability: AiCapability,
    action_class: AiActionClass,
    note: &str,
) -> AiInvocationAuditRecord {
    let mut audit = AiInvocationAuditRecord::new(
        format!("denied:{requested_action}"),
        "aoxcmd",
        "diagnostics-doctor",
        requested_action,
        "disabled",
        capability,
        action_class,
        KernelZone::Operator,
        "aoxcai-kernel-default",
    );
    audit.final_disposition = InvocationDisposition::Denied;
    audit.approval_state = note.to_string();
    audit
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoxcai::{
        AiActionClass, AiCapability, ExtensionDescriptor, InvocationDisposition, InvocationPolicy,
        KernelZone,
    };

    #[test]
    fn operator_plane_advisory_action_is_allowed() {
        let adapter = OperatorPlaneAiAdapter::default();
        let outcome = adapter.diagnostics_assistance(OperatorAssistRequest {
            topic: "diagnostics_explanation",
            verdict: "fail".into(),
            failed_checks: vec!["config-valid".into()],
        });

        assert!(outcome.available);
        assert_eq!(
            outcome.trace.final_disposition,
            InvocationDisposition::Allowed
        );
    }

    #[test]
    fn guarded_preparation_action_is_allowed_only_when_policy_grants_it() {
        let adapter = OperatorPlaneAiAdapter::default();
        let outcome = adapter.remediation_runbook(OperatorAssistRequest {
            topic: "remediation_runbook",
            verdict: "fail".into(),
            failed_checks: vec!["node-state".into()],
        });
        assert!(outcome.available);
        assert_eq!(outcome.artifact.unwrap().mode, "guarded_preparation");
    }

    #[test]
    fn kernel_correctness_is_unaffected_when_ai_is_disabled() {
        std::env::set_var("AOXC_AI_DISABLE", "1");
        let adapter = OperatorPlaneAiAdapter::default();
        let outcome = adapter.diagnostics_assistance(OperatorAssistRequest {
            topic: "diagnostics_explanation",
            verdict: "pass".into(),
            failed_checks: vec![],
        });
        std::env::remove_var("AOXC_AI_DISABLE");

        assert!(!outcome.available);
        assert_eq!(
            outcome.trace.final_disposition,
            InvocationDisposition::Denied
        );
    }

    #[test]
    fn unauthorized_capability_rejection_emits_denied_trace() {
        let descriptor = ExtensionDescriptor {
            id: "operator-danger".into(),
            provider_name: "local-heuristic".into(),
            zone: KernelZone::Operator,
            capability: AiCapability::DiagnosticsAssist,
            action_class: AiActionClass::GuardedPreparation,
            budget: ExecutionBudget::default(),
        };
        let denied = *descriptor
            .attempt_authorize(
                &InvocationPolicy::kernel_default(),
                "aoxcmd",
                "diagnostics-doctor",
                "dangerous_operator_action",
            )
            .unwrap_err();
        assert_eq!(denied.final_disposition, InvocationDisposition::Denied);
    }
}
