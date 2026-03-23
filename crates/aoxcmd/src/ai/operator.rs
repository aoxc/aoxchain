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
        Self::with_sink(MemoryAuditSink::default())
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
                capability: AiCapability::DiagnosticsAssist,
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
        if ai_disabled() {
            let trace = denied_trace(
                "operator-diagnostics-disabled",
                request.topic,
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

                let failed_issue_count = request.failed_checks.len();
                let verdict = request.verdict;
                let remediation_plan = request.failed_checks;

                let summary = if failed_issue_count == 0 {
                    format!("Native diagnostics verdict is '{verdict}'. No failed checks were reported. AI output remains advisory.")
                } else {
                    format!("Native diagnostics verdict is '{verdict}'. {failed_issue_count} failed checks were summarized for operator review. AI output remains advisory.")
                };

                OperatorAssistOutcome {
                    available: true,
                    artifact: Some(OperatorAssistArtifact {
                        mode: "advisory",
                        canonical: false,
                        executed: false,
                        requires_operator_approval: false,
                        summary,
                        remediation_plan,
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

    #[must_use]
    pub fn runbook_preparation(&self, request: OperatorAssistRequest) -> OperatorAssistOutcome {
        if ai_disabled() {
            let trace = denied_trace(
                "operator-runbook-disabled",
                request.topic,
                AiCapability::RunbookGenerate,
                AiActionClass::GuardedPreparation,
                "AI disabled by AOXC_AI_DISABLE",
            );
            self.sink.record(trace.clone());
            return OperatorAssistOutcome {
                available: false,
                artifact: None,
                trace,
            };
        }

        match self.guarded_descriptor.attempt_authorize(
            &self.policy,
            "aoxcmd",
            "diagnostics-doctor",
            request.topic,
        ) {
            Ok(authorized) => {
                self.sink.record(authorized.audit_record.clone());

                let failed_issue_count = request.failed_checks.len();
                let verdict = request.verdict;

                let remediation_plan = if request.failed_checks.is_empty() {
                    vec![
                        "Review native diagnostics output before preparing any operational change."
                            .to_string(),
                        "No failed checks were reported; no runbook action is suggested."
                            .to_string(),
                    ]
                } else {
                    request.failed_checks.iter().map(|check| {
                        format!("Review native failure '{check}' and prepare a human-approved remediation step before execution.")
                    }).collect()
                };

                let summary = format!(
                    "Prepared a guarded runbook draft for native verdict '{verdict}' with {failed_issue_count} failed checks. Operator approval remains mandatory."
                );

                OperatorAssistOutcome {
                    available: true,
                    artifact: Some(OperatorAssistArtifact {
                        mode: "guarded_preparation",
                        canonical: false,
                        executed: false,
                        requires_operator_approval: true,
                        summary,
                        remediation_plan,
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

fn ai_disabled() -> bool {
    std::env::var("AOXC_AI_DISABLE").ok().as_deref() == Some("1")
}

fn denied_trace(
    invocation_id: &str,
    requested_action: &str,
    capability: AiCapability,
    action_class: AiActionClass,
    approval_reason: &str,
) -> AiInvocationAuditRecord {
    let mut trace = AiInvocationAuditRecord::new(
        invocation_id,
        "aoxcmd",
        "operator-plane-ai-adapter",
        requested_action,
        "disabled",
        capability,
        action_class,
        KernelZone::Operator,
        "aoxcai-kernel-default",
    );
    trace.final_disposition = InvocationDisposition::Denied;
    trace.approval_state = approval_reason.to_string();
    trace.output_class = "no_artifact".to_string();
    trace
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn request() -> OperatorAssistRequest {
        OperatorAssistRequest {
            topic: "diagnostics_explanation",
            verdict: "fail".to_string(),
            failed_checks: vec!["key-material".to_string(), "node-state".to_string()],
        }
    }

    #[test]
    fn diagnostics_artifact_is_non_canonical_and_non_executing() {
        unsafe {
            std::env::remove_var("AOXC_AI_DISABLE");
        }
        let sink = MemoryAuditSink::default();
        let adapter = OperatorPlaneAiAdapter::with_sink(sink.clone());

        let outcome = adapter.diagnostics_assistance(request());
        let artifact = outcome.artifact.expect("artifact should be present");

        assert!(outcome.available);
        assert!(!artifact.canonical);
        assert!(!artifact.executed);
        assert!(!artifact.requires_operator_approval);
        assert_eq!(
            artifact.audit.final_disposition,
            InvocationDisposition::Allowed
        );
    }

    #[test]
    fn disabled_ai_produces_explicit_denied_audit_evidence() {
        let _guard = env_lock().lock().expect("env mutex must lock");
        unsafe {
            std::env::set_var("AOXC_AI_DISABLE", "1");
        }
        let sink = MemoryAuditSink::default();
        let adapter = OperatorPlaneAiAdapter::with_sink(sink.clone());

        let outcome = adapter.diagnostics_assistance(request());

        assert!(!outcome.available);
        assert!(outcome.artifact.is_none());
        assert_eq!(
            outcome.trace.final_disposition,
            InvocationDisposition::Denied
        );
        assert_eq!(outcome.trace.output_class, "no_artifact");

        unsafe {
            std::env::remove_var("AOXC_AI_DISABLE");
        }
    }

    #[test]
    fn diagnostics_summary_preserves_native_verdict_text() {
        let _guard = env_lock().lock().expect("env mutex must lock");
        unsafe {
            std::env::remove_var("AOXC_AI_DISABLE");
        }
        let adapter = OperatorPlaneAiAdapter::default();

        let outcome = adapter.diagnostics_assistance(request());
        let artifact = outcome.artifact.expect("artifact should be present");

        assert!(artifact
            .summary
            .contains("Native diagnostics verdict is 'fail'"));
    }

    #[test]
    fn runbook_preparation_requires_operator_approval_and_is_non_executing() {
        let _guard = env_lock().lock().expect("env mutex must lock");
        unsafe {
            std::env::remove_var("AOXC_AI_DISABLE");
        }
        let adapter = OperatorPlaneAiAdapter::default();

        let outcome = adapter.runbook_preparation(request());
        let artifact = outcome.artifact.expect("artifact should be present");

        assert!(outcome.available);
        assert!(artifact.requires_operator_approval);
    }

    #[test]
    fn disabled_runbook_preparation_produces_denied_trace() {
        let _guard = env_lock().lock().expect("env mutex must lock");
        unsafe {
            std::env::set_var("AOXC_AI_DISABLE", "1");
        }
        let adapter = OperatorPlaneAiAdapter::default();

        let outcome = adapter.runbook_preparation(request());

        assert!(!outcome.available);
        assert_eq!(
            outcome.trace.final_disposition,
            InvocationDisposition::Denied
        );

        unsafe {
            std::env::remove_var("AOXC_AI_DISABLE");
        }
    }
}
