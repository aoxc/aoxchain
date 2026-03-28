#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedIntent {
    pub id: &'static str,
    pub action: &'static str,
    pub dry_run_supported: bool,
    pub approval_required: bool,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub actor: &'static str,
    pub action: &'static str,
    pub target: &'static str,
    pub outcome: &'static str,
    pub source: &'static str,
}

pub fn governance_intents() -> Vec<SignedIntent> {
    vec![
        SignedIntent {
            id: "intent-upgrade-0142",
            action: "Upgrade intent queued for release window",
            dry_run_supported: true,
            approval_required: true,
            source: "governance_service.intent_queue",
        },
        SignedIntent {
            id: "intent-param-0091",
            action: "Protocol parameter proposal: block gas soft cap",
            dry_run_supported: true,
            approval_required: true,
            source: "governance_service.proposals",
        },
    ]
}

pub fn latest_audit_events() -> Vec<AuditEvent> {
    vec![
        AuditEvent {
            actor: "operator/root@aoxhub",
            action: "dry-run treasury transfer",
            target: "treasury/hot-wallet-1",
            outcome: "approved",
            source: "audit_service.events",
        },
        AuditEvent {
            actor: "operator/validator-team",
            action: "validator join intent",
            target: "validator/aoxc-val-17",
            outcome: "pending approval",
            source: "audit_service.events",
        },
    ]
}
