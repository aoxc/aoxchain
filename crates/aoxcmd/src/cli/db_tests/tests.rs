#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_INDEX_BACKEND, backend_label, build_status, cmd_db_compact, cmd_db_get_hash,
        cmd_db_get_height, cmd_db_init, cmd_db_put_block, cmd_db_status, count_ipfs_objects,
        db_root, open_store, parse_backend, parse_required_arg, parse_u64_arg,
    };
    use crate::test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock};
    use aoxcdata::{HybridDataStore, IndexBackend};
    use sha2::{Digest, Sha256};
    use std::{fs, path::PathBuf};

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label).expect("test home should be created");
        let _guard = AoxcHomeGuard::install(&_lock, home.path());
        test(&home)
    }

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    fn backend_args(backend: IndexBackend) -> Vec<String> {
        vec!["--backend".to_string(), backend_label(backend).to_string()]
    }

    fn zero_hash() -> String {
        "00".repeat(32)
    }

    fn sample_hash(height: u64, parent_hash: &str, payload: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_BLOCK_V1");
        hasher.update(height.to_le_bytes());
        hasher.update(hex::decode(parent_hash).expect("parent hash should decode"));
        hasher.update((payload.len() as u64).to_le_bytes());
        hasher.update(payload);
        hex::encode(hasher.finalize())
    }

    fn sample_block_json(height: u64, parent_hash: &str, payload: &[u8]) -> (String, String) {
        let block_hash = sample_hash(height, parent_hash, payload);
        let payload_json = payload
            .iter()
            .map(|byte| byte.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let block_json = format!(
            "{{\"height\":{height},\"block_hash_hex\":\"{block_hash}\",\"parent_hash_hex\":\"{parent_hash}\",\"payload\":[{payload_json}]}}"
        );

        (block_hash, block_json)
    }

    fn write_block_fixture(home: &TestHome, name: &str, content: &str) -> PathBuf {
        let block_file = home.path().join("support").join(name);
        fs::create_dir_all(
            block_file
                .parent()
                .expect("block fixture parent directory must exist"),
        )
        .expect("block fixture directory should be created");
        fs::write(&block_file, content).expect("block fixture should be written");
        block_file
    }

    fn open_store_for_backend(backend: IndexBackend) -> (HybridDataStore, PathBuf) {
        open_store(backend).unwrap_or_else(|err| {
            panic!(
                "store should open under the active AOXC root for backend {}: {}",
                backend_label(backend),
                err
            )
        })
    }

    fn persist_block_fixture(
        home: &TestHome,
        name: &str,
        height: u64,
        parent_hash: &str,
        payload: &[u8],
        backend: IndexBackend,
    ) -> String {
        let (block_hash, block_json) = sample_block_json(height, parent_hash, payload);
        let block_file = write_block_fixture(home, name, &block_json);

        let args = vec![
            "--backend".to_string(),
            backend_label(backend).to_string(),
            "--block-file".to_string(),
            block_file.display().to_string(),
        ];

        cmd_db_put_block(&args).expect("block fixture should persist successfully");
        block_hash
    }

    fn assert_block_present_by_height(expected_height: u64, backend: IndexBackend) {
        let (store, root) = open_store_for_backend(backend);
        let block = store
            .get_block_by_height(expected_height)
            .unwrap_or_else(|err| {
                panic!(
                    "expected block at height {} to be readable from {} using {} backend: {}",
                    expected_height,
                    root.display(),
                    backend_label(backend),
                    err
                )
            });

        assert_eq!(block.height, expected_height);
    }

    fn assert_block_present_by_hash(
        expected_hash: &str,
        expected_height: u64,
        backend: IndexBackend,
    ) {
        let (store, root) = open_store_for_backend(backend);
        let block = store
            .get_block_by_hash(expected_hash)
            .unwrap_or_else(|err| {
                panic!(
                    "expected block hash {} to be readable from {} using {} backend: {}",
                    expected_hash,
                    root.display(),
                    backend_label(backend),
                    err
                )
            });

        assert_eq!(block.height, expected_height);
        assert_eq!(block.block_hash_hex, expected_hash);
    }

    #[test]
    fn backend_parser_defaults_to_redb_when_backend_is_omitted() {
        let parsed = parse_backend(&[]).expect("default backend should parse");
        assert_eq!(parsed, DEFAULT_INDEX_BACKEND);
        assert_eq!(backend_label(parsed), "redb");
    }

    #[test]
    fn backend_parser_accepts_known_values() {
        let sqlite = parse_backend(&args(&["--backend", "sqlite"]))
            .expect("sqlite backend should parse successfully");
        let redb = parse_backend(&args(&["--backend", "redb"]))
            .expect("redb backend should parse successfully");

        assert_eq!(sqlite, IndexBackend::Sqlite);
        assert_eq!(redb, IndexBackend::Redb);
    }

    #[test]
    fn backend_parser_rejects_unknown_values() {
        let error = parse_backend(&args(&["--backend", "rocksdb"]))
            .expect_err("unknown backend should be rejected");
        assert_eq!(error.code(), "AOXC-USG-002");
        assert!(format!("{error}").contains("Invalid --backend value"));
    }

    #[test]
    fn parse_required_arg_rejects_missing_values() {
        let error = parse_required_arg(&[], "--hash")
            .expect_err("missing required argument should be rejected");
        assert_eq!(error.code(), "AOXC-USG-002");
        assert!(format!("{error}").contains("--hash"));
    }

    #[test]
    fn parse_u64_arg_rejects_missing_values() {
        let error = parse_u64_arg(&[], "--height")
            .expect_err("missing numeric argument should be rejected");
        assert_eq!(error.code(), "AOXC-USG-002");
        assert!(format!("{error}").contains("--height"));
    }

    #[test]
    fn parse_u64_arg_rejects_non_numeric_values() {
        let error = parse_u64_arg(&args(&["--height", "not-a-number"]), "--height")
            .expect_err("non-numeric value should be rejected");
        assert_eq!(error.code(), "AOXC-USG-002");
        assert!(format!("{error}").contains("Invalid numeric value"));
    }

    #[test]
    fn db_root_resolves_inside_active_test_home_runtime_db() {
        with_test_home("db-root-resolution", |home| {
            let root = db_root().expect("db root should resolve");
            assert_eq!(root, home.path().join("runtime").join("db"));
            assert!(root.is_dir());
        });
    }

    #[test]
    fn build_status_reports_expected_backend_and_root() {
        with_test_home("db-status-build", |home| {
            let root = db_root().expect("db root should resolve");
            let status =
                build_status(IndexBackend::Redb, &root).expect("status should build successfully");

            assert_eq!(status.backend, "redb");
            assert_eq!(
                status.db_root,
                home.path().join("runtime").join("db").display().to_string()
            );
            assert_eq!(status.ipfs_object_count, 0);
        });
    }

    #[test]
    fn count_ipfs_objects_returns_zero_when_directory_is_absent() {
        with_test_home("ipfs-empty", |_home| {
            let root = db_root().expect("db root should resolve");
            assert_eq!(
                count_ipfs_objects(&root).expect("ipfs scan should succeed"),
                0
            );
        });
    }

    #[test]
    fn count_ipfs_objects_counts_materialized_objects() {
        with_test_home("ipfs-count", |_home| {
            let root = db_root().expect("db root should resolve");
            let ipfs_dir = root.join("ipfs");

            fs::create_dir_all(&ipfs_dir).expect("ipfs directory should be created");
            fs::write(ipfs_dir.join("obj-1"), b"alpha").expect("first object should be written");
            fs::write(ipfs_dir.join("obj-2"), b"beta").expect("second object should be written");

            assert_eq!(
                count_ipfs_objects(&root).expect("ipfs scan should succeed"),
                2
            );
        });
    }

    #[test]
    fn db_init_and_status_succeed_under_isolated_aoxc_root_with_default_backend() {
        with_test_home("db-init-status-default", |home| {
            cmd_db_init(&[]).expect("db init should succeed");
            cmd_db_status(&[]).expect("db status should succeed");

            let root = home.path().join("runtime").join("db");
            assert!(root.is_dir());

            let status = build_status(DEFAULT_INDEX_BACKEND, &root)
                .expect("status should build successfully");
            assert_eq!(status.backend, "redb");
        });
    }

    #[test]
    fn db_roundtrip_flow_with_redb_backend() {
        with_test_home("db-roundtrip-redb", |home| {
            let parent = zero_hash();
            let payload = br#"{"tx":"db-smoke-redb"}"#;
            let (block_hash, block_json) = sample_block_json(1, &parent, payload);
            let block_file = write_block_fixture(home, "sample-block-redb.json", &block_json);

            let init_args = backend_args(IndexBackend::Redb);
            cmd_db_init(&init_args).expect("db init should succeed");

            let put_args = vec![
                "--backend".to_string(),
                "redb".to_string(),
                "--block-file".to_string(),
                block_file.display().to_string(),
            ];
            cmd_db_put_block(&put_args).expect("db put should succeed");

            let get_height_args = vec![
                "--backend".to_string(),
                "redb".to_string(),
                "--height".to_string(),
                "1".to_string(),
            ];
            cmd_db_get_height(&get_height_args).expect("db get by height should succeed");

            let get_hash_args = vec![
                "--backend".to_string(),
                "redb".to_string(),
                "--hash".to_string(),
                block_hash.clone(),
            ];
            cmd_db_get_hash(&get_hash_args).expect("db get by hash should succeed");

            assert_block_present_by_height(1, IndexBackend::Redb);
            assert_block_present_by_hash(&block_hash, 1, IndexBackend::Redb);
        });
    }

    #[test]
    fn db_roundtrip_flow_with_sqlite_backend_remains_supported() {
        with_test_home("db-roundtrip-sqlite", |home| {
            let parent = zero_hash();
            let payload = br#"{"tx":"db-smoke-sqlite"}"#;
            let (block_hash, block_json) = sample_block_json(1, &parent, payload);
            let block_file = write_block_fixture(home, "sample-block-sqlite.json", &block_json);

            let init_args = backend_args(IndexBackend::Sqlite);
            cmd_db_init(&init_args).expect("db init should succeed");

            let put_args = vec![
                "--backend".to_string(),
                "sqlite".to_string(),
                "--block-file".to_string(),
                block_file.display().to_string(),
            ];
            cmd_db_put_block(&put_args).expect("db put should succeed");

            let get_height_args = vec![
                "--backend".to_string(),
                "sqlite".to_string(),
                "--height".to_string(),
                "1".to_string(),
            ];
            cmd_db_get_height(&get_height_args).expect("db get by height should succeed");

            let get_hash_args = vec![
                "--backend".to_string(),
                "sqlite".to_string(),
                "--hash".to_string(),
                block_hash.clone(),
            ];
            cmd_db_get_hash(&get_hash_args).expect("db get by hash should succeed");

            assert_block_present_by_height(1, IndexBackend::Sqlite);
            assert_block_present_by_hash(&block_hash, 1, IndexBackend::Sqlite);
        });
    }

    #[test]
    fn db_put_and_follow_up_reads_use_the_same_isolated_root_for_redb() {
        with_test_home("db-root-consistency-redb", |home| {
            let parent = zero_hash();
            let payload = br#"{"tx":"consistency-check-redb"}"#;
            let block_hash = persist_block_fixture(
                home,
                "consistency-block-redb.json",
                1,
                &parent,
                payload,
                IndexBackend::Redb,
            );

            let root = db_root().expect("db root should resolve");
            assert_eq!(root, home.path().join("runtime").join("db"));

            assert_block_present_by_height(1, IndexBackend::Redb);
            assert_block_present_by_hash(&block_hash, 1, IndexBackend::Redb);
        });
    }

    #[test]
    fn db_put_and_follow_up_reads_use_the_same_isolated_root_for_sqlite() {
        with_test_home("db-root-consistency-sqlite", |home| {
            let parent = zero_hash();
            let payload = br#"{"tx":"consistency-check-sqlite"}"#;
            let block_hash = persist_block_fixture(
                home,
                "consistency-block-sqlite.json",
                1,
                &parent,
                payload,
                IndexBackend::Sqlite,
            );

            let root = db_root().expect("db root should resolve");
            assert_eq!(root, home.path().join("runtime").join("db"));

            assert_block_present_by_height(1, IndexBackend::Sqlite);
            assert_block_present_by_hash(&block_hash, 1, IndexBackend::Sqlite);
        });
    }

    #[test]
    fn db_put_rejects_duplicate_block_reinsertion_for_redb() {
        with_test_home("db-duplicate-height-redb", |home| {
            let parent = zero_hash();
            let payload = br#"{"tx":"duplicate-height-redb"}"#;
            let (_block_hash, block_json) = sample_block_json(1, &parent, payload);
            let block_file = write_block_fixture(home, "duplicate-height-redb.json", &block_json);

            let first_args = vec![
                "--backend".to_string(),
                "redb".to_string(),
                "--block-file".to_string(),
                block_file.display().to_string(),
            ];
            cmd_db_put_block(&first_args).expect("first db put should succeed");

            let second_args = vec![
                "--backend".to_string(),
                "redb".to_string(),
                "--block-file".to_string(),
                block_file.display().to_string(),
            ];
            let error =
                cmd_db_put_block(&second_args).expect_err("duplicate insertion must be rejected");

            assert_eq!(error.code(), "AOXC-LED-001");
            assert!(format!("{error}").contains("Failed to persist block"));
        });
    }

    #[test]
    fn db_put_rejects_duplicate_block_reinsertion_for_sqlite() {
        with_test_home("db-duplicate-height-sqlite", |home| {
            let parent = zero_hash();
            let payload = br#"{"tx":"duplicate-height-sqlite"}"#;
            let (_block_hash, block_json) = sample_block_json(1, &parent, payload);
            let block_file = write_block_fixture(home, "duplicate-height-sqlite.json", &block_json);

            let first_args = vec![
                "--backend".to_string(),
                "sqlite".to_string(),
                "--block-file".to_string(),
                block_file.display().to_string(),
            ];
            cmd_db_put_block(&first_args).expect("first db put should succeed");

            let second_args = vec![
                "--backend".to_string(),
                "sqlite".to_string(),
                "--block-file".to_string(),
                block_file.display().to_string(),
            ];
            let error =
                cmd_db_put_block(&second_args).expect_err("duplicate insertion must be rejected");

            assert_eq!(error.code(), "AOXC-LED-001");
            assert!(format!("{error}").contains("Failed to persist block"));
        });
    }

    #[test]
    fn db_put_requires_block_file_argument() {
        with_test_home("db-put-missing-arg", |_home| {
            let error =
                cmd_db_put_block(&[]).expect_err("missing block-file argument must be rejected");
            assert_eq!(error.code(), "AOXC-USG-002");
            assert!(format!("{error}").contains("--block-file"));
        });
    }

    #[test]
    fn db_put_rejects_missing_block_file_on_disk() {
        with_test_home("db-put-missing-file", |home| {
            let missing = home.path().join("support").join("does-not-exist.json");

            let error = cmd_db_put_block(&[
                "--backend".to_string(),
                "redb".to_string(),
                "--block-file".to_string(),
                missing.display().to_string(),
            ])
            .expect_err("missing block file must be rejected");

            assert_eq!(error.code(), "AOXC-FS-001");
            assert!(format!("{error}").contains("Failed to read block file"));
        });
    }

    #[test]
    fn db_put_rejects_invalid_block_json() {
        with_test_home("db-put-invalid-json", |home| {
            let block_file =
                write_block_fixture(home, "invalid-block.json", r#"{"height":not-a-number}"#);

            let error = cmd_db_put_block(&[
                "--backend".to_string(),
                "redb".to_string(),
                "--block-file".to_string(),
                block_file.display().to_string(),
            ])
            .expect_err("invalid block json must be rejected");

            assert_eq!(error.code(), "AOXC-USG-002");
            assert!(format!("{error}").contains("Invalid block JSON"));
        });
    }

    #[test]
    fn db_get_height_rejects_missing_argument() {
        with_test_home("db-get-height-missing-arg", |_home| {
            let error =
                cmd_db_get_height(&[]).expect_err("missing height argument must be rejected");
            assert_eq!(error.code(), "AOXC-USG-002");
            assert!(format!("{error}").contains("--height"));
        });
    }

    #[test]
    fn db_get_height_rejects_invalid_argument() {
        with_test_home("db-get-height-invalid-arg", |_home| {
            let error = cmd_db_get_height(&args(&["--height", "abc"]))
                .expect_err("invalid height argument must be rejected");
            assert_eq!(error.code(), "AOXC-USG-002");
            assert!(format!("{error}").contains("Invalid numeric value"));
        });
    }

    #[test]
    fn db_get_height_returns_ledger_error_when_block_is_absent() {
        with_test_home("db-get-height-absent", |_home| {
            let error = cmd_db_get_height(&[
                "--backend".to_string(),
                "redb".to_string(),
                "--height".to_string(),
                "999".to_string(),
            ])
            .expect_err("absent height lookup must fail");

            assert_eq!(error.code(), "AOXC-LED-001");
            assert!(format!("{error}").contains("Failed to load block at height 999"));
        });
    }

    #[test]
    fn db_get_hash_rejects_missing_argument() {
        with_test_home("db-get-hash-missing-arg", |_home| {
            let error = cmd_db_get_hash(&[]).expect_err("missing hash argument must be rejected");
            assert_eq!(error.code(), "AOXC-USG-002");
            assert!(format!("{error}").contains("--hash"));
        });
    }

    #[test]
    fn db_get_hash_returns_ledger_error_when_block_is_absent() {
        with_test_home("db-get-hash-absent", |_home| {
            let missing_hash = "aa".repeat(32);
            let error = cmd_db_get_hash(&[
                "--backend".to_string(),
                "redb".to_string(),
                "--hash".to_string(),
                missing_hash,
            ])
            .expect_err("absent hash lookup must fail");

            assert_eq!(error.code(), "AOXC-LED-001");
            assert!(format!("{error}").contains("Failed to load block hash"));
        });
    }

    #[test]
    fn db_compact_succeeds_after_persisting_a_genesis_block_with_redb() {
        with_test_home("db-compact-redb", |home| {
            cmd_db_init(&backend_args(IndexBackend::Redb)).expect("db init should succeed");

            let block_hash = persist_block_fixture(
                home,
                "compact-block-redb.json",
                1,
                &zero_hash(),
                br#"{"tx":"compact-me-redb"}"#,
                IndexBackend::Redb,
            );

            cmd_db_compact(&backend_args(IndexBackend::Redb)).expect("db compact should succeed");

            let root = home.path().join("runtime").join("db");
            let status =
                build_status(IndexBackend::Redb, &root).expect("status should build after compact");
            assert_eq!(status.backend, "redb");

            assert_block_present_by_height(1, IndexBackend::Redb);
            assert_block_present_by_hash(&block_hash, 1, IndexBackend::Redb);
        });
    }

    #[test]
    fn db_compact_succeeds_after_persisting_a_genesis_block_with_sqlite() {
        with_test_home("db-compact-sqlite", |home| {
            cmd_db_init(&backend_args(IndexBackend::Sqlite)).expect("db init should succeed");

            let block_hash = persist_block_fixture(
                home,
                "compact-block-sqlite.json",
                1,
                &zero_hash(),
                br#"{"tx":"compact-me-sqlite"}"#,
                IndexBackend::Sqlite,
            );

            cmd_db_compact(&backend_args(IndexBackend::Sqlite)).expect("db compact should succeed");

            let root = home.path().join("runtime").join("db");
            let status = build_status(IndexBackend::Sqlite, &root)
                .expect("status should build after compact");
            assert_eq!(status.backend, "sqlite");

            assert_block_present_by_height(1, IndexBackend::Sqlite);
            assert_block_present_by_hash(&block_hash, 1, IndexBackend::Sqlite);
        });
    }

    #[test]
    fn open_store_uses_the_active_aoxc_test_root_for_default_backend() {
        with_test_home("open-store-default-backend", |home| {
            let (_store, root) = open_store_for_backend(DEFAULT_INDEX_BACKEND);
            assert_eq!(root, home.path().join("runtime").join("db"));
            assert!(root.is_dir());
        });
    }
}
