// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::websocket::events::BlockConfirmedEvent;
use std::collections::{HashMap, HashSet};

/// WebSocket RPC server entry point.
#[derive(Debug, Default)]
pub struct WebSocketServer {
    sessions: HashMap<String, WebSocketSession>,
}

#[derive(Debug, Clone, Default)]
struct WebSocketSession {
    subscriptions: HashSet<String>,
}

impl WebSocketServer {
    /// Creates a new WebSocket server instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    #[must_use]
    pub fn format_event(&self, event: &BlockConfirmedEvent) -> String {
        serde_json::json!({
            "type": "BLOCK_CONFIRMED",
            "block_hash": event.block_hash,
            "height": event.height,
        })
        .to_string()
    }

    pub fn connect(&mut self, session_id: impl Into<String>) {
        self.sessions.entry(session_id.into()).or_default();
    }

    pub fn disconnect(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    pub fn subscribe(&mut self, session_id: &str, topic: &str) -> bool {
        let Some(session) = self.sessions.get_mut(session_id) else {
            return false;
        };
        session.subscriptions.insert(topic.to_string())
    }

    #[must_use]
    pub fn publish_block_confirmed(&self, event: &BlockConfirmedEvent) -> Vec<(String, String)> {
        let payload = self.format_event(event);
        self.sessions
            .iter()
            .filter(|(_, session)| session.subscriptions.contains("BLOCK_CONFIRMED"))
            .map(|(session_id, _)| (session_id.clone(), payload.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_targets_only_subscribed_sessions() {
        let mut server = WebSocketServer::new();
        server.connect("session-a");
        server.connect("session-b");
        assert!(server.subscribe("session-a", "BLOCK_CONFIRMED"));
        assert!(!server.subscribe("missing", "BLOCK_CONFIRMED"));

        let published = server.publish_block_confirmed(&BlockConfirmedEvent {
            block_hash: "0xabc".to_string(),
            height: 7,
        });

        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0, "session-a");
        assert!(published[0].1.contains("BLOCK_CONFIRMED"));
    }
}
