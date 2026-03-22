use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    UsageUnknownCommand,
    UsageInvalidArguments,
    HomeResolutionFailed,
    FilesystemIoFailed,
    ConfigInvalid,
    ConfigMissing,
    KeyMaterialMissing,
    KeyMaterialInvalid,
    GenesisInvalid,
    LedgerInvalid,
    NodeStateInvalid,
    PolicyGateFailed,
    NetworkProbeFailed,
    AuditFailure,
    OutputEncodingFailed,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UsageUnknownCommand => "AOXC-USG-001",
            Self::UsageInvalidArguments => "AOXC-USG-002",
            Self::HomeResolutionFailed => "AOXC-HOM-001",
            Self::FilesystemIoFailed => "AOXC-FS-001",
            Self::ConfigInvalid => "AOXC-CFG-001",
            Self::ConfigMissing => "AOXC-CFG-002",
            Self::KeyMaterialMissing => "AOXC-KEY-001",
            Self::KeyMaterialInvalid => "AOXC-KEY-002",
            Self::GenesisInvalid => "AOXC-GEN-001",
            Self::LedgerInvalid => "AOXC-LED-001",
            Self::NodeStateInvalid => "AOXC-NOD-001",
            Self::PolicyGateFailed => "AOXC-POL-001",
            Self::NetworkProbeFailed => "AOXC-NET-001",
            Self::AuditFailure => "AOXC-AUD-001",
            Self::OutputEncodingFailed => "AOXC-OUT-001",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::UsageUnknownCommand | Self::UsageInvalidArguments => 2,
            Self::HomeResolutionFailed | Self::FilesystemIoFailed => 3,
            Self::ConfigInvalid | Self::ConfigMissing => 4,
            Self::KeyMaterialMissing | Self::KeyMaterialInvalid => 5,
            Self::GenesisInvalid => 6,
            Self::LedgerInvalid | Self::NodeStateInvalid => 7,
            Self::PolicyGateFailed | Self::NetworkProbeFailed | Self::AuditFailure => 8,
            Self::OutputEncodingFailed => 9,
        }
    }
}

#[derive(Debug)]
pub struct AppError {
    code: ErrorCode,
    message: String,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn with_source(
        code: ErrorCode,
        message: impl Into<String>,
        source: impl std::error::Error,
    ) -> Self {
        Self {
            code,
            message: format!("{}: {}", message.into(), source),
        }
    }

    pub fn code(&self) -> &'static str {
        self.code.as_str()
    }

    pub fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for AppError {}
