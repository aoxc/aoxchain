// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::vm_kind::VmKind;

/// EAM-style address binding record.
///
/// This is the seed of a system-level Ethereum address manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EamRecord {
    pub eth_address: [u8; 20],
    pub native_id: [u8; 32],
    pub lane: VmKind,
}
