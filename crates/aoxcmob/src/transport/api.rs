// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::config::MobileConfig;
use crate::error::MobError;
use crate::session::protocol::{SessionChallenge, SessionEnvelope, SessionPermit};
use crate::types::{ChainHealth, DeviceProfile, SignedTaskReceipt, TaskDescriptor};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Outcome returned after a signed mobile task receipt is submitted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskSubmissionResult {
    pub accepted: bool,
    pub reward_units: u64,
    pub receipt_id: String,
}

/// Relay / RPC boundary used by the mobile native gateway.
#[async_trait]
pub trait AoxcMobileTransport: Send + Sync {
    async fn request_session_challenge(
        &self,
        profile: &DeviceProfile,
        config: &MobileConfig,
    ) -> Result<SessionChallenge, MobError>;

    async fn submit_session_envelope(
        &self,
        envelope: SessionEnvelope,
        config: &MobileConfig,
    ) -> Result<SessionPermit, MobError>;

    async fn fetch_chain_health(
        &self,
        permit: &SessionPermit,
        config: &MobileConfig,
    ) -> Result<ChainHealth, MobError>;

    async fn fetch_available_tasks(
        &self,
        permit: &SessionPermit,
        config: &MobileConfig,
    ) -> Result<Vec<TaskDescriptor>, MobError>;

    async fn submit_task_receipt(
        &self,
        receipt: SignedTaskReceipt,
        config: &MobileConfig,
    ) -> Result<TaskSubmissionResult, MobError>;
}
