use crate::websocket::events::BlockConfirmedEvent;

/// WebSocket RPC server entry point.
#[derive(Debug, Default)]
pub struct WebSocketServer;

impl WebSocketServer {
    /// Creates a new WebSocket server instance.
    #[must_use]
    pub fn new() -> Self {
        Self
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
}
