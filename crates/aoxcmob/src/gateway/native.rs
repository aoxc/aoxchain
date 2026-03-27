use crate::config::MobileConfig;
use crate::error::MobError;
use crate::security::keystore::SecureStore;
use crate::security::signer::{public_key_fingerprint, public_key_hex, sign_json_payload};
use crate::session::protocol::{
    SessionChallenge, SessionContext, SessionEnvelope, SessionPermit, SessionSigningPayload,
};
use crate::transport::api::{AoxcMobileTransport, TaskSubmissionResult};
use crate::types::{
    ChainHealth, DevicePlatform, DeviceProfile, SignedTaskReceipt, TaskDescriptor, TaskReceipt,
    WitnessDecision,
};
use crate::util::{now_epoch_secs, prefixed_id};
use aoxcore::identity::hd_path::HdPath;
use aoxcore::identity::key_engine::{KeyEngine, MASTER_SEED_LEN};
use ed25519_dalek::SigningKey;
use std::cmp::Ordering as CmpOrdering;
use std::sync::atomic::{AtomicU64, Ordering};

/// High-level mobile gateway for secure native AOXC participation flows.
pub struct NativeGateway<T, S> {
    config: MobileConfig,
    transport: T,
    store: S,
    client_nonce: AtomicU64,
}

impl<T, S> NativeGateway<T, S>
where
    T: AoxcMobileTransport,
    S: SecureStore,
{
    /// Creates a new gateway after validating runtime policy.
    pub fn new(config: MobileConfig, transport: T, store: S) -> Result<Self, MobError> {
        config.validate()?;
        Ok(Self {
            config,
            transport,
            store,
            client_nonce: AtomicU64::new(1),
        })
    }

    /// Returns the immutable mobile runtime policy.
    #[must_use]
    pub fn config(&self) -> &MobileConfig {
        &self.config
    }

    /// Returns true when the device security boundary already holds a provisioned key.
    pub fn is_device_provisioned(&self) -> Result<bool, MobError> {
        self.store.is_provisioned()
    }

    /// Deterministically provisions a device signing key from AOXC master seed material.
    pub fn provision_from_master_seed(
        &self,
        master_seed: [u8; MASTER_SEED_LEN],
        hd_path: HdPath,
        platform: DevicePlatform,
        device_label: impl Into<String>,
    ) -> Result<DeviceProfile, MobError> {
        let device_label = device_label.into();
        if device_label.trim().is_empty() {
            return Err(MobError::InvalidInput("device_label must not be empty"));
        }

        let key_engine = KeyEngine::from_seed(master_seed);
        let key_material = key_engine
            .derive_key_material(&hd_path)
            .map_err(|error| MobError::Crypto(error.to_string()))?;

        let mut secret = [0u8; 32];
        secret.copy_from_slice(&key_material[..32]);
        let signing_key = SigningKey::from_bytes(&secret);
        self.bind_signing_key(
            signing_key,
            Some(hd_path.to_string()),
            platform,
            device_label,
        )
    }

    /// Binds an already-prepared signing key to the secure store.
    pub fn bind_signing_key(
        &self,
        signing_key: SigningKey,
        hd_path: Option<String>,
        platform: DevicePlatform,
        device_label: impl Into<String>,
    ) -> Result<DeviceProfile, MobError> {
        let device_label = device_label.into();
        if device_label.trim().is_empty() {
            return Err(MobError::InvalidInput("device_label must not be empty"));
        }

        let verifying_key = signing_key.verifying_key();
        let public_key_hex = public_key_hex(&signing_key);
        let public_key_fingerprint = public_key_fingerprint(&verifying_key);
        let created_at_epoch_secs = now_epoch_secs()?;
        let app_installation_id = prefixed_id(
            "INSTALL",
            &[
                self.config.app_id.as_bytes(),
                public_key_hex.as_bytes(),
                device_label.as_bytes(),
            ],
        );
        let device_id = prefixed_id(
            "DEVICE",
            &[
                self.config.chain_id.as_bytes(),
                self.config.app_id.as_bytes(),
                public_key_hex.as_bytes(),
            ],
        );

        let profile = DeviceProfile {
            device_id,
            device_label,
            platform,
            public_key_hex,
            public_key_fingerprint,
            hd_path,
            app_installation_id,
            created_at_epoch_secs,
        };

        self.store.store_device(profile.clone(), signing_key)?;
        Ok(profile)
    }

    /// Opens a short-lived signed mobile session.
    pub async fn open_session(&self) -> Result<SessionContext, MobError> {
        let profile = self.store.load_device_profile()?;
        let signing_key = self.store.load_signing_key()?;
        let challenge = self
            .transport
            .request_session_challenge(&profile, &self.config)
            .await?;
        self.validate_challenge(&challenge)?;

        let client_timestamp_epoch_secs = now_epoch_secs()?;
        let payload = SessionSigningPayload {
            challenge_id: challenge.challenge_id.clone(),
            relay_nonce: challenge.relay_nonce.clone(),
            device_id: profile.device_id.clone(),
            app_id: self.config.app_id.clone(),
            chain_id: self.config.chain_id.clone(),
            client_nonce: self.client_nonce.fetch_add(1, Ordering::SeqCst),
            client_timestamp_epoch_secs,
            public_key_hex: profile.public_key_hex.clone(),
        };
        let (signature_hex, payload_hash_hex) = sign_json_payload(&signing_key, &payload)?;
        let envelope = SessionEnvelope {
            challenge_id: payload.challenge_id,
            relay_nonce: payload.relay_nonce,
            device_id: payload.device_id,
            app_id: payload.app_id,
            chain_id: payload.chain_id,
            client_nonce: payload.client_nonce,
            client_timestamp_epoch_secs: payload.client_timestamp_epoch_secs,
            public_key_hex: payload.public_key_hex,
            payload_hash_hex,
            signature_hex,
        };
        let permit = self
            .transport
            .submit_session_envelope(envelope, &self.config)
            .await?;
        self.validate_permit(&permit, &profile)?;
        Ok(SessionContext { profile, permit })
    }

    /// Fetches lightweight chain health using an existing permit.
    pub async fn fetch_chain_health(
        &self,
        permit: &SessionPermit,
    ) -> Result<ChainHealth, MobError> {
        self.ensure_permit_active(permit)?;
        self.transport
            .fetch_chain_health(permit, &self.config)
            .await
    }

    /// Fetches lightweight mobile tasks for the active session.
    pub async fn fetch_available_tasks(
        &self,
        permit: &SessionPermit,
    ) -> Result<Vec<TaskDescriptor>, MobError> {
        self.ensure_permit_active(permit)?;
        self.transport
            .fetch_available_tasks(permit, &self.config)
            .await
    }

    /// Signs and submits a witness decision for a task.
    pub async fn submit_witness_decision(
        &self,
        permit: &SessionPermit,
        task_id: impl Into<String>,
        decision: WitnessDecision,
    ) -> Result<TaskSubmissionResult, MobError> {
        self.ensure_permit_active(permit)?;
        let task_id = task_id.into();
        if task_id.trim().is_empty() {
            return Err(MobError::InvalidInput("task_id must not be empty"));
        }
        if task_id.len() > 128
            || !task_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            return Err(MobError::InvalidInput(
                "task_id must be <=128 chars and use [A-Za-z0-9_-]",
            ));
        }
        let profile = self.store.load_device_profile()?;
        let signing_key = self.store.load_signing_key()?;
        let receipt = TaskReceipt {
            task_id,
            decision,
            client_timestamp_epoch_secs: now_epoch_secs()?,
            device_id: profile.device_id,
            session_id: permit.session_id.clone(),
        };
        let (signature_hex, payload_hash_hex) = sign_json_payload(&signing_key, &receipt)?;
        let signed = SignedTaskReceipt {
            receipt,
            signature_hex,
            payload_hash_hex,
            public_key_hex: profile.public_key_hex,
        };
        self.transport
            .submit_task_receipt(signed, &self.config)
            .await
    }

    fn validate_challenge(&self, challenge: &SessionChallenge) -> Result<(), MobError> {
        let now = now_epoch_secs()?;
        if challenge.challenge_id.trim().is_empty() {
            return Err(MobError::InvalidSessionChallenge(
                "challenge_id must not be empty",
            ));
        }
        if challenge.relay_nonce.trim().is_empty() {
            return Err(MobError::InvalidSessionChallenge(
                "relay_nonce must not be empty",
            ));
        }
        if challenge.audience != self.config.app_id {
            return Err(MobError::InvalidSessionChallenge(
                "challenge audience does not match app_id",
            ));
        }
        if challenge.expires_at_epoch_secs <= now {
            return Err(MobError::InvalidSessionChallenge(
                "challenge is already expired",
            ));
        }
        if challenge.issued_at_epoch_secs > challenge.expires_at_epoch_secs {
            return Err(MobError::InvalidSessionChallenge(
                "challenge issued_at must be <= expires_at",
            ));
        }
        if now.saturating_sub(challenge.issued_at_epoch_secs) > self.config.challenge_max_skew_secs
        {
            return Err(MobError::InvalidSessionChallenge(
                "challenge issued_at is older than allowed skew",
            ));
        }
        if challenge.issued_at_epoch_secs > now + self.config.challenge_max_skew_secs {
            return Err(MobError::InvalidSessionChallenge(
                "challenge issued_at is unreasonably far in the future",
            ));
        }
        if challenge.session_ttl_secs == 0 {
            return Err(MobError::InvalidSessionChallenge(
                "challenge session_ttl_secs must be greater than zero",
            ));
        }
        Ok(())
    }

    fn validate_permit(
        &self,
        permit: &SessionPermit,
        profile: &DeviceProfile,
    ) -> Result<(), MobError> {
        if permit.session_id.trim().is_empty() {
            return Err(MobError::InvalidSessionChallenge(
                "session_id must not be empty",
            ));
        }
        if permit.device_id != profile.device_id {
            return Err(MobError::InvalidSessionChallenge(
                "session permit device_id does not match local profile",
            ));
        }
        if permit.issued_at_epoch_secs > permit.expires_at_epoch_secs {
            return Err(MobError::InvalidSessionChallenge(
                "session permit issued_at must be <= expires_at",
            ));
        }
        if permit
            .expires_at_epoch_secs
            .saturating_sub(permit.issued_at_epoch_secs)
            .cmp(&self.config.session_ttl_secs)
            == CmpOrdering::Greater
        {
            return Err(MobError::InvalidSessionChallenge(
                "session permit ttl exceeds configured session_ttl_secs",
            ));
        }
        self.ensure_permit_active(permit)
    }

    fn ensure_permit_active(&self, permit: &SessionPermit) -> Result<(), MobError> {
        let now = now_epoch_secs()?;
        if permit.expires_at_epoch_secs <= now {
            return Err(MobError::SessionExpired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::signer::verify_json_payload;
    use crate::session::protocol::SessionSigningPayload;
    use crate::transport::mock::MockRelayTransport;
    use crate::types::{TaskKind, WitnessDecision};

    fn sample_seed() -> [u8; MASTER_SEED_LEN] {
        [0x44; MASTER_SEED_LEN]
    }

    #[test]
    fn deterministic_device_provisioning_is_stable_for_same_seed_and_path() {
        let config = MobileConfig::default();
        let gateway_a = NativeGateway::new(
            config.clone(),
            MockRelayTransport::new(config.chain_id.clone()),
            crate::InMemorySecureStore::new(),
        )
        .expect("gateway creation must succeed");
        let gateway_b = NativeGateway::new(
            config,
            MockRelayTransport::new("AOXC-MAIN"),
            crate::InMemorySecureStore::new(),
        )
        .expect("gateway creation must succeed");

        let profile_a = gateway_a
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Android,
                "guardian-device",
            )
            .expect("device provisioning must succeed");
        let profile_b = gateway_b
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Android,
                "guardian-device",
            )
            .expect("device provisioning must succeed");

        assert_eq!(profile_a.device_id, profile_b.device_id);
        assert_eq!(profile_a.public_key_hex, profile_b.public_key_hex);
        assert_eq!(
            profile_a.public_key_fingerprint,
            profile_b.public_key_fingerprint
        );
    }

    #[tokio::test]
    async fn open_session_emits_verifiable_signed_envelope() {
        let config = MobileConfig::default();
        let transport = MockRelayTransport::new(config.chain_id.clone());
        let gateway =
            NativeGateway::new(config.clone(), transport, crate::InMemorySecureStore::new())
                .expect("gateway creation must succeed");
        let profile = gateway
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Desktop,
                "guardian-desktop",
            )
            .expect("device provisioning must succeed");

        let session = gateway
            .open_session()
            .await
            .expect("session open must succeed");
        let envelope = gateway
            .transport
            .last_session_envelope()
            .expect("mock transport should retain envelope")
            .expect("session envelope should be stored");

        let payload = SessionSigningPayload {
            challenge_id: envelope.challenge_id.clone(),
            relay_nonce: envelope.relay_nonce.clone(),
            device_id: envelope.device_id.clone(),
            app_id: envelope.app_id.clone(),
            chain_id: envelope.chain_id.clone(),
            client_nonce: envelope.client_nonce,
            client_timestamp_epoch_secs: envelope.client_timestamp_epoch_secs,
            public_key_hex: envelope.public_key_hex.clone(),
        };

        let signing_key = gateway
            .store
            .load_signing_key()
            .expect("signing key should load");
        verify_json_payload(
            &signing_key.verifying_key(),
            &payload,
            &envelope.signature_hex,
        )
        .expect("session envelope signature should verify");
        assert_eq!(session.profile.device_id, profile.device_id);
        assert_eq!(session.permit.device_id, profile.device_id);
    }

    #[tokio::test]
    async fn witness_submission_emits_verifiable_signed_receipt() {
        let config = MobileConfig::default();
        let transport = MockRelayTransport::new(config.chain_id.clone());
        let gateway = NativeGateway::new(config, transport, crate::InMemorySecureStore::new())
            .expect("gateway creation must succeed");
        gateway
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Android,
                "guardian-android",
            )
            .expect("device provisioning must succeed");

        let session = gateway
            .open_session()
            .await
            .expect("session open must succeed");
        gateway
            .submit_witness_decision(&session.permit, "TASK-VERIFY", WitnessDecision::Approve)
            .await
            .expect("task submission must succeed");

        let signed = gateway
            .transport
            .last_receipt()
            .expect("mock transport should retain receipt")
            .expect("signed receipt should be stored");
        let signing_key = gateway
            .store
            .load_signing_key()
            .expect("signing key should load");
        verify_json_payload(
            &signing_key.verifying_key(),
            &signed.receipt,
            &signed.signature_hex,
        )
        .expect("signed task receipt should verify");
        assert_eq!(signed.receipt.task_id, "TASK-VERIFY");
    }

    #[tokio::test]
    async fn session_open_fetch_health_and_tasks_succeeds() {
        let config = MobileConfig::default();
        let transport = MockRelayTransport::new(config.chain_id.clone());
        transport
            .push_task(TaskDescriptor {
                task_id: "TASK-1".to_string(),
                kind: TaskKind::SecurityWitness,
                title: "Confirm emergency policy action".to_string(),
                detail: "Review and confirm the security witness action.".to_string(),
                reward_units: 25,
                expires_at_epoch_secs: now_epoch_secs().expect("time must be available") + 600,
                required_session: true,
            })
            .expect("task push must succeed");

        let gateway = NativeGateway::new(config, transport, crate::InMemorySecureStore::new())
            .expect("gateway creation must succeed");
        gateway
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Ios,
                "guardian-ios",
            )
            .expect("device provisioning must succeed");

        let session = gateway
            .open_session()
            .await
            .expect("session open must succeed");
        let health = gateway
            .fetch_chain_health(&session.permit)
            .await
            .expect("health fetch must succeed");
        let tasks = gateway
            .fetch_available_tasks(&session.permit)
            .await
            .expect("task fetch must succeed");

        assert!(health.healthy);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_id, "TASK-1");
    }

    #[tokio::test]
    async fn witness_decision_is_signed_and_submitted() {
        let config = MobileConfig::default();
        let transport = MockRelayTransport::new(config.chain_id.clone());
        let gateway = NativeGateway::new(config, transport, crate::InMemorySecureStore::new())
            .expect("gateway creation must succeed");
        gateway
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Android,
                "guardian-android",
            )
            .expect("device provisioning must succeed");
        let session = gateway
            .open_session()
            .await
            .expect("session open must succeed");
        let result = gateway
            .submit_witness_decision(&session.permit, "TASK-ALERT-1", WitnessDecision::Approve)
            .await
            .expect("task submission must succeed");

        assert!(result.accepted);
        assert_eq!(result.reward_units, 25);
    }

    #[tokio::test]
    async fn submit_witness_decision_rejects_invalid_task_id() {
        let config = MobileConfig::default();
        let transport = MockRelayTransport::new(config.chain_id.clone());
        let gateway = NativeGateway::new(config, transport, crate::InMemorySecureStore::new())
            .expect("gateway creation must succeed");
        gateway
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Android,
                "guardian-android",
            )
            .expect("device provisioning must succeed");
        let session = gateway
            .open_session()
            .await
            .expect("session open must succeed");

        let error = gateway
            .submit_witness_decision(&session.permit, "task invalid", WitnessDecision::Approve)
            .await
            .expect_err("invalid task_id must be rejected");
        assert_eq!(error.code(), "AOXCMOB_INPUT_INVALID");
    }

    #[tokio::test]
    async fn open_session_rejects_replay_like_old_challenge() {
        struct ReplayChallengeTransport;

        #[async_trait::async_trait]
        impl AoxcMobileTransport for ReplayChallengeTransport {
            async fn request_session_challenge(
                &self,
                _profile: &DeviceProfile,
                config: &MobileConfig,
            ) -> Result<SessionChallenge, MobError> {
                let now = now_epoch_secs()?;
                Ok(SessionChallenge {
                    challenge_id: "CH-OLD".to_string(),
                    relay_nonce: "NONCE-OLD".to_string(),
                    issued_at_epoch_secs: now.saturating_sub(config.challenge_max_skew_secs + 5),
                    expires_at_epoch_secs: now + 10,
                    audience: config.app_id.clone(),
                    session_ttl_secs: config.session_ttl_secs,
                })
            }

            async fn submit_session_envelope(
                &self,
                _envelope: SessionEnvelope,
                _config: &MobileConfig,
            ) -> Result<SessionPermit, MobError> {
                unreachable!("challenge should be rejected before envelope submission")
            }

            async fn fetch_chain_health(
                &self,
                _permit: &SessionPermit,
                _config: &MobileConfig,
            ) -> Result<ChainHealth, MobError> {
                unreachable!()
            }

            async fn fetch_available_tasks(
                &self,
                _permit: &SessionPermit,
                _config: &MobileConfig,
            ) -> Result<Vec<TaskDescriptor>, MobError> {
                unreachable!()
            }

            async fn submit_task_receipt(
                &self,
                _receipt: SignedTaskReceipt,
                _config: &MobileConfig,
            ) -> Result<TaskSubmissionResult, MobError> {
                unreachable!()
            }
        }

        let config = MobileConfig::default();
        let gateway = NativeGateway::new(
            config,
            ReplayChallengeTransport,
            crate::InMemorySecureStore::new(),
        )
        .expect("gateway creation must succeed");
        gateway
            .provision_from_master_seed(
                sample_seed(),
                HdPath::new(1, 100, 1, 0).expect("path must be valid"),
                DevicePlatform::Desktop,
                "guardian-desktop",
            )
            .expect("device provisioning must succeed");

        let error = gateway
            .open_session()
            .await
            .expect_err("replay-like old challenge must be rejected");
        assert_eq!(error.code(), "AOXCMOB_SESSION_CHALLENGE_INVALID");
    }
}
