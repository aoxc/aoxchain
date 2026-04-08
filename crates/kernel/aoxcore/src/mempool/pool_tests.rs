#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id(byte: u8) -> [u8; TX_ID_LEN] {
        [byte; TX_ID_LEN]
    }

    fn default_config() -> MempoolConfig {
        MempoolConfig {
            max_txs: 3,
            max_tx_size: 16,
            max_total_bytes: 32,
            tx_ttl: Duration::from_secs(60),
        }
    }

    #[test]
    fn creates_valid_mempool() {
        let mempool = Mempool::new(default_config()).expect("valid config must construct mempool");
        assert_eq!(mempool.len(), 0);
        assert!(mempool.is_empty());
        assert_eq!(mempool.total_bytes(), 0);
    }

    #[test]
    fn rejects_invalid_config() {
        let result = Mempool::new(MempoolConfig {
            max_txs: 0,
            max_tx_size: 16,
            max_total_bytes: 32,
            tx_ttl: Duration::from_secs(60),
        });

        assert!(matches!(result, Err(MempoolError::InvalidConfig(_))));
    }

    #[test]
    fn accepts_valid_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1, 2, 3])
            .expect("valid transaction must be accepted");

        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
        assert!(mempool.contains(&sample_id(1)));
        assert_eq!(mempool.lifecycle_stats().accepted, 1);
    }

    #[test]
    fn rejects_zero_transaction_id() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let result = mempool.add_tx([0u8; TX_ID_LEN], vec![1]);

        assert_eq!(result, Err(MempoolError::ZeroTransactionId));
        assert_eq!(mempool.rejection_stats().zero_id, 1);
    }

    #[test]
    fn rejects_empty_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let result = mempool.add_tx(sample_id(1), vec![]);

        assert_eq!(result, Err(MempoolError::EmptyTransaction));
        assert_eq!(mempool.rejection_stats().empty, 1);
    }

    #[test]
    fn rejects_oversized_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let result = mempool.add_tx(sample_id(1), vec![0u8; 17]);

        assert_eq!(
            result,
            Err(MempoolError::TransactionTooLarge {
                size: 17,
                max_allowed: 16,
            })
        );
        assert_eq!(mempool.rejection_stats().oversized, 1);
    }

    #[test]
    fn rejects_duplicate_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1, 2, 3])
            .expect("first transaction must be accepted");
        let result = mempool
            .add_tx(sample_id(1), vec![1, 2, 3])
            .expect_err("duplicate transaction must be rejected");

        assert_eq!(result, MempoolError::DuplicateTransaction);
        assert_eq!(mempool.rejection_stats().duplicate, 1);
    }

    #[test]
    fn rejects_when_tx_capacity_is_reached() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1]).unwrap();
        mempool.add_tx(sample_id(2), vec![2]).unwrap();
        mempool.add_tx(sample_id(3), vec![3]).unwrap();

        let result = mempool
            .add_tx(sample_id(4), vec![4])
            .expect_err("capacity overflow must be rejected");

        assert_eq!(result, MempoolError::MempoolFull { max_txs: 3 });
        assert_eq!(mempool.rejection_stats().full, 1);
    }

    #[test]
    fn rejects_when_total_bytes_would_be_exceeded() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![0u8; 16]).unwrap();
        mempool.add_tx(sample_id(2), vec![0u8; 16]).unwrap();

        let result = mempool
            .add_tx(sample_id(3), vec![1])
            .expect_err("total bytes overflow must be rejected");

        assert_eq!(
            result,
            MempoolError::TotalBytesExceeded {
                current_bytes: 32,
                tx_size: 1,
                max_allowed: 32,
            }
        );
        assert_eq!(mempool.rejection_stats().bytes_exceeded, 1);
    }

    #[test]
    fn collect_preserves_fifo_order() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1]).unwrap();
        mempool.add_tx(sample_id(2), vec![2]).unwrap();
        mempool.add_tx(sample_id(3), vec![3]).unwrap();

        let collected = mempool.collect(2);

        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].id, sample_id(1));
        assert_eq!(collected[1].id, sample_id(2));
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 1);
        assert!(mempool.contains(&sample_id(3)));
        assert_eq!(mempool.lifecycle_stats().collected, 2);
    }

    #[test]
    fn collect_with_zero_limit_is_no_op() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2, 3]).unwrap();

        let collected = mempool.collect(0);

        assert!(collected.is_empty());
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
    }

    #[test]
    fn remove_existing_transaction_updates_accounting() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2]).unwrap();
        mempool.add_tx(sample_id(2), vec![3, 4, 5]).unwrap();

        let removed = mempool.remove_tx(&sample_id(1));

        assert!(removed);
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
        assert!(!mempool.contains(&sample_id(1)));
        assert!(mempool.contains(&sample_id(2)));
        assert_eq!(mempool.lifecycle_stats().explicit_removals, 1);
    }

    #[test]
    fn clear_resets_live_state_and_preserves_counters() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2]).unwrap();
        mempool.add_tx(sample_id(2), vec![3]).unwrap();

        mempool.clear();

        assert_eq!(mempool.len(), 0);
        assert_eq!(mempool.total_bytes(), 0);
        assert!(mempool.is_empty());
        assert_eq!(mempool.lifecycle_stats().clears, 1);
    }

    #[test]
    fn ids_snapshot_matches_fifo_order() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1]).unwrap();
        mempool.add_tx(sample_id(2), vec![2]).unwrap();
        mempool.add_tx(sample_id(3), vec![3]).unwrap();

        let ids = mempool.ids_in_order();

        assert_eq!(ids, vec![sample_id(1), sample_id(2), sample_id(3)]);
    }

    #[test]
    fn stats_snapshot_matches_runtime_state() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2, 3]).unwrap();
        let stats = mempool.stats();

        assert_eq!(stats.len, 1);
        assert_eq!(stats.total_bytes, 3);
        assert_eq!(stats.max_txs, 3);
        assert_eq!(stats.max_tx_size, 16);
        assert_eq!(stats.max_total_bytes, 32);
        assert_eq!(stats.remaining_tx_capacity, 2);
        assert_eq!(stats.remaining_byte_capacity, 29);
        assert_eq!(stats.lifecycle_stats.accepted, 1);
    }

    #[test]
    fn explicit_metadata_is_preserved_on_collection() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let meta = AdmissionMeta::now(AdmissionSource::P2P, AdmissionPriority::High);

        mempool
            .add_tx_with_meta(sample_id(9), vec![1, 2, 3], meta)
            .unwrap();

        let collected = mempool.collect(1);
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].meta.source, AdmissionSource::P2P);
        assert_eq!(collected[0].meta.priority, AdmissionPriority::High);
    }

    #[test]
    fn accepted_by_source_is_tracked() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx_with_meta(
                sample_id(1),
                vec![1],
                AdmissionMeta::now(AdmissionSource::Rpc, AdmissionPriority::Normal),
            )
            .unwrap();
        mempool
            .add_tx_with_meta(
                sample_id(2),
                vec![2],
                AdmissionMeta::now(AdmissionSource::P2P, AdmissionPriority::Normal),
            )
            .unwrap();

        let by_source = mempool.accepted_by_source();
        assert_eq!(by_source.rpc, 1);
        assert_eq!(by_source.p2p, 1);
    }
}
