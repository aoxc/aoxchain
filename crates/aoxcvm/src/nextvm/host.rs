use crate::nextvm::error::NextVmError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostCallRequest {
    pub selector: u64,
    pub arg0: u64,
}

pub trait HostAdapter {
    fn call(&mut self, request: HostCallRequest) -> Result<u64, NextVmError>;
}

#[derive(Default)]
pub struct NullHost;

impl HostAdapter for NullHost {
    fn call(&mut self, request: HostCallRequest) -> Result<u64, NextVmError> {
        Ok(request.selector.saturating_add(request.arg0))
    }
}
