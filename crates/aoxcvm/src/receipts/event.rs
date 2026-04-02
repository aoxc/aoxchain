//! Structured constitutional receipt events.

use crate::{
    receipts::outcome::ExecutionReceipt,
    vm::constitutional_runtime::{ConstitutionalProvenance, RuntimeSurface},
};

/// Structured event family for constitutional observability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstitutionalEventKind {
    CapabilityAuthorized,
    CapabilityDenied,
    GovernanceAction,
    PackageLifecycle,
    UpgradeLifecycle,
}

/// Canonical event payload linked to constitutional provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionalEvent {
    pub kind: ConstitutionalEventKind,
    pub surface: RuntimeSurface,
    pub message: String,
    pub signer_class: &'static str,
    pub governance_lane: &'static str,
    pub auth_profile_id: Option<u32>,
    pub auth_profile_version: Option<u16>,
}

impl ConstitutionalEvent {
    pub fn from_provenance(
        kind: ConstitutionalEventKind,
        message: impl Into<String>,
        provenance: &ConstitutionalProvenance,
    ) -> Self {
        Self {
            kind,
            surface: provenance.surface,
            message: message.into(),
            signer_class: provenance.signer_class.wire_id(),
            governance_lane: match provenance.governance_lane {
                crate::policy::governance::GovernanceLane::Constitutional => "constitutional",
                crate::policy::governance::GovernanceLane::Operations => "operations",
                crate::policy::governance::GovernanceLane::Emergency => "emergency",
            },
            auth_profile_id: provenance.auth_profile_id,
            auth_profile_version: provenance.auth_profile_version,
        }
    }
}

/// Audit-grade receipt that carries constitutional event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionalReceipt {
    pub execution: ExecutionReceipt,
    pub events: Vec<ConstitutionalEvent>,
}

impl ConstitutionalReceipt {
    pub fn new(execution: ExecutionReceipt) -> Self {
        Self {
            execution,
            events: Vec::new(),
        }
    }

    pub fn push_event(&mut self, event: ConstitutionalEvent) {
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::signer::SignerClass,
        policy::governance::{GovernanceAction, GovernanceLane},
        receipts::{
            event::{ConstitutionalEvent, ConstitutionalEventKind, ConstitutionalReceipt},
            outcome::{ExecutionReceipt, ReceiptStatus},
        },
        state::JournaledState,
        vm::constitutional_runtime::{ConstitutionalProvenance, RuntimeSurface},
    };

    #[test]
    fn constitutional_event_derives_wire_metadata_from_provenance() {
        let provenance = ConstitutionalProvenance {
            surface: RuntimeSurface::GovernanceAction(GovernanceAction::UpgradeProtocol),
            signer_class: SignerClass::Governance,
            governance_lane: GovernanceLane::Constitutional,
            auth_profile_id: Some(11),
            auth_profile_version: Some(2),
        };

        let event = ConstitutionalEvent::from_provenance(
            ConstitutionalEventKind::GovernanceAction,
            "upgrade authorized",
            &provenance,
        );

        assert_eq!(event.signer_class, "governance");
        assert_eq!(event.governance_lane, "constitutional");
        assert_eq!(event.auth_profile_id, Some(11));
    }

    #[test]
    fn constitutional_receipt_collects_events() {
        let mut state = JournaledState::default();
        state.put(b"k".to_vec(), b"v".to_vec());
        let receipt = ExecutionReceipt::from_state(ReceiptStatus::Success, 100, vec![], &state);

        let mut constitutional = ConstitutionalReceipt::new(receipt);
        constitutional.push_event(ConstitutionalEvent {
            kind: ConstitutionalEventKind::CapabilityAuthorized,
            surface: RuntimeSurface::UpgradeTrigger,
            message: "upgrade trigger authorized".to_string(),
            signer_class: "governance",
            governance_lane: "constitutional",
            auth_profile_id: Some(3),
            auth_profile_version: Some(1),
        });

        assert_eq!(constitutional.events.len(), 1);
    }
}
