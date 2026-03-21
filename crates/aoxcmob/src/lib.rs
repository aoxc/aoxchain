pub mod config;
pub mod error;
pub mod gateway;
pub mod security;
pub mod session;
pub mod transport;
pub mod types;
pub mod util;

pub use config::MobileConfig;
pub use error::MobError;
pub use gateway::native::NativeGateway;
pub use security::keystore::{InMemorySecureStore, SecureStore};
pub use session::protocol::{SessionContext, SessionPermit};
pub use transport::api::{AoxcMobileTransport, TaskSubmissionResult};
pub use transport::mock::MockRelayTransport;
pub use types::{
    ChainHealth, DevicePlatform, DeviceProfile, SignedTaskReceipt, TaskDescriptor, TaskKind,
    TaskReceipt, WitnessDecision,
};

/// Returns the AOXC mobile core version.
#[must_use]
pub fn mob_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the base SDK version currently coupled to the mobile crate.
#[must_use]
pub fn coupled_sdk_version() -> &'static str {
    aoxcsdk::sdk_version()
}
