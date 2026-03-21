use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeHandleSet {
    pub node: &'static str,
    pub telemetry: &'static str,
    pub ledger: &'static str,
}

pub fn default_handles() -> RuntimeHandleSet {
    RuntimeHandleSet {
        node: "local-node-handle",
        telemetry: "local-telemetry-handle",
        ledger: "local-ledger-handle",
    }
}
