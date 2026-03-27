// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    receipts::{Receipt, calculate_receipts_root},
    transaction::{Transaction, TransactionError},
};

use super::{BlockError, TargetOutpost, Task, calculate_task_root};

/// Canonical lane identifier derived from AOXC task destinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssemblyLane {
    Native,
    EthereumSettlement,
    SolanaReward,
    BaseSettlement,
}

impl AssemblyLane {
    #[must_use]
    pub const fn lane_id(self) -> u32 {
        match self {
            Self::Native => 1,
            Self::EthereumSettlement => 10,
            Self::SolanaReward => 20,
            Self::BaseSettlement => 30,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::EthereumSettlement => "ethereum-settlement",
            Self::SolanaReward => "solana-reward",
            Self::BaseSettlement => "base-settlement",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssemblyLaneCommitment {
    pub lane: AssemblyLane,
    pub lane_id: u32,
    pub task_count: u32,
    pub payload_bytes: u64,
    pub task_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalBlockAssemblyPlan {
    pub plan_version: u8,
    pub height: u64,
    pub parent_hash: [u8; 32],
    pub producer: [u8; 32],
    pub task_count: u32,
    pub total_payload_bytes: u64,
    pub task_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub execution_root: [u8; 32],
    pub lanes: Vec<AssemblyLaneCommitment>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssemblyError {
    EmptyTransactions,
    ReceiptCountMismatch {
        transactions: usize,
        receipts: usize,
    },
    TaskCountOverflow,
    PayloadLengthOverflow,
    Transaction(TransactionError),
    Block(BlockError),
    ReceiptHashingFailed,
}

impl core::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyTransactions => write!(f, "block assembly failed: no transactions supplied"),
            Self::ReceiptCountMismatch {
                transactions,
                receipts,
            } => write!(
                f,
                "block assembly failed: receipt count {} does not match transaction count {}",
                receipts, transactions
            ),
            Self::TaskCountOverflow => write!(
                f,
                "block assembly failed: transaction/task count exceeds supported range"
            ),
            Self::PayloadLengthOverflow => write!(
                f,
                "block assembly failed: aggregate payload length exceeds supported range"
            ),
            Self::Transaction(error) => write!(f, "block assembly failed: {error}"),
            Self::Block(error) => write!(f, "block assembly failed: {error}"),
            Self::ReceiptHashingFailed => {
                write!(f, "block assembly failed: receipts root computation failed")
            }
        }
    }
}

impl std::error::Error for AssemblyError {}

impl From<BlockError> for AssemblyError {
    fn from(value: BlockError) -> Self {
        Self::Block(value)
    }
}

impl From<TransactionError> for AssemblyError {
    fn from(value: TransactionError) -> Self {
        Self::Transaction(value)
    }
}

impl CanonicalBlockAssemblyPlan {
    pub const PLAN_VERSION: u8 = 1;

    pub fn from_transactions(
        height: u64,
        parent_hash: [u8; 32],
        producer: [u8; 32],
        transactions: &[Transaction],
        receipts: &[Receipt],
    ) -> Result<Self, AssemblyError> {
        if transactions.is_empty() {
            return Err(AssemblyError::EmptyTransactions);
        }
        if transactions.len() != receipts.len() {
            return Err(AssemblyError::ReceiptCountMismatch {
                transactions: transactions.len(),
                receipts: receipts.len(),
            });
        }

        let tasks = transactions
            .iter()
            .map(Transaction::to_task)
            .collect::<Result<Vec<_>, _>>()?;
        let task_root = calculate_task_root(&tasks)?;
        let receipts_root = calculate_receipts_root(receipts);
        let task_count =
            u32::try_from(tasks.len()).map_err(|_| AssemblyError::TaskCountOverflow)?;
        let total_payload_bytes = tasks.iter().try_fold(0u64, |acc, task| {
            let len = u64::try_from(task.payload_len())
                .map_err(|_| AssemblyError::PayloadLengthOverflow)?;
            acc.checked_add(len)
                .ok_or(AssemblyError::PayloadLengthOverflow)
        })?;

        let lanes = build_lane_commitments(&tasks)?;
        let execution_root = derive_execution_root(task_root, receipts_root, &lanes);

        Ok(Self {
            plan_version: Self::PLAN_VERSION,
            height,
            parent_hash,
            producer,
            task_count,
            total_payload_bytes,
            task_root,
            receipts_root,
            execution_root,
            lanes,
            tasks,
        })
    }
}

fn build_lane_commitments(tasks: &[Task]) -> Result<Vec<AssemblyLaneCommitment>, AssemblyError> {
    let mut grouped: BTreeMap<AssemblyLane, Vec<Task>> = BTreeMap::new();
    for task in tasks {
        grouped
            .entry(lane_for_target(task.target_outpost))
            .or_default()
            .push(task.clone());
    }

    grouped
        .into_iter()
        .map(|(lane, tasks)| {
            let task_root = calculate_task_root(&tasks)?;
            let task_count =
                u32::try_from(tasks.len()).map_err(|_| AssemblyError::TaskCountOverflow)?;
            let payload_bytes = tasks.iter().try_fold(0u64, |acc, task| {
                let len = u64::try_from(task.payload_len())
                    .map_err(|_| AssemblyError::PayloadLengthOverflow)?;
                acc.checked_add(len)
                    .ok_or(AssemblyError::PayloadLengthOverflow)
            })?;

            Ok(AssemblyLaneCommitment {
                lane,
                lane_id: lane.lane_id(),
                task_count,
                payload_bytes,
                task_root,
            })
        })
        .collect()
}

fn derive_execution_root(
    task_root: [u8; 32],
    receipts_root: [u8; 32],
    lanes: &[AssemblyLaneCommitment],
) -> [u8; 32] {
    use sha3::{Digest, Sha3_256};

    let mut hasher = Sha3_256::new();
    hasher.update(b"AOXC-CANONICAL-BLOCK-ASSEMBLY-V1");
    hasher.update(task_root);
    hasher.update(receipts_root);
    hasher.update((lanes.len() as u64).to_le_bytes());
    for lane in lanes {
        hasher.update(lane.lane_id.to_le_bytes());
        hasher.update(lane.task_count.to_le_bytes());
        hasher.update(lane.payload_bytes.to_le_bytes());
        hasher.update(lane.task_root);
    }
    hasher.finalize().into()
}

const fn lane_for_target(target: TargetOutpost) -> AssemblyLane {
    match target {
        TargetOutpost::EthMainnetGateway => AssemblyLane::EthereumSettlement,
        TargetOutpost::SolanaRewardProgram => AssemblyLane::SolanaReward,
        TargetOutpost::BaseSettlementRouter => AssemblyLane::BaseSettlement,
        TargetOutpost::AovmNative => AssemblyLane::Native,
    }
}

#[cfg(test)]
mod tests {
    use super::CanonicalBlockAssemblyPlan;
    use crate::{
        block::{Capability, TargetOutpost},
        receipts::Receipt,
        transaction::Transaction,
    };
    use ed25519_dalek::{Signer, SigningKey};

    fn signed_tx(nonce: u64, target: TargetOutpost, payload: &[u8]) -> (Transaction, [u8; 32]) {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let sender = signing_key.verifying_key().to_bytes();
        let mut signature = [0u8; 64];
        let tx = Transaction::new(
            sender,
            nonce,
            Capability::UserSigned,
            target,
            payload.to_vec(),
            signature,
        )
        .expect("transaction shape must be valid");
        let sig = signing_key.sign(
            &tx.try_signing_message()
                .expect("signing payload must encode"),
        );
        signature.copy_from_slice(&sig.to_bytes());
        let signed = Transaction::new(
            sender,
            nonce,
            Capability::UserSigned,
            target,
            payload.to_vec(),
            signature,
        )
        .expect("signed transaction must be valid");
        let hash = signed.tx_id();
        (signed, hash)
    }

    #[test]
    fn assembly_plan_is_deterministic_for_same_inputs() {
        let (tx1, tx1_hash) = signed_tx(1, TargetOutpost::AovmNative, b"hello");
        let (tx2, tx2_hash) = signed_tx(2, TargetOutpost::EthMainnetGateway, b"world");
        let receipts = vec![
            Receipt::success(tx1_hash, 10),
            Receipt::success(tx2_hash, 20),
        ];

        let a = CanonicalBlockAssemblyPlan::from_transactions(
            7,
            [1u8; 32],
            [2u8; 32],
            &[tx1.clone(), tx2.clone()],
            &receipts,
        )
        .expect("assembly must succeed");
        let b = CanonicalBlockAssemblyPlan::from_transactions(
            7,
            [1u8; 32],
            [2u8; 32],
            &[tx1, tx2],
            &receipts,
        )
        .expect("assembly must succeed");

        assert_eq!(a.task_root, b.task_root);
        assert_eq!(a.receipts_root, b.receipts_root);
        assert_eq!(a.execution_root, b.execution_root);
        assert_eq!(a.lanes, b.lanes);
    }

    #[test]
    fn assembly_plan_rejects_receipt_count_mismatch() {
        let (tx, _) = signed_tx(1, TargetOutpost::AovmNative, b"hello");
        let error =
            CanonicalBlockAssemblyPlan::from_transactions(1, [0u8; 32], [1u8; 32], &[tx], &[])
                .expect_err("receipt mismatch must fail");

        assert!(matches!(
            error,
            super::AssemblyError::ReceiptCountMismatch {
                transactions: 1,
                receipts: 0
            }
        ));
    }
}
