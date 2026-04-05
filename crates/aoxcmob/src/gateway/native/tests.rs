use super::*;
use aoxcore::identity::{MASTER_SEED_LEN, hd_path::HdPath};
use crate::{
    AoxcMobileTransport, ChainHealth, DevicePlatform, DeviceProfile, MobError, MobileConfig,
    SessionPermit, SignedTaskReceipt, TaskDescriptor, TaskSubmissionResult,
};
use crate::security::keystore::SecureStore;
use crate::security::signer::verify_json_payload;
use crate::session::protocol::SessionSigningPayload;
use crate::session::protocol::{SessionChallenge, SessionEnvelope};
use crate::transport::mock::MockRelayTransport;
use crate::types::{TaskKind, WitnessDecision};
use crate::util::now_epoch_secs;

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
        .transport_ref()
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
        .store_ref()
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
        .transport_ref()
        .last_receipt()
        .expect("mock transport should retain receipt")
        .expect("signed receipt should be stored");
    let signing_key = gateway
        .store_ref()
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
