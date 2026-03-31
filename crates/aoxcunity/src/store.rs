// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

/// Canonical durable storage layout for consensus crash-recovery artifacts.
#[derive(Debug, Clone)]
pub struct FileConsensusStore {
    journal_path: PathBuf,
    snapshot_path: PathBuf,
    evidence_path: PathBuf,
    finality_path: PathBuf,
}

impl FileConsensusStore {
    #[must_use]
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let base = base_dir.as_ref();
        Self {
            journal_path: base.join("consensus_journal.log"),
            snapshot_path: base.join("kernel_snapshot.json"),
            evidence_path: base.join("evidence_store.json"),
            finality_path: base.join("finality_store.json"),
        }
    }

    fn ensure_parent(path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn overwrite_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
        Self::ensure_parent(path)?;
        let temp_path = path.with_extension("tmp");
        let payload = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        fs::write(&temp_path, payload).map_err(|error| error.to_string())?;
        fs::rename(&temp_path, path).map_err(|error| error.to_string())
    }

    fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, String> {
        match fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| error.to_string()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
}

/// Stable hash for persisted events, used during replay validation.
pub fn hash_consensus_event(event: &ConsensusEvent) -> Result<[u8; 32], String> {
    let payload = serde_json::to_vec(event).map_err(|error| error.to_string())?;
    Ok(Sha256::digest(payload).into())
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

impl ConsensusJournal for FileConsensusStore {
    fn append(&mut self, event: PersistedConsensusEvent) -> Result<(), String> {
        Self::ensure_parent(&self.journal_path)?;
        let encoded = serde_json::to_string(&event).map_err(|error| error.to_string())?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.journal_path)
            .map_err(|error| error.to_string())?;
        file.write_all(encoded.as_bytes())
            .map_err(|error| error.to_string())?;
        file.write_all(b"\n").map_err(|error| error.to_string())
    }

    fn load_all(&self) -> Result<Vec<PersistedConsensusEvent>, String> {
        let file = match File::open(&self.journal_path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.to_string()),
        };

        let mut out = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|error| error.to_string())?;
            if line.trim().is_empty() {
                continue;
            }
            let event: PersistedConsensusEvent =
                serde_json::from_str(&line).map_err(|error| error.to_string())?;
            out.push(event);
        }
        out.sort_by_key(|entry| entry.sequence);
        Ok(out)
    }
}

impl SnapshotStore for FileConsensusStore {
    fn store_snapshot(&mut self, snapshot: KernelSnapshot) -> Result<(), String> {
        Self::overwrite_json(&self.snapshot_path, &snapshot)
    }

    fn load_snapshot(&self) -> Result<Option<KernelSnapshot>, String> {
        Self::read_json(&self.snapshot_path)
    }
}

impl EvidenceStore for FileConsensusStore {
    fn append_evidence(&mut self, evidence: ConsensusEvidence) -> Result<(), String> {
        let mut entries = self.load_evidence()?;
        entries.push(evidence);
        Self::overwrite_json(&self.evidence_path, &entries)
    }

    fn load_evidence(&self) -> Result<Vec<ConsensusEvidence>, String> {
        Ok(Self::read_json(&self.evidence_path)?.unwrap_or_default())
    }
}

impl FinalityStore for FileConsensusStore {
    fn store_finalized_seal(&mut self, seal: ConstitutionalSeal) -> Result<(), String> {
        Self::overwrite_json(&self.finality_path, &seal)
    }

    fn load_finalized_seal(&self) -> Result<Option<ConstitutionalSeal>, String> {
        Self::read_json(&self.finality_path)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::constitutional::{
        ConstitutionalSeal, ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
    };
    use crate::kernel::{ConsensusEvent, VerifiedVote};
    use crate::seal::QuorumCertificate;
    use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

    use super::{
        ConsensusEvidence, ConsensusJournal, EvidenceStore, FileConsensusStore, FinalityStore,
        KernelSnapshot, PersistedConsensusEvent, RecoveryState, SnapshotStore,
        hash_consensus_event, recover_state,
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
                            pq_attestation_root: [7u8; 32],
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

    #[test]
    fn file_consensus_store_round_trips_journal_and_snapshot() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("aoxcunity-store-{unique}"));
        let mut store = FileConsensusStore::new(&base);

        let event = ConsensusEvent::AdvanceRound {
            height: 4,
            round: 9,
        };

        let entry = PersistedConsensusEvent {
            sequence: 7,
            event_hash: hash_consensus_event(&event).unwrap(),
            event,
        };
        store.append(entry).unwrap();
        store
            .store_snapshot(KernelSnapshot {
                snapshot_height: 4,
                snapshot_round: 9,
                lock_state: Default::default(),
                finalized_seal: None,
            })
            .unwrap();

        let loaded_events = store.load_all().unwrap();
        assert_eq!(loaded_events.len(), 1);
        assert_eq!(loaded_events[0].sequence, 7);
        assert_eq!(store.load_snapshot().unwrap().unwrap().snapshot_round, 9);

        let _ = std::fs::remove_dir_all(base);
    }
}
