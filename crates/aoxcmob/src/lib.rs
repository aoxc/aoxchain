// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.
//!
//! AOXC mobile integration surface.
//!
//! # Examples
//! ```rust
//! let mob = aoxcmob::mob_version();
//! let sdk = aoxcmob::coupled_sdk_version();
//! assert!(!mob.is_empty());
//! assert!(!sdk.is_empty());
//! ```

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
pub use transport::http::HttpRelayTransport;
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
///
/// # Examples
/// ```rust
/// assert_eq!(aoxcmob::coupled_sdk_version(), aoxcsdk::sdk_version());
/// ```
#[must_use]
pub fn coupled_sdk_version() -> &'static str {
    aoxcsdk::sdk_version()
}
