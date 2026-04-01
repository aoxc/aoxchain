use crate::errors::AoxcvmError;
use crate::object::classes::capability::CapabilityObject;
use crate::result::Result;

pub fn require_capability(
    capabilities: &[CapabilityObject],
    namespace: &str,
    action: &str,
    resource: &str,
) -> Result<()> {
    let found = capabilities.iter().any(|c| c.namespace == namespace && c.action == action && c.resource == resource);
    if found { Ok(()) } else { Err(AoxcvmError::MissingCapability("required capability not present")) }
}
