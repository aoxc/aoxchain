use serde::{Deserialize, Serialize};

use crate::error::NetworkError;
use crate::metrics::NetworkMetrics;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncRequest {
    pub from_height: u64,
    pub to_height: u64,
    pub include_state_summary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncResponse {
    pub from_height: u64,
    pub to_height: u64,
    pub blocks_transferred: u64,
    pub state_summary_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncState {
    pub local_height: u64,
    pub target_height: u64,
    pub in_flight: bool,
}

#[derive(Debug, Clone)]
pub struct SyncController {
    state: SyncState,
}

impl SyncController {
    #[must_use]
    pub fn new(local_height: u64, target_height: u64) -> Self {
        Self {
            state: SyncState {
                local_height,
                target_height,
                in_flight: false,
            },
        }
    }

    #[must_use]
    pub fn state(&self) -> &SyncState {
        &self.state
    }

    pub fn build_request(&mut self, max_batch: u64) -> Result<SyncRequest, NetworkError> {
        if self.state.local_height >= self.state.target_height {
            return Err(NetworkError::InvalidSyncRequest(
                "local height is already at or above target height".to_string(),
            ));
        }
        let from_height = self.state.local_height.saturating_add(1);
        let to_height = from_height
            .saturating_add(max_batch.saturating_sub(1))
            .min(self.state.target_height);
        self.state.in_flight = true;
        Ok(SyncRequest {
            from_height,
            to_height,
            include_state_summary: to_height == self.state.target_height,
        })
    }

    pub fn apply_response(&mut self, response: &SyncResponse, metrics: &mut NetworkMetrics) {
        self.state.local_height = response.to_height;
        self.state.in_flight = false;
        metrics.sync_requests = metrics.sync_requests.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{SyncController, SyncResponse};
    use crate::metrics::NetworkMetrics;

    #[test]
    fn sync_controller_builds_progressing_request() {
        let mut controller = SyncController::new(10, 20);
        let request = controller
            .build_request(5)
            .expect("request should be created");
        assert_eq!(request.from_height, 11);
        assert_eq!(request.to_height, 15);

        let mut metrics = NetworkMetrics::default();
        controller.apply_response(
            &SyncResponse {
                from_height: 11,
                to_height: 15,
                blocks_transferred: 5,
                state_summary_hash: None,
            },
            &mut metrics,
        );
        assert_eq!(controller.state().local_height, 15);
        assert_eq!(metrics.sync_requests, 1);
    }
}
