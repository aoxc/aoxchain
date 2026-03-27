// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::constitutional::ConstitutionalSeal;
use crate::kernel::ConsensusEvent;
use crate::safety::LockState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedConsensusEvent {
    pub sequence: u64,
    pub event_hash: [u8; 32],
    pub event: ConsensusEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelSnapshot {
    pub snapshot_height: u64,
    pub snapshot_round: u64,
    pub lock_state: LockState,
    pub finalized_seal: Option<ConstitutionalSeal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusEvidence {
    pub evidence_hash: [u8; 32],
    pub related_block_hash: [u8; 32],
    pub reason: String,
}

pub trait ConsensusJournal {
    fn append(&mut self, event: PersistedConsensusEvent) -> Result<(), String>;
    fn load_all(&self) -> Result<Vec<PersistedConsensusEvent>, String>;
}

pub trait SnapshotStore {
    fn store_snapshot(&mut self, snapshot: KernelSnapshot) -> Result<(), String>;
    fn load_snapshot(&self) -> Result<Option<KernelSnapshot>, String>;
}

pub trait EvidenceStore {
    fn append_evidence(&mut self, evidence: ConsensusEvidence) -> Result<(), String>;
    fn load_evidence(&self) -> Result<Vec<ConsensusEvidence>, String>;
}

pub trait FinalityStore {
    fn store_finalized_seal(&mut self, seal: ConstitutionalSeal) -> Result<(), String>;
    fn load_finalized_seal(&self) -> Result<Option<ConstitutionalSeal>, String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryState {
    pub snapshot: Option<KernelSnapshot>,
    pub journal: Vec<PersistedConsensusEvent>,
    pub evidence: Vec<ConsensusEvidence>,
    pub finalized_seal: Option<ConstitutionalSeal>,
}

pub fn recover_state<J, S, E, F>(
    journal: &J,
    snapshots: &S,
    evidence: &E,
    finality: &F,
) -> Result<RecoveryState, String>
where
    J: ConsensusJournal,
    S: SnapshotStore,
    E: EvidenceStore,
    F: FinalityStore,
{
    Ok(RecoveryState {
        snapshot: snapshots.load_snapshot()?,
        journal: journal.load_all()?,
        evidence: evidence.load_evidence()?,
        finalized_seal: finality.load_finalized_seal()?,
    })
}

#[cfg(test)]
mod tests {
    use crate::constitutional::{
        ConstitutionalSeal, ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
    };
    use crate::kernel::{ConsensusEvent, VerifiedVote};
    use crate::seal::QuorumCertificate;
    use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

    use super::{
        ConsensusEvidence, ConsensusJournal, EvidenceStore, FinalityStore, KernelSnapshot,
        PersistedConsensusEvent, RecoveryState, SnapshotStore, recover_state,
    };

    #[derive(Default)]
    struct MemoryStore {
        events: Vec<PersistedConsensusEvent>,
        snapshot: Option<KernelSnapshot>,
        evidence: Vec<ConsensusEvidence>,
        finality: Option<ConstitutionalSeal>,
    }

    impl ConsensusJournal for MemoryStore {
        fn append(&mut self, event: PersistedConsensusEvent) -> Result<(), String> {
            self.events.push(event);
            Ok(())
        }

        fn load_all(&self) -> Result<Vec<PersistedConsensusEvent>, String> {
            Ok(self.events.clone())
        }
    }

    impl SnapshotStore for MemoryStore {
        fn store_snapshot(&mut self, snapshot: KernelSnapshot) -> Result<(), String> {
            self.snapshot = Some(snapshot);
            Ok(())
        }

        fn load_snapshot(&self) -> Result<Option<KernelSnapshot>, String> {
            Ok(self.snapshot.clone())
        }
    }

    impl EvidenceStore for MemoryStore {
        fn append_evidence(&mut self, evidence: ConsensusEvidence) -> Result<(), String> {
            self.evidence.push(evidence);
            Ok(())
        }

        fn load_evidence(&self) -> Result<Vec<ConsensusEvidence>, String> {
            Ok(self.evidence.clone())
        }
    }

    impl FinalityStore for MemoryStore {
        fn store_finalized_seal(&mut self, seal: ConstitutionalSeal) -> Result<(), String> {
            self.finality = Some(seal);
            Ok(())
        }

        fn load_finalized_seal(&self) -> Result<Option<ConstitutionalSeal>, String> {
            Ok(self.finality.clone())
        }
    }

    #[test]
    fn recovery_state_loads_snapshot_journal_evidence_and_finality() {
        let mut store = MemoryStore::default();
        store
            .append(PersistedConsensusEvent {
                sequence: 1,
                event_hash: [1u8; 32],
                event: ConsensusEvent::AdmitVerifiedVote(VerifiedVote {
                    authenticated_vote: VerifiedAuthenticatedVote {
                        vote: Vote {
                            voter: [7u8; 32],
                            block_hash: [8u8; 32],
                            height: 2,
                            round: 3,
                            kind: VoteKind::Commit,
                        },
                        context: VoteAuthenticationContext {
                            network_id: 2626,
                            epoch: 0,
                            validator_set_root: [6u8; 32],
                            signature_scheme: 1,
                        },
                    },
                    verification_tag: [9u8; 32],
                }),
            })
            .unwrap();
        store
            .store_snapshot(KernelSnapshot {
                snapshot_height: 2,
                snapshot_round: 3,
                lock_state: Default::default(),
                finalized_seal: None,
            })
            .unwrap();
        store
            .append_evidence(ConsensusEvidence {
                evidence_hash: [2u8; 32],
                related_block_hash: [8u8; 32],
                reason: "equivocation".to_string(),
            })
            .unwrap();

        let qc = QuorumCertificate::new([8u8; 32], 2, 3, vec![[7u8; 32]], 1, 1, 1, 1);
        let execution = ExecutionCertificate::new(4, [3u8; 32], qc);
        let legitimacy = LegitimacyCertificate::new(
            [8u8; 32],
            4,
            [4u8; 32],
            [5u8; 32],
            [6u8; 32],
            vec![[7u8; 32]],
        );
        let continuity = ContinuityCertificate::new([8u8; 32], 2, 3, 4, 4, 1, vec![[7u8; 32]]);
        let seal = ConstitutionalSeal::compose(&execution, &legitimacy, &continuity).unwrap();
        store.store_finalized_seal(seal.clone()).unwrap();

        let recovered = recover_state(&store, &store, &store, &store).unwrap();
        assert_eq!(recovered.journal.len(), 1);
        assert_eq!(recovered.evidence.len(), 1);
        assert_eq!(recovered.finalized_seal, Some(seal));
    }

    #[test]
    fn recovery_state_is_deterministic_for_same_storage_view() {
        let store = MemoryStore::default();
        let a: RecoveryState = recover_state(&store, &store, &store, &store).unwrap();
        let b: RecoveryState = recover_state(&store, &store, &store, &store).unwrap();
        assert_eq!(a, b);
    }
}
