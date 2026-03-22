use serde::Serialize;

use aoxcai::{
    AiActionClass, AiAuditSink, AiCapability, AiInvocationAuditRecord, ExecutionBudget,
    ExtensionDescriptor, InvocationDisposition, InvocationPolicy, KernelZone, MemoryAuditSink,
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
    pub canonical: bool,
    pub executed: bool,
    pub requires_operator_approval: bool,
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

pub struct OperatorPlaneAiAdapter<S: AiAuditSink = MemoryAuditSink> {
    policy: InvocationPolicy,
    advisory_descriptor: ExtensionDescriptor,
    guarded_descriptor: ExtensionDescriptor,
    sink: S,
}

impl Default for OperatorPlaneAiAdapter<MemoryAuditSink> {
    fn default() -> Self {
        Self {
            policy: InvocationPolicy::kernel_default(),
            advisory_descriptor: ExtensionDescriptor {
                id: "operator-diagnostics".into(),
                provider_name: "local-heuristic".into(),
                zone: KernelZone::Operator,
                capability: AiCapability::DiagnosticsAssist, // FIXED
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
            sink: MemoryAuditSink::default(),
        }
    }
}

impl<S: AiAuditSink> OperatorPlaneAiAdapter<S> {
    #[must_use]
    pub fn with_sink(sink: S) -> Self {
        Self {
            policy: InvocationPolicy::kernel_default(),
            advisory_descriptor: ExtensionDescriptor {
                id: "operator-diagnostics".into(),
                provider_name: "local-heuristic".into(),
                zone: KernelZone::Operator,
                capability: AiCapability::DiagnosticsAssist, // FIXED
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
            sink,
        }
    }

    #[must_use]
    pub fn diagnostics_assistance(&self, request: OperatorAssistRequest) -> OperatorAssistOutcome {
        if std::env::var("AOXC_AI_DISABLE").ok().as_deref() == Some("1") {
            let trace = denied_trace(
                "operator-diagnostics-disabled",
                AiCapability::DiagnosticsAssist,
                AiActionClass::Advisory,
                "AI disabled by AOXC_AI_DISABLE",
            );
            self.sink.record(trace.clone());
            return OperatorAssistOutcome {
                available: false,
                artifact: None,
                trace,
            };
        }

        match self.advisory_descriptor.attempt_authorize(
            &self.policy,
            "aoxcmd",
            "diagnostics-doctor",
            request.topic,
        ) {
            Ok(authorized) => {
                self.sink.record(authorized.audit_record.clone());

                OperatorAssistOutcome {
                    available: true,
                    artifact: Some(OperatorAssistArtifact {
                        mode: "advisory",
                        canonical: false,
                        executed: false,
                        requires_operator_approval: false,
                        summary: format!(
                            "Diagnostics analyzed. {} issues detected.",
                            request.failed_checks.len()
                        ),
                        remediation_plan: request.failed_checks,
                        audit: authorized.audit_record.clone(),
                    }),
                    trace: authorized.audit_record,
                }
            }
            Err(trace) => {
                self.sink.record((*trace).clone());
                OperatorAssistOutcome {
                    available: false,
                    artifact: None,
                    trace: *trace,
                }
            }
        }
    }
}