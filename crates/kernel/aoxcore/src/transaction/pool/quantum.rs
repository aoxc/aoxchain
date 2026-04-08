// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::HashMap;

use crate::block::Task;
use crate::protocol::quantum::{QuantumAdmissionError, QuantumKernelProfile};
use crate::transaction::quantum::QuantumTransaction;

/// Deterministic pool for admitted quantum transactions.
#[derive(Debug, Default)]
pub struct QuantumTransactionPool {
    pending: HashMap<[u8; 32], QuantumTransaction>,
    sender_nonces: HashMap<(Vec<u8>, u64), [u8; 32]>,
}

/// Quantum pool admission failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantumTransactionPoolError {
    AdmissionRejected(QuantumAdmissionError),
    DuplicateTxId([u8; 32]),
    SenderNonceConflict([u8; 32]),
    TaskConversionFailed,
}

impl QuantumTransactionPool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn add_with_profile(
        &mut self,
        profile: &QuantumKernelProfile,
        tx: QuantumTransaction,
    ) -> Result<[u8; 32], QuantumTransactionPoolError> {
        profile
            .admit_quantum_transaction(&tx)
            .map_err(QuantumTransactionPoolError::AdmissionRejected)?;

        let tx_id = tx.tx_id();
        if self.pending.contains_key(&tx_id) {
            return Err(QuantumTransactionPoolError::DuplicateTxId(tx_id));
        }

        let sender_nonce = (tx.sender_public_key.clone(), tx.nonce);
        if let Some(existing) = self.sender_nonces.get(&sender_nonce) {
            return Err(QuantumTransactionPoolError::SenderNonceConflict(*existing));
        }

        self.sender_nonces.insert(sender_nonce, tx_id);
        self.pending.insert(tx_id, tx);
        Ok(tx_id)
    }

    pub fn remove(&mut self, tx_id: &[u8; 32]) -> Option<QuantumTransaction> {
        let removed = self.pending.remove(tx_id)?;
        self.sender_nonces
            .remove(&(removed.sender_public_key.clone(), removed.nonce));
        Some(removed)
    }

    /// Selects transactions for block assembly under count and payload limits.
    #[must_use]
    pub fn select_for_block(&self, max_count: usize, max_payload_bytes: usize) -> Vec<[u8; 32]> {
        let mut selected = Vec::new();
        let mut consumed = 0usize;

        for (tx_id, tx) in &self.pending {
            if selected.len() >= max_count {
                break;
            }

            let payload_len = tx.payload.len();
            if consumed + payload_len > max_payload_bytes {
                continue;
            }

            selected.push(*tx_id);
            consumed += payload_len;
        }

        selected
    }

    /// Drains selected transactions and converts them into block tasks.
    pub fn drain_for_block(
        &mut self,
        max_count: usize,
        max_payload_bytes: usize,
    ) -> Result<Vec<([u8; 32], Task)>, QuantumTransactionPoolError> {
        let selected = self.select_for_block(max_count, max_payload_bytes);
        let mut drained = Vec::with_capacity(selected.len());

        for tx_id in selected {
            let tx = self
                .remove(&tx_id)
                .expect("selected tx id must exist in pool during drain");

            let task = tx
                .to_task()
                .map_err(|_| QuantumTransactionPoolError::TaskConversionFailed)?;

            drained.push((tx_id, task));
        }

        Ok(drained)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Capability, TargetOutpost};
    use crate::identity::pq_keys;
    fn build_signed_tx(_seed: u8, nonce: u64, payload: Vec<u8>) -> QuantumTransaction {
        let (pk, sk) = pq_keys::generate_keypair();
        build_signed_tx_with_keypair(&pk, &sk, nonce, payload)
    }

    fn build_signed_tx_with_keypair(
        pk: &libcrux_ml_dsa::ml_dsa_65::MLDSA65VerificationKey,
        sk: &libcrux_ml_dsa::ml_dsa_65::MLDSA65SigningKey,
        nonce: u64,
        payload: Vec<u8>,
    ) -> QuantumTransaction {
        let message = QuantumTransaction::canonical_signing_message(
            nonce,
            Capability::UserSigned,
            TargetOutpost::EthMainnetGateway,
            &payload,
        )
        .expect("canonical signing message must be valid");

        let signed_payload = pq_keys::sign_message_domain_separated(&message, sk);

        QuantumTransaction::new(
            pq_keys::serialize_public_key(pk),
            nonce,
            Capability::UserSigned,
            TargetOutpost::EthMainnetGateway,
            payload,
            signed_payload,
        )
        .expect("quantum transaction must be valid")
    }

    #[test]
    fn pool_accepts_profile_admitted_quantum_transaction() {
        let profile = QuantumKernelProfile::strict_default();
        let mut pool = QuantumTransactionPool::new();
        let tx = build_signed_tx(1, 1, vec![1, 2, 3]);

        let tx_id = pool
            .add_with_profile(&profile, tx)
            .expect("admitted tx must enter pool");

        assert_eq!(pool.len(), 1);
        assert!(pool.pending.contains_key(&tx_id));
    }

    #[test]
    fn pool_rejects_sender_nonce_conflict() {
        let profile = QuantumKernelProfile::strict_default();
        let mut pool = QuantumTransactionPool::new();
        let (pk, sk) = pq_keys::generate_keypair();
        let tx1 = build_signed_tx_with_keypair(&pk, &sk, 7, vec![1]);
        let tx2 = build_signed_tx_with_keypair(&pk, &sk, 7, vec![2]);

        let existing_id = pool
            .add_with_profile(&profile, tx1)
            .expect("first tx must be admitted");
        let result = pool.add_with_profile(&profile, tx2);

        assert_eq!(
            result,
            Err(QuantumTransactionPoolError::SenderNonceConflict(
                existing_id
            ))
        );
    }

    #[test]
    fn pool_rejects_invalid_transaction_via_profile_admission() {
        let profile = QuantumKernelProfile::strict_default();
        let mut pool = QuantumTransactionPool::new();
        let mut tx = build_signed_tx(1, 3, vec![7, 8, 9]);
        tx.nonce = 0;

        let result = pool.add_with_profile(&profile, tx);
        assert_eq!(
            result,
            Err(QuantumTransactionPoolError::AdmissionRejected(
                QuantumAdmissionError::InvalidTransactionPayload
            ))
        );
    }

    #[test]
    fn remove_releases_sender_nonce_lane() {
        let profile = QuantumKernelProfile::strict_default();
        let mut pool = QuantumTransactionPool::new();
        let tx = build_signed_tx(1, 9, vec![1]);
        let tx_id = pool
            .add_with_profile(&profile, tx)
            .expect("tx must be admitted");
        assert_eq!(pool.len(), 1);

        let removed = pool.remove(&tx_id).expect("tx must be removable");
        assert_eq!(removed.nonce, 9);
        assert!(pool.is_empty());
    }

    #[test]
    fn drain_for_block_returns_tasks_and_removes_entries() {
        let profile = QuantumKernelProfile::strict_default();
        let mut pool = QuantumTransactionPool::new();

        pool.add_with_profile(&profile, build_signed_tx(1, 1, vec![1, 2]))
            .expect("tx1 must be admitted");
        pool.add_with_profile(&profile, build_signed_tx(2, 1, vec![3, 4]))
            .expect("tx2 must be admitted");

        let drained = pool
            .drain_for_block(1, 1024)
            .expect("drain must produce tasks");

        assert_eq!(drained.len(), 1);
        assert_eq!(pool.len(), 1);
    }
}
