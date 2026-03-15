use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;
use crate::lanes::VirtualMachine;
use crate::vm_kind::VmKind;

use super::utxo_store::Utxo;

/// Minimal but real Cardano-style executor.
///
/// Supported operations:
/// - `0x00 || <value>`: create UTxO owned by sender
/// - `0x01 || <32-byte utxo-id>`: spend UTxO if owned by sender
#[derive(Debug, Default, Clone, Copy)]
pub struct CardanoExecutor;

impl CardanoExecutor {
    const NS_UTXOS: &'static [u8] = b"utxos";
    const TOPIC_UTXO_CREATED: &'static [u8] = b"cardano.utxo.created";
    const TOPIC_UTXO_SPENT: &'static [u8] = b"cardano.utxo.spent";
    const CREATE_GAS: u64 = 12_000;
    const SPEND_GAS: u64 = 9_000;
}

impl VirtualMachine for CardanoExecutor {
    fn kind(&self) -> VmKind {
        VmKind::Cardano
    }

    fn validate_tx(
        &self,
        _state: &dyn HostStateView,
        _block: &BlockContext,
        tx: &TxContext,
    ) -> Result<(), AovmError> {
        tx.validate_basic()?;

        if tx.vm_kind != VmKind::Cardano {
            return Err(AovmError::InvalidTransaction(
                "transaction routed to incorrect VM",
            ));
        }

        match tx.payload.first().copied() {
            Some(0x00) if tx.payload.len() >= 2 => Ok(()),
            Some(0x01) if tx.payload.len() == 33 => Ok(()),
            Some(_) => Err(AovmError::DecodeError("unsupported Cardano operation")),
            None => Err(AovmError::DecodeError("missing Cardano opcode envelope")),
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
                state.charge_gas(Self::CREATE_GAS)?;

                let utxo = Utxo {
                    utxo_id: tx.tx_hash,
                    owner: tx.sender.clone(),
                    value: tx.payload[1..].to_vec(),
                    datum: None,
                };

                state.write(
                    VmKind::Cardano,
                    Self::NS_UTXOS,
                    &utxo.utxo_id,
                    utxo.encode(),
                )?;
                state.emit_event(
                    VmKind::Cardano,
                    Self::TOPIC_UTXO_CREATED.to_vec(),
                    utxo.utxo_id.to_vec(),
                )?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::Cardano,
                    Self::CREATE_GAS,
                    events,
                    utxo.utxo_id.to_vec(),
                ))
            }
            0x01 => {
                state.charge_gas(Self::SPEND_GAS)?;

                let mut utxo_id = [0u8; 32];
                utxo_id.copy_from_slice(&tx.payload[1..33]);

                let stored = state
                    .read(VmKind::Cardano, Self::NS_UTXOS, &utxo_id)?
                    .ok_or(AovmError::NotFound("UTxO not found"))?;

                let utxo =
                    Utxo::decode(&stored).ok_or(AovmError::DecodeError("corrupt UTxO encoding"))?;

                if utxo.owner != tx.sender {
                    return Err(AovmError::StateAccessViolation(
                        "sender does not own the referenced UTxO",
                    ));
                }

                state.delete(VmKind::Cardano, Self::NS_UTXOS, &utxo_id)?;
                state.emit_event(
                    VmKind::Cardano,
                    Self::TOPIC_UTXO_SPENT.to_vec(),
                    utxo_id.to_vec(),
                )?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::Cardano,
                    Self::SPEND_GAS,
                    events,
                    utxo.value,
                ))
            }
            _ => Err(AovmError::UnsupportedOperation("unknown Cardano operation")),
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
                "Cardano query expects a 32-byte UTxO id",
            ));
        }

        state
            .read(VmKind::Cardano, Self::NS_UTXOS, payload)?
            .ok_or(AovmError::NotFound("UTxO not found"))
    }
}
