// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::vm_kind::VmKind;

/// Canonical event emitted by any execution lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEvent {
    pub vm_kind: VmKind,
    pub topic: Vec<u8>,
    pub data: Vec<u8>,
}

impl ExecutionEvent {
    /// Constructs a normalized execution event.
    pub fn new(vm_kind: VmKind, topic: Vec<u8>, data: Vec<u8>) -> Self {
        Self {
            vm_kind,
            topic,
            data,
        }
    }
}

/// Canonical execution receipt returned by any execution lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionReceipt {
    pub vm_kind: VmKind,
    pub success: bool,
    pub gas_used: u64,
    pub events: Vec<ExecutionEvent>,
    pub output: Vec<u8>,
}

impl ExecutionReceipt {
    /// Constructs a successful receipt.
    pub fn success(
        vm_kind: VmKind,
        gas_used: u64,
        events: Vec<ExecutionEvent>,
        output: Vec<u8>,
    ) -> Self {
        Self {
            vm_kind,
            success: true,
            gas_used,
            events,
            output,
        }
    }

    /// Constructs a failed receipt.
    pub fn failure(vm_kind: VmKind, gas_used: u64, output: Vec<u8>) -> Self {
        Self {
            vm_kind,
            success: false,
            gas_used,
            events: Vec::new(),
            output,
        }
    }
}
