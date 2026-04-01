use crate::errors::AoxcvmError;
use crate::policy::vm_policy::VmPolicy;
use crate::result::Result;

pub fn enforce_execution_policy(policy: &VmPolicy, requested_syscalls: u16, touched_objects: u32) -> Result<()> {
    if requested_syscalls > policy.limits.max_syscalls {
        return Err(AoxcvmError::PolicyViolation("syscall limit exceeded"));
    }
    if touched_objects > policy.limits.max_objects_touched {
        return Err(AoxcvmError::PolicyViolation("object touch limit exceeded"));
    }
    Ok(())
}
