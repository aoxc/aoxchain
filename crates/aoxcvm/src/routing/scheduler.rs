// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::context::TxContext;
use crate::vm_kind::VmKind;

/// Deterministic scheduler that partitions transactions by lane.
///
/// The scheduler is intentionally simple at this stage. It preserves input
/// order within each lane while exposing stable lane partitions.
#[derive(Debug, Default)]
pub struct Scheduler;

impl Scheduler {
    /// Partitions transactions by lane while preserving intra-lane order.
    pub fn partition<'a>(
        &self,
        txs: &'a [TxContext],
    ) -> (
        Vec<&'a TxContext>,
        Vec<&'a TxContext>,
        Vec<&'a TxContext>,
        Vec<&'a TxContext>,
    ) {
        let mut evm = Vec::new();
        let mut sui = Vec::new();
        let mut wasm = Vec::new();
        let mut cardano = Vec::new();

        for tx in txs {
            match tx.vm_kind {
                VmKind::Evm => evm.push(tx),
                VmKind::SuiMove => sui.push(tx),
                VmKind::Wasm => wasm.push(tx),
                VmKind::Cardano => cardano.push(tx),
            }
        }

        (evm, sui, wasm, cardano)
    }
}
