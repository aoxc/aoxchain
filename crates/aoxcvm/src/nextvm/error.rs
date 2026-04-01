use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NextVmError {
    #[error("gas exhausted before instruction execution")]
    OutOfGas,
    #[error("program counter moved outside instruction range")]
    ProgramCounterOutOfRange,
    #[error("host capability '{0}' is required")]
    MissingCapability(&'static str),
    #[error("invalid signature envelope for configured crypto profile")]
    InvalidSignatureEnvelope,
}
