// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;
use crate::lanes::VirtualMachine;
use crate::vm_kind::VmKind;

use super::state::EvmContractAccount;

/// Minimal but real EVM lane executor.
///
/// Supported operations:
/// - `0x00 || <bytecode>`: deploy contract
/// - `0x01 || <20-byte address> || <calldata>`: call contract
#[derive(Debug, Default, Clone, Copy)]
pub struct EvmExecutor;

impl EvmExecutor {
    const NS_CONTRACTS: &'static [u8] = b"contracts";
    const TOPIC_DEPLOYED: &'static [u8] = b"evm.contract.deployed";
    const TOPIC_CALLED: &'static [u8] = b"evm.contract.called";
    const DEPLOY_GAS: u64 = 53_000;
    const CALL_GAS: u64 = 25_000;

    fn derive_contract_address(tx_hash: &[u8; 32]) -> [u8; 20] {
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&tx_hash[12..32]);
        addr
    }
}

impl VirtualMachine for EvmExecutor {
    fn kind(&self) -> VmKind {
        VmKind::Evm
    }

    fn validate_tx(
        &self,
        _state: &dyn HostStateView,
        _block: &BlockContext,
        tx: &TxContext,
    ) -> Result<(), AovmError> {
        tx.validate_basic()?;

        if tx.vm_kind != VmKind::Evm {
            return Err(AovmError::InvalidTransaction(
                "transaction routed to incorrect VM",
            ));
        }

        match tx.payload.first().copied() {
            Some(0x00) if tx.payload.len() >= 2 => Ok(()),
            Some(0x01) if tx.payload.len() > 20 => Ok(()),
            Some(_) => Err(AovmError::DecodeError("unsupported EVM opcode envelope")),
            None => Err(AovmError::DecodeError("missing EVM opcode envelope")),
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
                state.charge_gas(Self::DEPLOY_GAS)?;

                let address = Self::derive_contract_address(&tx.tx_hash);
                let account = EvmContractAccount {
                    address,
                    code: tx.payload[1..].to_vec(),
                };

                state.write(VmKind::Evm, Self::NS_CONTRACTS, &address, account.encode())?;

                state.emit_event(VmKind::Evm, Self::TOPIC_DEPLOYED.to_vec(), address.to_vec())?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::Evm,
                    Self::DEPLOY_GAS,
                    events,
                    address.to_vec(),
                ))
            }
            0x01 => {
                state.charge_gas(Self::CALL_GAS)?;

                let mut address = [0u8; 20];
                address.copy_from_slice(&tx.payload[1..21]);

                let stored = state
                    .read(VmKind::Evm, Self::NS_CONTRACTS, &address)?
                    .ok_or(AovmError::NotFound("contract account not found"))?;

                let _account = EvmContractAccount::decode(&stored).ok_or(
                    AovmError::DecodeError("corrupt EVM contract account encoding"),
                )?;

                let calldata = tx.payload[21..].to_vec();

                state.emit_event(VmKind::Evm, Self::TOPIC_CALLED.to_vec(), calldata.clone())?;

                let events = state.drain_events();
                Ok(ExecutionReceipt::success(
                    VmKind::Evm,
                    Self::CALL_GAS,
                    events,
                    calldata,
                ))
            }
            _ => Err(AovmError::UnsupportedOperation("unknown EVM operation")),
        }
    }

    fn query(
        &self,
        state: &dyn HostStateView,
        _block: &BlockContext,
        payload: &[u8],
    ) -> Result<Vec<u8>, AovmError> {
        if payload.len() != 20 {
            return Err(AovmError::DecodeError(
                "EVM query expects a 20-byte address",
            ));
        }

        state
            .read(VmKind::Evm, Self::NS_CONTRACTS, payload)?
            .ok_or(AovmError::NotFound("contract account not found"))
    }
}
