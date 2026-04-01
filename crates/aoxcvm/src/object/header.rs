use crate::object::kind::ObjectKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectHeader {
    pub id: [u8; 32],
    pub kind: ObjectKind,
    pub version: u32,
    pub policy_id: [u8; 32],
}
