use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;
use crate::lanes::VirtualMachine;
use crate::vm_kind::VmKind;

use super::state::{WasmCode, WasmInstance};

/// Minimal but real deterministic WASM executor.
///
/// Supported operations:
/// - `0x00 || <wasm-bytes>`: upload code
/// - `0x01 || <32-byte code-id> || <init-state>`: instantiate contract
/// - `0x02 || <32-byte instance-id> || <new-state>`: mutate instance state
#[derive(Debug, Default, Clone, Copy)]
pub struct WasmExecutor;

impl WasmExecutor {
    const NS_CODES: &'static [u8] = b"codes";
    const NS_INSTANCES: &'static [u8] = b"instances";
    const TOPIC_CODE_UPLOADED: &'static [u8] = b"wasm.code.uploaded";
    const TOPIC_INSTANCE_CREATED: &'static [u8] = b"wasm.instance.created";
    const TOPIC_INSTANCE_EXECUTED: &'static [u8] = b"wasm.instance.executed";
    const UPLOAD_GAS: u64 = 30_000;
    const INSTANTIATE_GAS: u64 = 24_000;
    const EXECUTE_GAS: u64 = 15_000;
}

impl VirtualMachine for WasmExecutor {
    fn kind(&self) -> VmKind {
        VmKind::Wasm
    }

    fn validate_tx(
        &self,
        _state: &dyn HostStateView,
        _block: &BlockContext,
        tx: &TxContext,
    ) -> Result<(), AovmError> {
        tx.validate_basic()?;

        if tx.vm_kind != VmKind::Wasm {
            return Err(AovmError::InvalidTransaction(
                "transaction routed to incorrect VM",
            ));
        }

        match tx.payload.first().copied() {
            Some(0x00) if tx.payload.len() >= 2 => Ok(()),
            Some(0x01) if tx.payload.len() >= 33 => Ok(()),
            Some(0x02) if tx.payload.len() >= 33 => Ok(()),
            Some(_) => Err(AovmError::DecodeError("unsupported WASM operation")),
            None => Err(AovmError::DecodeError("missing WASM opcode envelope")),
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
                state.charge_gas(Self::UPLOAD_GAS)?;

                let code = WasmCode {
                    code_id: tx.tx_hash,
                    bytes: tx.payload[1..].to_vec(),
                };

                state.write(VmKind::Wasm, Self::NS_CODES, &code.code_id, code.encode())?;
                state.emit_event(
                    VmKind::Wasm,
                    Self::TOPIC_CODE_UPLOADED.to_vec(),
                    code.code_id.to_vec(),
                )?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::Wasm,
                    Self::UPLOAD_GAS,
                    events,
                    code.code_id.to_vec(),
                ))
            }
            0x01 => {
                state.charge_gas(Self::INSTANTIATE_GAS)?;

                let mut code_id = [0u8; 32];
                code_id.copy_from_slice(&tx.payload[1..33]);

                let stored = state
                    .read(VmKind::Wasm, Self::NS_CODES, &code_id)?
                    .ok_or(AovmError::NotFound("WASM code not found"))?;

                let _code = WasmCode::decode(&stored)
                    .ok_or(AovmError::DecodeError("corrupt WASM code encoding"))?;

                let instance = WasmInstance {
                    instance_id: tx.tx_hash,
                    code_id,
                    state: tx.payload[33..].to_vec(),
                };

                state.write(
                    VmKind::Wasm,
                    Self::NS_INSTANCES,
                    &instance.instance_id,
                    instance.encode(),
                )?;

                state.emit_event(
                    VmKind::Wasm,
                    Self::TOPIC_INSTANCE_CREATED.to_vec(),
                    instance.instance_id.to_vec(),
                )?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::Wasm,
                    Self::INSTANTIATE_GAS,
                    events,
                    instance.instance_id.to_vec(),
                ))
            }
            0x02 => {
                state.charge_gas(Self::EXECUTE_GAS)?;

                let mut instance_id = [0u8; 32];
                instance_id.copy_from_slice(&tx.payload[1..33]);

                let stored = state
                    .read(VmKind::Wasm, Self::NS_INSTANCES, &instance_id)?
                    .ok_or(AovmError::NotFound("WASM instance not found"))?;

                let mut instance = WasmInstance::decode(&stored)
                    .ok_or(AovmError::DecodeError("corrupt WASM instance encoding"))?;

                instance.state = tx.payload[33..].to_vec();

                state.write(
                    VmKind::Wasm,
                    Self::NS_INSTANCES,
                    &instance.instance_id,
                    instance.encode(),
                )?;

                state.emit_event(
                    VmKind::Wasm,
                    Self::TOPIC_INSTANCE_EXECUTED.to_vec(),
                    instance.instance_id.to_vec(),
                )?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::Wasm,
                    Self::EXECUTE_GAS,
                    events,
                    instance.state,
                ))
            }
            _ => Err(AovmError::UnsupportedOperation("unknown WASM operation")),
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
                "WASM query expects a 32-byte identifier",
            ));
        }

        if let Some(instance) = state.read(VmKind::Wasm, Self::NS_INSTANCES, payload)? {
            return Ok(instance);
        }

        state
            .read(VmKind::Wasm, Self::NS_CODES, payload)?
            .ok_or(AovmError::NotFound("WASM code or instance not found"))
    }
}
