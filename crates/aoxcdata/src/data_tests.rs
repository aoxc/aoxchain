#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after unix epoch")
            .as_nanos();

        let dir = std::env::temp_dir().join(format!("aoxcdata_{label}_{nanos}"));
        fs::create_dir_all(&dir).expect("temporary directory must be created");
        dir
    }

    fn make_block(height: u64, parent_hash_hex: &str, payload: &[u8]) -> BlockEnvelope {
        let block_hash_hex = canonical_block_envelope_hash_hex(height, parent_hash_hex, payload)
            .expect("block hash computation must succeed");

        BlockEnvelope {
            height,
            block_hash_hex,
            parent_hash_hex: parent_hash_hex.to_owned(),
            payload: payload.to_vec(),
        }
    }

    #[test]
    fn canonical_block_hash_is_deterministic_for_identical_inputs() {
        let parent_hash_hex = "00".repeat(32);
        let fixtures = vec![
            (1u64, b"alpha".to_vec()),
            (2u64, b"beta".to_vec()),
            (5u64, b"deterministic-payload".to_vec()),
            (99u64, vec![0u8, 1, 2, 3, 4, 5, 6, 7]),
        ];

        for (height, payload) in fixtures {
            let first = canonical_block_envelope_hash_hex(height, &parent_hash_hex, &payload)
                .expect("canonical block hash must compute");
            let second = canonical_block_envelope_hash_hex(height, &parent_hash_hex, &payload)
                .expect("canonical block hash must compute");
            assert_eq!(first, second);
        }
    }

    #[test]
    fn block_roundtrip_by_height_succeeds() {
        let dir = unique_temp_dir("roundtrip");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block = make_block(1, &"00".repeat(32), b"hello");
        let meta = store.put_block(&block).expect("put block must succeed");
        assert!(meta.cid.starts_with("bafy"));

        let loaded = store
            .get_block_by_height(1)
            .expect("block retrieval by height must succeed");
        assert_eq!(loaded, block);

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn duplicate_height_is_rejected() {
        let dir = unique_temp_dir("dup_height");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block_a = make_block(1, &"00".repeat(32), b"alpha");
        let block_b = make_block(1, &"00".repeat(32), b"beta");

        store.put_block(&block_a).expect("first block must succeed");

        let err = store
            .put_block(&block_b)
            .expect_err("duplicate height must fail");

        match err {
            DataError::AlreadyExists(message) => assert!(message.contains("height")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn duplicate_hash_is_rejected() {
        let dir = unique_temp_dir("dup_hash");
        let index = FileMetaIndexStore::new(dir.join("index"), IndexBackend::Redb)
            .expect("index init must succeed");

        let meta_a = BlockMeta {
            height: 1,
            cid: "bafyalpha".to_owned(),
            block_hash_hex: "aa".repeat(32),
            parent_hash_hex: "00".repeat(32),
            created_at_unix: 1,
        };

        let meta_b = BlockMeta {
            height: 2,
            cid: "bafybeta".to_owned(),
            block_hash_hex: "aa".repeat(32),
            parent_hash_hex: "11".repeat(32),
            created_at_unix: 2,
        };

        index.put_meta(&meta_a).expect("first meta must succeed");

        let err = index
            .put_meta(&meta_b)
            .expect_err("duplicate hash must fail");

        match err {
            DataError::AlreadyExists(message) => assert!(message.contains("hash")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn invalid_block_hash_is_rejected() {
        let block = BlockEnvelope {
            height: 1,
            block_hash_hex: "ff".repeat(32),
            parent_hash_hex: "00".repeat(32),
            payload: b"payload".to_vec(),
        };

        let err = block.validate().expect_err("invalid block hash must fail");
        match err {
            DataError::Integrity(message) => assert!(message.contains("block hash mismatch")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn tampered_blob_is_detected_by_integrity_verification() {
        let dir = unique_temp_dir("tampered_blob");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block = make_block(1, &"00".repeat(32), b"hello");
        let meta = store.put_block(&block).expect("put block must succeed");

        let path = store.ipfs.block_path(&meta.cid);
        write_atomic(&path, b"{\"tampered\":true}").expect("tampered write must succeed");

        let err = store
            .ipfs
            .get_block(&meta.cid)
            .expect_err("tampered blob must fail");

        match err {
            DataError::Integrity(_) | DataError::Corrupted(_) => {}
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn journal_replay_restores_index_state() {
        let dir = unique_temp_dir("journal_replay");
        let index_dir = dir.join("index");

        {
            let index = FileMetaIndexStore::new(&index_dir, IndexBackend::Sqlite)
                .expect("index init must succeed");

            let meta = BlockMeta {
                height: 7,
                cid: "bafyrestore".to_owned(),
                block_hash_hex: "ab".repeat(32),
                parent_hash_hex: "cd".repeat(32),
                created_at_unix: 42,
            };

            index.put_meta(&meta).expect("meta insert must succeed");
        }

        let reopened = FileMetaIndexStore::new(&index_dir, IndexBackend::Sqlite)
            .expect("index reopen must succeed");

        let loaded = reopened
            .get_by_height(7)
            .expect("replayed entry must exist");

        assert_eq!(loaded.cid, "bafyrestore");
        assert_eq!(loaded.block_hash_hex, "ab".repeat(32));

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn corrupted_journal_is_rejected_on_open() {
        let dir = unique_temp_dir("corrupted_journal");
        let index_dir = dir.join("index");
        let index = FileMetaIndexStore::new(&index_dir, IndexBackend::Redb)
            .expect("index init must succeed");

        index.compact().expect("compact must succeed");

        let journal_path = index_dir.join("meta_redb_journal.log");
        write_atomic(&journal_path, b"{invalid json\n")
            .expect("corrupt journal write must succeed");

        let err = FileMetaIndexStore::new(&index_dir, IndexBackend::Redb)
            .expect_err("corrupted journal must fail");

        match err {
            DataError::Corrupted(message) => assert!(message.contains("invalid journal record")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn non_genesis_block_requires_parent_height_to_exist() {
        let dir = unique_temp_dir("missing_parent");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block = make_block(2, &"ab".repeat(32), b"orphan");
        let err = store
            .put_block(&block)
            .expect_err("orphan block must be rejected");

        match err {
            DataError::InvalidInput(message) => assert!(message.contains("missing parent block")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn non_genesis_block_requires_parent_hash_match() {
        let dir = unique_temp_dir("parent_mismatch");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let genesis = make_block(1, &"00".repeat(32), b"genesis");
        store.put_block(&genesis).expect("genesis put must succeed");

        let bad_child = make_block(2, &"ff".repeat(32), b"child");
        let err = store
            .put_block(&bad_child)
            .expect_err("child with mismatched parent hash must fail");

        match err {
            DataError::Integrity(message) => assert!(message.contains("parent hash mismatch")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn genesis_block_requires_zero_parent_hash() {
        let dir = unique_temp_dir("genesis_parent");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let bad_genesis = make_block(1, &"11".repeat(32), b"genesis");
        let err = store
            .put_block(&bad_genesis)
            .expect_err("genesis with non-zero parent hash must fail");

        match err {
            DataError::InvalidInput(message) => {
                assert!(message.contains("genesis block parent hash"))
            }
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }
}
