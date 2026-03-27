// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::BTreeMap;

use crate::error::AovmError;
use crate::host::receipt::ExecutionEvent;
use crate::host::storage::compose_storage_key;
use crate::vm_kind::VmKind;

/// Shared host state interface.
///
/// Every lane is expected to preserve its own state discipline while
/// using this common deterministic storage and accounting surface.
pub trait HostStateView {
    fn read(&self, vm: VmKind, namespace: &[u8], key: &[u8]) -> Result<Option<Vec<u8>>, AovmError>;
    fn write(
        &mut self,
        vm: VmKind,
        namespace: &[u8],
        key: &[u8],
        value: Vec<u8>,
    ) -> Result<(), AovmError>;
    fn delete(&mut self, vm: VmKind, namespace: &[u8], key: &[u8]) -> Result<(), AovmError>;
    fn emit_event(&mut self, vm: VmKind, topic: Vec<u8>, data: Vec<u8>) -> Result<(), AovmError>;
    fn charge_gas(&mut self, amount: u64) -> Result<(), AovmError>;
    fn gas_remaining(&self) -> u64;
    fn drain_events(&mut self) -> Vec<ExecutionEvent>;
}

/// In-memory deterministic host state used for development and tests.
///
/// This implementation is intentionally strict and side-effect transparent.
#[derive(Debug, Clone)]
pub struct InMemoryHostState {
    storage: BTreeMap<Vec<u8>, Vec<u8>>,
    pending_events: Vec<ExecutionEvent>,
    gas_remaining: u64,
}

impl InMemoryHostState {
    /// Creates a new in-memory host state with a gas budget.
    pub fn new(gas_limit: u64) -> Self {
        Self {
            storage: BTreeMap::new(),
            pending_events: Vec::new(),
            gas_remaining: gas_limit,
        }
    }

    /// Returns the number of stored host keys.
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Returns whether the host state is empty.
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Exposes a read-only view of the raw storage map for testing.
    pub fn raw_storage(&self) -> &BTreeMap<Vec<u8>, Vec<u8>> {
        &self.storage
    }
}

impl HostStateView for InMemoryHostState {
    fn read(&self, vm: VmKind, namespace: &[u8], key: &[u8]) -> Result<Option<Vec<u8>>, AovmError> {
        let composite = compose_storage_key(vm, namespace, key);
        Ok(self.storage.get(&composite).cloned())
    }

    fn write(
        &mut self,
        vm: VmKind,
        namespace: &[u8],
        key: &[u8],
        value: Vec<u8>,
    ) -> Result<(), AovmError> {
        let composite = compose_storage_key(vm, namespace, key);
        self.storage.insert(composite, value);
        Ok(())
    }

    fn delete(&mut self, vm: VmKind, namespace: &[u8], key: &[u8]) -> Result<(), AovmError> {
        let composite = compose_storage_key(vm, namespace, key);
        self.storage.remove(&composite);
        Ok(())
    }

    fn emit_event(&mut self, vm: VmKind, topic: Vec<u8>, data: Vec<u8>) -> Result<(), AovmError> {
        self.pending_events
            .push(ExecutionEvent::new(vm, topic, data));
        Ok(())
    }

    fn charge_gas(&mut self, amount: u64) -> Result<(), AovmError> {
        if self.gas_remaining < amount {
            return Err(AovmError::GasExhausted);
        }
        self.gas_remaining -= amount;
        Ok(())
    }

    fn gas_remaining(&self) -> u64 {
        self.gas_remaining
    }

    fn drain_events(&mut self) -> Vec<ExecutionEvent> {
        std::mem::take(&mut self.pending_events)
    }
}
