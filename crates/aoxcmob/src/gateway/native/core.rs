// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::config::MobileConfig;
use crate::error::MobError;
use crate::security::keystore::SecureStore;
use crate::security::signer::{
    public_key_fingerprint, public_key_hex, sign_json_payload, verify_json_payload,
};
use crate::session::protocol::{
    RelayPermitSigningPayload, SessionChallenge, SessionContext, SessionEnvelope, SessionPermit,
    SessionSigningPayload,
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
use ed25519_dalek::VerifyingKey;
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

    #[cfg(test)]
    pub(crate) fn transport_ref(&self) -> &T {
        &self.transport
    }

    #[cfg(test)]
    pub(crate) fn store_ref(&self) -> &S {
        &self.store
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
        self.verify_challenge_signature(challenge)?;
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
        self.verify_permit_signature(permit)?;
        self.ensure_permit_active(permit)
    }

    fn verify_permit_signature(&self, permit: &SessionPermit) -> Result<(), MobError> {
        let Some(relay_public_key_hex) = &self.config.relay_verifying_key_hex else {
            return Ok(());
        };
        let signature_hex =
            permit
                .relay_signature_hex
                .as_deref()
                .ok_or(MobError::InvalidSessionChallenge(
                    "session permit relay_signature_hex is required",
                ))?;
        let relay_public_key_bytes = hex::decode(relay_public_key_hex)
            .map_err(|_| MobError::InvalidConfiguration("relay_verifying_key_hex decode failed"))?;
        let relay_public_key_array: [u8; 32] = relay_public_key_bytes.try_into().map_err(|_| {
            MobError::InvalidConfiguration("relay_verifying_key_hex must be 32 bytes")
        })?;
        let verifying_key = VerifyingKey::from_bytes(&relay_public_key_array).map_err(|_| {
            MobError::InvalidConfiguration("relay_verifying_key_hex is not a valid ed25519 key")
        })?;
        let payload = RelayPermitSigningPayload {
            session_id: permit.session_id.clone(),
            device_id: permit.device_id.clone(),
            issued_at_epoch_secs: permit.issued_at_epoch_secs,
            expires_at_epoch_secs: permit.expires_at_epoch_secs,
            relay_signature_hint: permit.relay_signature_hint.clone(),
        };
        verify_json_payload(&verifying_key, &payload, signature_hex).map_err(|_| {
            MobError::InvalidSessionChallenge("session permit signature verification failed")
        })
    }

    fn ensure_permit_active(&self, permit: &SessionPermit) -> Result<(), MobError> {
        let now = now_epoch_secs()?;
        if permit.expires_at_epoch_secs <= now {
            return Err(MobError::SessionExpired);
        }
        Ok(())
    }
}
