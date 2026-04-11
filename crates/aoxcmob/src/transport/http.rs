// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::config::MobileConfig;
use crate::error::MobError;
use crate::session::protocol::{SessionChallenge, SessionEnvelope, SessionPermit};
use crate::transport::api::{AoxcMobileTransport, TaskSubmissionResult};
use crate::types::{ChainHealth, DeviceProfile, SignedTaskReceipt, TaskDescriptor};
use async_trait::async_trait;
use reqwest::{Client, StatusCode, Url};
use serde::Serialize;
use std::time::Duration;

/// Production-oriented HTTP relay transport for mobile gateway integration.
#[derive(Debug, Clone)]
pub struct HttpRelayTransport {
    client: Client,
}

impl HttpRelayTransport {
    /// Constructs an HTTP transport with sane timeout defaults.
    pub fn new(config: &MobileConfig) -> Result<Self, MobError> {
        config.validate()?;
        let client = Client::builder()
            .timeout(Duration::from_millis(config.request_timeout_ms))
            .build()
            .map_err(|error| MobError::Transport(format!("http client init failed: {}", error)))?;
        Ok(Self { client })
    }

    async fn post_json<B: Serialize, R: serde::de::DeserializeOwned>(
        &self,
        config: &MobileConfig,
        path: &str,
        body: &B,
    ) -> Result<R, MobError> {
        let url = endpoint_url(config, path)?;
        let response = self
            .client
            .post(url)
            .header("X-AOXC-App-Id", &config.app_id)
            .header("X-AOXC-Chain-Id", &config.chain_id)
            .json(body)
            .send()
            .await
            .map_err(|error| MobError::Transport(format!("http post failed: {}", error)))?;
        parse_json_response(response).await
    }

    async fn get_json<R: serde::de::DeserializeOwned>(
        &self,
        config: &MobileConfig,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<R, MobError> {
        let mut url = endpoint_url(config, path)?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        let response = self
            .client
            .get(url)
            .header("X-AOXC-App-Id", &config.app_id)
            .header("X-AOXC-Chain-Id", &config.chain_id)
            .send()
            .await
            .map_err(|error| MobError::Transport(format!("http get failed: {}", error)))?;
        parse_json_response(response).await
    }
}

#[async_trait]
impl AoxcMobileTransport for HttpRelayTransport {
    async fn request_session_challenge(
        &self,
        profile: &DeviceProfile,
        config: &MobileConfig,
    ) -> Result<SessionChallenge, MobError> {
        self.post_json(config, "/v1/mobile/session/challenge", profile)
            .await
    }

    async fn submit_session_envelope(
        &self,
        envelope: SessionEnvelope,
        config: &MobileConfig,
    ) -> Result<SessionPermit, MobError> {
        self.post_json(config, "/v1/mobile/session/open", &envelope)
            .await
    }

    async fn fetch_chain_health(
        &self,
        permit: &SessionPermit,
        config: &MobileConfig,
    ) -> Result<ChainHealth, MobError> {
        self.get_json(
            config,
            "/v1/mobile/chain/health",
            &[("session_id", &permit.session_id)],
        )
        .await
    }

    async fn fetch_available_tasks(
        &self,
        permit: &SessionPermit,
        config: &MobileConfig,
    ) -> Result<Vec<TaskDescriptor>, MobError> {
        self.get_json(
            config,
            "/v1/mobile/tasks",
            &[("session_id", &permit.session_id)],
        )
        .await
    }

    async fn submit_task_receipt(
        &self,
        receipt: SignedTaskReceipt,
        config: &MobileConfig,
    ) -> Result<TaskSubmissionResult, MobError> {
        self.post_json(config, "/v1/mobile/tasks/submit", &receipt)
            .await
    }
}

fn endpoint_url(config: &MobileConfig, path: &str) -> Result<Url, MobError> {
    let base = Url::parse(&config.relay_origin)
        .map_err(|error| MobError::Transport(format!("relay_origin parse failed: {}", error)))?;
    base.join(path)
        .map_err(|error| MobError::Transport(format!("endpoint url build failed: {}", error)))
}

async fn parse_json_response<R: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<R, MobError> {
    let status = response.status();
    if status == StatusCode::NO_CONTENT {
        return Err(MobError::Transport(
            "relay returned empty response payload".to_string(),
        ));
    }
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable error body>".to_string());
        return Err(MobError::Transport(format!(
            "relay request failed with status {}: {}",
            status, body
        )));
    }
    response
        .json::<R>()
        .await
        .map_err(|error| MobError::Transport(format!("relay json decode failed: {}", error)))
}
