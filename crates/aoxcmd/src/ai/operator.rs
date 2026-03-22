use serde::Serialize;

use aoxcai::{
    AiActionClass, AiAuditSink, AiCapability, AiInvocationAuditRecord, ExecutionBudget,
    ExtensionDescriptor, InvocationDisposition, InvocationPolicy, KernelZone, MemoryAuditSink,
};

/// Operator-plane request envelope for optional AI assistance.
///
/// # Trust Boundary
/// `verdict` and `failed_checks` are produced exclusively by deterministic
/// AOXCMD validation and diagnostic logic. They are forwarded to the AI layer
/// as explanatory context only and must remain the sole source of operational
/// truth and readiness status.
#[derive(Debug, Clone, Serialize)]
pub struct OperatorAssistRequest {
    /// Stable topic identifier describing the operator assistance request.
    pub topic: &'static str,
    /// Native AOXCMD diagnostic verdict. This value remains authoritative.
    pub verdict: String,
    /// Native failed checks emitted by deterministic AOXCMD validation paths.
    pub failed_checks: Vec<String>,
}

/// Non-canonical artifact returned by the operator-plane AI adapter.
///
/// # Security Posture
/// This structure is advisory or preparatory only. It never becomes canonical
/// AOXChain truth, never mutates state, and is never auto-executed by AOXCMD.
#[derive(Debug, Clone, Serialize)]
pub struct OperatorAssistArtifact {
    /// Output classification used by the caller during rendering.
    pub mode: &'static str,
    /// Always `false`; AI output is never canonical AOXChain truth.
    pub canonical: bool,
    /// Always `false`; AOXCMD never auto-executes AI-produced output.
    pub executed: bool,
    /// Indicates whether explicit human approval is required before any
    /// downstream operational use.
    pub requires_operator_approval: bool,
    /// Human-readable explanation or preparation summary.
    pub summary: String,
    /// Proposed advisory or preparatory steps for operator review.
    pub remediation_plan: Vec<String>,
    /// Audit evidence describing the invocation that produced this artifact.
    pub audit: AiInvocationAuditRecord,
}

/// Result envelope returned by the operator-plane adapter.
///
/// `trace` is always returned so the caller can surface audit evidence even
/// when assistance is disabled, denied, or otherwise unavailable.
#[derive(Debug, Clone, Serialize)]
pub struct OperatorAssistOutcome {
    /// Indicates whether an AI artifact was produced and made available.
    pub available: bool,
    /// Optional AI-produced artifact. Absent on denial, disablement, or failure
    /// to produce an authorized advisory/preparatory output.
    pub artifact: Option<OperatorAssistArtifact>,
    /// Mandatory audit trail for the attempted invocation.
    pub trace: AiInvocationAuditRecord,
}

/// Stable operator-plane adapter for AOXCMD.
///
/// # Integration Contract
/// This adapter is the only supported integration shape for `aoxcmd`:
///
/// native diagnostics -> bounded adapter -> `aoxcai` authorization ->
/// auditable optional artifact
///
/// # Security Invariants
/// - The adapter never mutates chain state.
/// - The adapter never upgrades AI output into authority.
/// - All AI access remains policy-gated and auditable.
/// - Native AOXCMD diagnostics remain the sole source of truth.
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
    /// Constructs the adapter with the provided audit sink.
    ///
    /// # Security Notes
    /// The adapter is initialized with the default kernel invocation policy and
    /// two strictly bounded descriptors:
    /// - advisory diagnostics assistance
    /// - guarded-preparation runbook generation
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

    /// Produces an optional advisory explanation for native diagnostics output.
    ///
    /// # Authority Model
    /// The returned artifact is explanatory only. It does not alter the native
    /// AOXCMD verdict, does not grant execution authority, and is omitted
    /// entirely when AI is disabled or authorization is denied by policy.
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
                    format!(
                        "Native diagnostics verdict is '{verdict}'. No failed checks were reported. AI output remains advisory."
                    )
                } else {
                    format!(
                        "Native diagnostics verdict is '{verdict}'. {failed_issue_count} failed checks were summarized for operator review. AI output remains advisory."
                    )
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

    /// Produces a guarded-preparation runbook draft for operator review.
    ///
    /// # Security Posture
    /// The returned artifact is deliberately non-executing and requires explicit
    /// human approval before any downstream use. AOXCMD does not apply, enact,
    /// or execute the generated steps automatically.
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
                    request
                        .failed_checks
                        .iter()
                        .map(|check| {
                            format!(
                                "Review native failure '{check}' and prepare a human-approved remediation step before execution."
                            )
                        })
                        .collect()
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

/// Returns `true` when the operator plane has explicitly disabled AI support.
///
/// # Operational Semantics
/// AI is considered disabled only when `AOXC_AI_DISABLE=1`.
#[must_use]
fn ai_disabled() -> bool {
    std::env::var("AOXC_AI_DISABLE").ok().as_deref() == Some("1")
}

/// Builds a denied invocation trace for explicitly disabled operator-plane AI.
///
/// # Audit Intent
/// This helper ensures that disablement remains observable, attributable, and
/// forensically reviewable even when no artifact is produced.
#[must_use]
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

    fn request() -> OperatorAssistRequest {
        OperatorAssistRequest {
            topic: "diagnostics_explanation",
            verdict: "fail".to_string(),
            failed_checks: vec!["key-material".to_string(), "node-state".to_string()],
        }
    }

    #[test]
    fn diagnostics_artifact_is_non_canonical_and_non_executing() {
        std::env::remove_var("AOXC_AI_DISABLE");
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

        let records = sink.snapshot();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].final_disposition, InvocationDisposition::Allowed);
    }

    #[test]
    fn disabled_ai_produces_explicit_denied_audit_evidence() {
        std::env::set_var("AOXC_AI_DISABLE", "1");
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
        assert!(outcome.trace.approval_state.contains("AOXC_AI_DISABLE"));
        assert_eq!(outcome.trace.capability, AiCapability::DiagnosticsAssist);
        assert_eq!(outcome.trace.kernel_zone, KernelZone::Operator);

        let records = sink.snapshot();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].final_disposition, InvocationDisposition::Denied);

        std::env::remove_var("AOXC_AI_DISABLE");
    }

    #[test]
    fn diagnostics_summary_preserves_native_verdict_text() {
        std::env::remove_var("AOXC_AI_DISABLE");
        let adapter = OperatorPlaneAiAdapter::default();

        let outcome = adapter.diagnostics_assistance(request());
        let artifact = outcome.artifact.expect("artifact should be present");

        assert!(artifact
            .summary
            .contains("Native diagnostics verdict is 'fail'"));
        assert!(artifact.summary.contains("AI output remains advisory"));
    }

    #[test]
    fn runbook_preparation_requires_operator_approval_and_is_non_executing() {
        std::env::remove_var("AOXC_AI_DISABLE");
        let sink = MemoryAuditSink::default();
        let adapter = OperatorPlaneAiAdapter::with_sink(sink.clone());

        let outcome = adapter.runbook_preparation(request());
        let artifact = outcome.artifact.expect("artifact should be present");

        assert!(outcome.available);
        assert_eq!(artifact.mode, "guarded_preparation");
        assert!(!artifact.canonical);
        assert!(!artifact.executed);
        assert!(artifact.requires_operator_approval);
        assert_eq!(
            artifact.audit.final_disposition,
            InvocationDisposition::Allowed
        );
        assert_eq!(
            artifact.audit.action_class,
            AiActionClass::GuardedPreparation
        );

        let records = sink.snapshot();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].final_disposition, InvocationDisposition::Allowed);
    }

    #[test]
    fn disabled_runbook_preparation_produces_denied_trace() {
        std::env::set_var("AOXC_AI_DISABLE", "1");
        let sink = MemoryAuditSink::default();
        let adapter = OperatorPlaneAiAdapter::with_sink(sink.clone());

        let outcome = adapter.runbook_preparation(request());

        assert!(!outcome.available);
        assert!(outcome.artifact.is_none());
        assert_eq!(
            outcome.trace.final_disposition,
            InvocationDisposition::Denied
        );
        assert_eq!(
            outcome.trace.action_class,
            AiActionClass::GuardedPreparation
        );
        assert_eq!(outcome.trace.capability, AiCapability::RunbookGenerate);

        let records = sink.snapshot();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].final_disposition, InvocationDisposition::Denied);

        std::env::remove_var("AOXC_AI_DISABLE");
    }
}