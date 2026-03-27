// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::vm_kind::VmKind;

/// Canonical protocol registry entry.
///
/// This structure allows the host to index protocol-native resources
/// without coupling them to a single execution model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    pub resource_id: [u8; 32],
    pub lane: VmKind,
    pub owner: Vec<u8>,
}
