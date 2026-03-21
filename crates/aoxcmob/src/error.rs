use std::error::Error;
use std::fmt;
use std::io;

/// Mobile-native error surface.
///
/// The variants intentionally carry stable codes so that higher layers can map
/// failures to telemetry, product messaging, and incident response policies.
#[derive(Debug)]
#[non_exhaustive]
pub enum MobError {
    InvalidConfiguration(&'static str),
    InvalidInput(&'static str),
    DeviceAlreadyProvisioned,
    DeviceNotProvisioned,
    SecureStorePoisoned,
    InvalidSessionChallenge(&'static str),
    SessionExpired,
    Transport(String),
    Serialization(String),
    Crypto(String),
    Io(String),
    Time(String),
}

impl MobError {
    /// Returns a stable machine-readable error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidConfiguration(_) => "AOXCMOB_CONFIG_INVALID",
            Self::InvalidInput(_) => "AOXCMOB_INPUT_INVALID",
            Self::DeviceAlreadyProvisioned => "AOXCMOB_DEVICE_ALREADY_PROVISIONED",
            Self::DeviceNotProvisioned => "AOXCMOB_DEVICE_NOT_PROVISIONED",
            Self::SecureStorePoisoned => "AOXCMOB_SECURE_STORE_POISONED",
            Self::InvalidSessionChallenge(_) => "AOXCMOB_SESSION_CHALLENGE_INVALID",
            Self::SessionExpired => "AOXCMOB_SESSION_EXPIRED",
            Self::Transport(_) => "AOXCMOB_TRANSPORT_ERROR",
            Self::Serialization(_) => "AOXCMOB_SERIALIZATION_ERROR",
            Self::Crypto(_) => "AOXCMOB_CRYPTO_ERROR",
            Self::Io(_) => "AOXCMOB_IO_ERROR",
            Self::Time(_) => "AOXCMOB_TIME_ERROR",
        }
    }
}

impl fmt::Display for MobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(message) => {
                write!(f, "{}: invalid configuration: {}", self.code(), message)
            }
            Self::InvalidInput(message) => {
                write!(f, "{}: invalid input: {}", self.code(), message)
            }
            Self::DeviceAlreadyProvisioned => {
                write!(f, "{}: device is already provisioned", self.code())
            }
            Self::DeviceNotProvisioned => {
                write!(f, "{}: device has not been provisioned", self.code())
            }
            Self::SecureStorePoisoned => {
                write!(f, "{}: secure store lock is poisoned", self.code())
            }
            Self::InvalidSessionChallenge(message) => {
                write!(f, "{}: invalid session challenge: {}", self.code(), message)
            }
            Self::SessionExpired => write!(f, "{}: session permit is expired", self.code()),
            Self::Transport(message) => write!(f, "{}: transport failure: {}", self.code(), message),
            Self::Serialization(message) => {
                write!(f, "{}: serialization failure: {}", self.code(), message)
            }
            Self::Crypto(message) => write!(f, "{}: cryptographic failure: {}", self.code(), message),
            Self::Io(message) => write!(f, "{}: I/O failure: {}", self.code(), message),
            Self::Time(message) => write!(f, "{}: time failure: {}", self.code(), message),
        }
    }
}

impl Error for MobError {}

impl From<serde_json::Error> for MobError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}

impl From<io::Error> for MobError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
