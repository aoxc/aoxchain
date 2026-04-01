pub mod access;
pub mod classes;
pub mod header;
pub mod id;
pub mod kind;
pub mod lease;
pub mod lock;
pub mod owner;
pub mod policy;
pub mod storage;
pub mod tombstone;
pub mod version;

use crate::object::header::ObjectHeader;
use crate::object::owner::Owner;
use crate::object::policy::MutationPolicy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedObject {
    pub header: ObjectHeader,
    pub owner: Owner,
    pub policy: MutationPolicy,
    pub body_hash: [u8; 32],
}
