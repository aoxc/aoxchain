use crate::vm_kind::VmKind;

/// Cross-lane message envelope.
///
/// The runtime will eventually persist these messages into an outbox.
/// The structure is already stable enough to build on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossVmMessage {
    pub source_vm: VmKind,
    pub target_vm: VmKind,
    pub source_id: Vec<u8>,
    pub target_id: Vec<u8>,
    pub nonce: u64,
    pub payload: Vec<u8>,
}
