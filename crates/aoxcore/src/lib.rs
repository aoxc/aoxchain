// core/src/lib.rs

pub mod block;
pub mod contract;
pub mod genesis; // Exported for node visibility
pub mod identity; // Public export
pub mod mempool; // Keep public export
pub mod protocol;
pub mod receipts;
pub mod state;
pub mod transaction; // Keep public export

pub mod native_token;
pub mod version;
