use super::*;
use crate::block::{Capability, TargetOutpost};
use crate::transaction::Transaction;
use ed25519_dalek::{Signer, SigningKey};

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn signed_transaction(seed: u8, nonce: u64, payload: Vec<u8>) -> Transaction {
    let signing_key = signing_key(seed);
    let sender = signing_key.verifying_key().to_bytes();

    let unsigned = Transaction {
        sender,
        nonce,
        capability: Capability::UserSigned,
        target: TargetOutpost::EthMainnetGateway,
        payload,
        signature: [0u8; 64],
    };

    let signature = signing_key.sign(&unsigned.signing_message()).to_bytes();

    Transaction {
        signature,
        ..unsigned
    }
}

#[test]
fn default_config_is_valid() {
    let config = TransactionPoolConfig::default();
    assert_eq!(config.validate(), Ok(config));
}

#[test]
fn invalid_config_is_rejected() {
    let result = TransactionPool::with_config(TransactionPoolConfig {
        max_transactions: 0,
        max_transactions_per_sender: 1,
    });

    assert!(matches!(
        result,
        Err(TransactionPoolError::InvalidConfig(
            InvalidPoolConfig::ZeroMaxTransactions
        ))
    ));
}

#[test]
fn pool_accepts_valid_transaction() {
    let mut pool = TransactionPool::new();
    let tx = signed_transaction(1, 1, vec![1, 2, 3]);

    let tx_id = pool.add(tx).expect("valid transaction must be admitted");

    assert_eq!(pool.len(), 1);
    assert!(pool.contains_tx_id(&tx_id));
    assert!(pool.validate().is_ok());
}

#[test]
fn pool_rejects_duplicate_transaction_id() {
    let mut pool = TransactionPool::new();
    let tx = signed_transaction(1, 1, vec![1, 2, 3]);
    let tx_clone = tx.clone();

    let first_id = pool.add(tx).expect("first transaction must be admitted");
    let result = pool.add(tx_clone);

    assert_eq!(
        result,
        Err(TransactionPoolError::DuplicateTransactionId { tx_id: first_id })
    );
}

#[test]
fn pool_rejects_sender_nonce_conflict() {
    let mut pool = TransactionPool::new();

    let tx_a = signed_transaction(1, 7, vec![1, 2, 3]);
    let tx_b = signed_transaction(1, 7, vec![9, 9, 9]);

    let existing_tx_id = pool.add(tx_a).expect("first transaction must be admitted");

    let result = pool.add(tx_b);

    match result {
        Err(TransactionPoolError::SenderNonceConflict {
            sender: _,
            nonce,
            existing_tx_id: observed,
        }) => {
            assert_eq!(nonce, 7);
            assert_eq!(observed, existing_tx_id);
        }
        other => panic!("unexpected result: {:?}", other),
    }
}

#[test]
fn pool_rejects_when_global_capacity_is_reached() {
    let config = TransactionPoolConfig {
        max_transactions: 1,
        max_transactions_per_sender: 1,
    };

    let mut pool = TransactionPool::with_config(config).expect("config must be valid");
    pool.add(signed_transaction(1, 1, vec![1]))
        .expect("first transaction must be admitted");

    let result = pool.add(signed_transaction(2, 1, vec![2]));
    assert_eq!(
        result,
        Err(TransactionPoolError::PoolFull { current: 1, max: 1 })
    );
}

#[test]
fn pool_rejects_when_sender_capacity_is_reached() {
    let config = TransactionPoolConfig {
        max_transactions: 16,
        max_transactions_per_sender: 1,
    };

    let mut pool = TransactionPool::with_config(config).expect("config must be valid");
    let sender_seed = 9;

    pool.add(signed_transaction(sender_seed, 1, vec![1]))
        .expect("first transaction must be admitted");

    let result = pool.add(signed_transaction(sender_seed, 2, vec![2]));

    match result {
        Err(TransactionPoolError::SenderPoolLimitExceeded {
            sender: _,
            current,
            max,
        }) => {
            assert_eq!(current, 1);
            assert_eq!(max, 1);
        }
        other => panic!("unexpected result: {:?}", other),
    }
}

#[test]
fn pool_remove_clears_secondary_indexes() {
    let mut pool = TransactionPool::new();
    let tx = signed_transaction(2, 9, vec![4, 5, 6]);

    let tx_id = pool.add(tx).expect("transaction must be admitted");
    let removed = pool.remove(&tx_id).expect("transaction must be removable");

    assert_eq!(removed.nonce, 9);
    assert!(pool.is_empty());
    assert!(!pool.contains_sender_nonce(&removed.sender, removed.nonce));
    assert_eq!(pool.sender_transaction_count(&removed.sender), 0);
    assert!(pool.validate().is_ok());
}

#[test]
fn selection_is_bounded_by_count_and_payload() {
    let mut pool = TransactionPool::new();

    pool.add(signed_transaction(1, 1, vec![1, 2, 3]))
        .expect("tx1 must be admitted");
    pool.add(signed_transaction(2, 1, vec![4, 5, 6]))
        .expect("tx2 must be admitted");
    pool.add(signed_transaction(3, 1, vec![7, 8, 9]))
        .expect("tx3 must be admitted");

    let selected = pool.select_for_block(2, 6);
    assert_eq!(selected.len(), 2);
    assert_eq!(
        selected
            .iter()
            .map(|(_, tx)| tx.payload_len())
            .sum::<usize>(),
        6
    );
}

#[test]
fn drain_for_block_removes_selected_transactions() {
    let mut pool = TransactionPool::new();

    pool.add(signed_transaction(1, 1, vec![1]))
        .expect("tx1 must be admitted");
    pool.add(signed_transaction(2, 1, vec![2]))
        .expect("tx2 must be admitted");

    let drained = pool.drain_for_block(1, 1024).expect("drain must succeed");

    assert_eq!(drained.len(), 1);
    assert_eq!(pool.len(), 1);
    assert!(pool.validate().is_ok());
}
