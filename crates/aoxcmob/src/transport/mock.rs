use crate::config::MobileConfig;
use crate::error::MobError;
use crate::session::protocol::{SessionChallenge, SessionEnvelope, SessionPermit};
use crate::transport::api::{AoxcMobileTransport, TaskSubmissionResult};
use crate::types::{ChainHealth, DeviceProfile, SignedTaskReceipt, TaskDescriptor};
use crate::util::{now_epoch_secs, prefixed_id, sha3_hex_upper};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// Deterministic in-memory relay transport used for tests and local development.
#[derive(Debug)]
pub struct MockRelayTransport {
    chain_id: String,
    sequence: AtomicU64,
    tasks: Mutex<Vec<TaskDescriptor>>,
    sessions: Mutex<HashMap<String, String>>,
    last_session_envelope: Mutex<Option<SessionEnvelope>>,
    last_receipt: Mutex<Option<SignedTaskReceipt>>,
}

impl MockRelayTransport {
    #[must_use]
    pub fn new(chain_id: impl Into<String>) -> Self {
        Self {
            chain_id: chain_id.into(),
            sequence: AtomicU64::new(1),
            tasks: Mutex::new(Vec::new()),
            sessions: Mutex::new(HashMap::new()),
            last_session_envelope: Mutex::new(None),
            last_receipt: Mutex::new(None),
        }
    }

    pub fn push_task(&self, task: TaskDescriptor) -> Result<(), MobError> {
        let mut guard = self
            .tasks
            .lock()
            .map_err(|_| MobError::Transport("mock task store lock poisoned".to_string()))?;
        guard.push(task);
        Ok(())
    }

    pub fn last_session_envelope(&self) -> Result<Option<SessionEnvelope>, MobError> {
        let guard = self
            .last_session_envelope
            .lock()
            .map_err(|_| MobError::Transport("mock envelope store lock poisoned".to_string()))?;
        Ok(guard.clone())
    }

    pub fn last_receipt(&self) -> Result<Option<SignedTaskReceipt>, MobError> {
        let guard = self
            .last_receipt
            .lock()
            .map_err(|_| MobError::Transport("mock receipt store lock poisoned".to_string()))?;
        Ok(guard.clone())
    }
}

#[async_trait]
impl AoxcMobileTransport for MockRelayTransport {
    async fn request_session_challenge(
        &self,
        profile: &DeviceProfile,
        config: &MobileConfig,
    ) -> Result<SessionChallenge, MobError> {
        let now = now_epoch_secs()?;
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        Ok(SessionChallenge {
            challenge_id: format!("CH-{}", sequence),
            relay_nonce: prefixed_id(
                "NONCE",
                &[
                    profile.device_id.as_bytes(),
                    sequence.to_string().as_bytes(),
                ],
            ),
            issued_at_epoch_secs: now,
            expires_at_epoch_secs: now + config.challenge_max_skew_secs,
            audience: config.app_id.clone(),
            session_ttl_secs: config.session_ttl_secs,
        })
    }

    async fn submit_session_envelope(
        &self,
        envelope: SessionEnvelope,
        _config: &MobileConfig,
    ) -> Result<SessionPermit, MobError> {
        if envelope.device_id.trim().is_empty() {
            return Err(MobError::Transport(
                "device_id must not be empty".to_string(),
            ));
        }
        if envelope.signature_hex.trim().is_empty() {
            return Err(MobError::Transport(
                "signature_hex must not be empty".to_string(),
            ));
        }

        {
            let mut last_envelope = self.last_session_envelope.lock().map_err(|_| {
                MobError::Transport("mock envelope store lock poisoned".to_string())
            })?;
            *last_envelope = Some(envelope.clone());
        }

        let now = now_epoch_secs()?;
        let session_id = prefixed_id(
            "SESS",
            &[
                envelope.device_id.as_bytes(),
                envelope.challenge_id.as_bytes(),
                envelope.payload_hash_hex.as_bytes(),
            ],
        );
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| MobError::Transport("mock session store lock poisoned".to_string()))?;
        sessions.insert(session_id.clone(), envelope.device_id.clone());
        Ok(SessionPermit {
            session_id: session_id.clone(),
            device_id: envelope.device_id,
            issued_at_epoch_secs: now,
            expires_at_epoch_secs: now + 300,
            relay_signature_hint: sha3_hex_upper(session_id.as_bytes())[..24].to_string(),
        })
    }

    async fn fetch_chain_health(
        &self,
        permit: &SessionPermit,
        _config: &MobileConfig,
    ) -> Result<ChainHealth, MobError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| MobError::Transport("mock session store lock poisoned".to_string()))?;
        if !sessions.contains_key(&permit.session_id) {
            return Err(MobError::Transport("unknown session_id".to_string()));
        }
        Ok(ChainHealth {
            chain_id: self.chain_id.clone(),
            height: 1_000,
            peer_count: 7,
            error_rate: 0.0,
            healthy: true,
        })
    }

    async fn fetch_available_tasks(
        &self,
        permit: &SessionPermit,
        _config: &MobileConfig,
    ) -> Result<Vec<TaskDescriptor>, MobError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| MobError::Transport("mock session store lock poisoned".to_string()))?;
        if !sessions.contains_key(&permit.session_id) {
            return Err(MobError::Transport("unknown session_id".to_string()));
        }
        let tasks = self
            .tasks
            .lock()
            .map_err(|_| MobError::Transport("mock task store lock poisoned".to_string()))?;
        Ok(tasks.clone())
    }

    async fn submit_task_receipt(
        &self,
        receipt: SignedTaskReceipt,
        _config: &MobileConfig,
    ) -> Result<TaskSubmissionResult, MobError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| MobError::Transport("mock session store lock poisoned".to_string()))?;
        let Some(bound_device_id) = sessions.get(&receipt.receipt.session_id) else {
            return Err(MobError::Transport("unknown session_id".to_string()));
        };
        if bound_device_id != &receipt.receipt.device_id {
            return Err(MobError::Transport(
                "receipt device_id does not match session device binding".to_string(),
            ));
        }
        drop(sessions);

        let mut last_receipt = self
            .last_receipt
            .lock()
            .map_err(|_| MobError::Transport("mock receipt store lock poisoned".to_string()))?;
        *last_receipt = Some(receipt.clone());
        Ok(TaskSubmissionResult {
            accepted: true,
            reward_units: 25,
            receipt_id: prefixed_id(
                "RCPT",
                &[
                    receipt.receipt.task_id.as_bytes(),
                    receipt.receipt.device_id.as_bytes(),
                    receipt.payload_hash_hex.as_bytes(),
                ],
            ),
        })
    }
}
