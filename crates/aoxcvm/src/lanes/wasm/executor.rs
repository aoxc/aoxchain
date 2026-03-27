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
    const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
    const WASM_SUPPORTED_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];
    const MAX_CODE_BYTES: usize = 2 * 1024 * 1024;
    const MAX_INSTANCE_STATE_BYTES: usize = 256 * 1024;
    const UPLOAD_GAS_PER_BYTE: u64 = 3;
    const EXECUTE_GAS_PER_BYTE: u64 = 2;

    fn validate_wasm_module(bytes: &[u8]) -> Result<(), AovmError> {
        if bytes.len() < 8 {
            return Err(AovmError::InvalidTransaction(
                "WASM module is too small to contain magic/version header",
            ));
        }
        if !bytes.starts_with(&Self::WASM_MAGIC) {
            return Err(AovmError::InvalidTransaction(
                "WASM module must start with \\0asm magic bytes",
            ));
        }
        if bytes[4..8] != Self::WASM_SUPPORTED_VERSION {
            return Err(AovmError::InvalidTransaction(
                "WASM module version is unsupported (expected v1)",
            ));
        }
        if bytes.len() > Self::MAX_CODE_BYTES {
            return Err(AovmError::InvalidTransaction(
                "WASM module exceeds max code size limit",
            ));
        }
        Ok(())
    }

    fn validate_instance_state(bytes: &[u8]) -> Result<(), AovmError> {
        if bytes.len() > Self::MAX_INSTANCE_STATE_BYTES {
            return Err(AovmError::InvalidTransaction(
                "WASM instance state exceeds max size limit",
            ));
        }
        Ok(())
    }
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
            Some(0x00) if tx.payload.len() >= 2 => Self::validate_wasm_module(&tx.payload[1..]),
            Some(0x01) if tx.payload.len() >= 33 => {
                Self::validate_instance_state(&tx.payload[33..])
            }
            Some(0x02) if tx.payload.len() >= 33 => {
                Self::validate_instance_state(&tx.payload[33..])
            }
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
                let code_bytes = &tx.payload[1..];
                state.charge_gas(
                    Self::UPLOAD_GAS
                        + (code_bytes.len() as u64).saturating_mul(Self::UPLOAD_GAS_PER_BYTE),
                )?;

                let code = WasmCode {
                    code_id: tx.tx_hash,
                    bytes: code_bytes.to_vec(),
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
                    Self::UPLOAD_GAS
                        + (code.bytes.len() as u64).saturating_mul(Self::UPLOAD_GAS_PER_BYTE),
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
                let new_state = &tx.payload[33..];
                state.charge_gas(
                    Self::EXECUTE_GAS
                        + (new_state.len() as u64).saturating_mul(Self::EXECUTE_GAS_PER_BYTE),
                )?;

                let mut instance_id = [0u8; 32];
                instance_id.copy_from_slice(&tx.payload[1..33]);

                let stored = state
                    .read(VmKind::Wasm, Self::NS_INSTANCES, &instance_id)?
                    .ok_or(AovmError::NotFound("WASM instance not found"))?;

                let mut instance = WasmInstance::decode(&stored)
                    .ok_or(AovmError::DecodeError("corrupt WASM instance encoding"))?;

                instance.state = new_state.to_vec();

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
                    Self::EXECUTE_GAS
                        + (instance.state.len() as u64).saturating_mul(Self::EXECUTE_GAS_PER_BYTE),
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
