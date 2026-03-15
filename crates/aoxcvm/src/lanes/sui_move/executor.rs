use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;
use crate::lanes::VirtualMachine;
use crate::vm_kind::VmKind;

use super::object_store::{SuiObject, SuiOwner};
use super::package_store::SuiPackage;

/// Minimal but real Sui / Move-style executor.
///
/// Supported operations:
/// - `0x00 || <package-bytes>`: publish package
/// - `0x01 || <type-len:u8> || <type-tag> || <body>`: create owned object
#[derive(Debug, Default, Clone, Copy)]
pub struct SuiMoveExecutor;

impl SuiMoveExecutor {
    const NS_PACKAGES: &'static [u8] = b"packages";
    const NS_OBJECTS: &'static [u8] = b"objects";
    const TOPIC_PACKAGE_PUBLISHED: &'static [u8] = b"sui.package.published";
    const TOPIC_OBJECT_CREATED: &'static [u8] = b"sui.object.created";
    const PUBLISH_GAS: u64 = 40_000;
    const OBJECT_CREATE_GAS: u64 = 18_000;

    fn derive_id(seed: &[u8; 32]) -> [u8; 32] {
        *seed
    }
}

impl VirtualMachine for SuiMoveExecutor {
    fn kind(&self) -> VmKind {
        VmKind::SuiMove
    }

    fn validate_tx(
        &self,
        _state: &dyn HostStateView,
        _block: &BlockContext,
        tx: &TxContext,
    ) -> Result<(), AovmError> {
        tx.validate_basic()?;

        if tx.vm_kind != VmKind::SuiMove {
            return Err(AovmError::InvalidTransaction(
                "transaction routed to incorrect VM",
            ));
        }

        match tx.payload.first().copied() {
            Some(0x00) if tx.payload.len() >= 2 => Ok(()),
            Some(0x01) if tx.payload.len() >= 3 => Ok(()),
            Some(_) => Err(AovmError::DecodeError("unsupported Sui/Move operation")),
            None => Err(AovmError::DecodeError("missing Sui/Move opcode envelope")),
        }
    }

    fn execute_tx(
        &self,
        state: &mut dyn HostStateView,
        block: &BlockContext,
        tx: &TxContext,
    ) -> Result<ExecutionReceipt, AovmError> {
        self.validate_tx(state, block, tx)?;

        match tx.payload[0] {
            0x00 => {
                state.charge_gas(Self::PUBLISH_GAS)?;

                let package_id = Self::derive_id(&tx.tx_hash);
                let package = SuiPackage {
                    package_id,
                    modules: tx.payload[1..].to_vec(),
                };

                state.write(
                    VmKind::SuiMove,
                    Self::NS_PACKAGES,
                    &package_id,
                    package.encode(),
                )?;

                state.emit_event(
                    VmKind::SuiMove,
                    Self::TOPIC_PACKAGE_PUBLISHED.to_vec(),
                    package_id.to_vec(),
                )?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::SuiMove,
                    Self::PUBLISH_GAS,
                    events,
                    package_id.to_vec(),
                ))
            }
            0x01 => {
                state.charge_gas(Self::OBJECT_CREATE_GAS)?;

                let type_len = tx.payload[1] as usize;
                let start = 2usize;
                let end = start + type_len;

                if tx.payload.len() < end {
                    return Err(AovmError::DecodeError("truncated Sui type tag"));
                }

                let type_tag = String::from_utf8(tx.payload[start..end].to_vec())
                    .map_err(|_| AovmError::DecodeError("invalid UTF-8 type tag"))?;

                let body = tx.payload[end..].to_vec();
                let object_id = Self::derive_id(&tx.tx_hash);

                let object = SuiObject {
                    object_id,
                    version: 1,
                    owner: SuiOwner::Address(tx.sender.clone()),
                    type_tag,
                    bcs_bytes: body,
                };

                state.write(
                    VmKind::SuiMove,
                    Self::NS_OBJECTS,
                    &object_id,
                    object.encode(),
                )?;

                state.emit_event(
                    VmKind::SuiMove,
                    Self::TOPIC_OBJECT_CREATED.to_vec(),
                    object_id.to_vec(),
                )?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::SuiMove,
                    Self::OBJECT_CREATE_GAS,
                    events,
                    object_id.to_vec(),
                ))
            }
            _ => Err(AovmError::UnsupportedOperation(
                "unknown Sui/Move operation",
            )),
        }
    }

    fn query(
        &self,
        state: &dyn HostStateView,
        _block: &BlockContext,
        payload: &[u8],
    ) -> Result<Vec<u8>, AovmError> {
        if payload.len() != 32 {
            return Err(AovmError::DecodeError(
                "Sui query expects a 32-byte object or package id",
            ));
        }

        if let Some(object) = state.read(VmKind::SuiMove, Self::NS_OBJECTS, payload)? {
            return Ok(object);
        }

        state
            .read(VmKind::SuiMove, Self::NS_PACKAGES, payload)?
            .ok_or(AovmError::NotFound("object or package not found"))
    }
}
