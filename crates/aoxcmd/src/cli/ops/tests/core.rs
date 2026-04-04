use super::*;

#[test]
fn parse_positive_u64_arg_rejects_zero() {
    let error = parse_positive_u64_arg(&args(&["--rounds", "0"]), "--rounds", 10, "node run")
        .expect_err("zero rounds must fail");
    assert_eq!(error.code(), "AOXC-USG-002");
}
#[test]
fn parse_required_or_default_text_arg_rejects_blank_value() {
    let error = parse_required_or_default_text_arg(&args(&["--to", "   "]), "--to", "ops", false)
        .expect_err("blank target must fail");
    assert_eq!(error.code(), "AOXC-USG-002");
}
#[test]
fn historical_tx_hashes_extracts_payload_from_block_envelope() {
    let envelope = BlockEnvelope {
        height: 7,
        block_hash_hex: "e3c0fdbff6f570f0449557cb9a9d8bc95eeb5d1f7e5bc8f2a580f7f7f6f7a9a7"
            .to_string(),
        parent_hash_hex: "7f6f7a9ae3c0fdbff6f570f0449557cb9a9d8bc95eeb5d1f7e5bc8f2a580f7f7"
            .to_string(),
        payload: br#"{"body":{"sections":[{"payload":"tx-demo-7"}]}}"#.to_vec(),
    };

    let tx_hashes = historical_tx_hashes(&envelope);
    assert_eq!(tx_hashes, vec!["tx-demo-7".to_string()]);
}

#[test]
fn faucet_claim_rejects_amount_above_max_without_force() {
    let state = FaucetState::default();
    let decision = evaluate_faucet_claim(
        &state,
        "alice",
        state.max_claim_amount + 1,
        1_775_238_343,
        false,
        Some(5_000_000),
        "testnet",
    );
    assert!(!decision.allowed);
    assert!(
        decision
            .denied_reason
            .expect("reason should exist")
            .contains("max claim amount")
    );
}

#[test]
fn faucet_claim_rejects_when_cooldown_active() {
    let mut state = FaucetState::default();
    state.claims.push(FaucetClaimRecord {
        account_id: "alice".to_string(),
        amount: 50,
        claimed_at: 1_775_238_343,
        tx_hash: "tx-1".to_string(),
        status: "confirmed".to_string(),
    });
    let decision = evaluate_faucet_claim(
        &state,
        "alice",
        50,
        1_775_238_343 + 100,
        false,
        Some(5_000_000),
        "testnet",
    );
    assert!(!decision.allowed);
    assert!(decision.cooldown_remaining_secs > 0);
}

#[test]
fn release_evidence_requires_expected_bundle_files() {
    let dir = unique_dir("release-evidence");
    touch(&dir.join("release-evidence-20260323T000000Z.md"));
    touch(&dir.join("build-manifest-20260323T000000Z.json"));
    touch(&dir.join("compat-matrix-20260323T000000Z.json"));
    touch(&dir.join("production-audit-20260323T000000Z.json"));
    touch(&dir.join("sbom-20260323T000000Z.json"));
    touch(&dir.join("aoxc-20260323T000000Z.sig.status"));

    assert!(has_release_evidence(&dir));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn release_provenance_bundle_requires_expected_artifacts() {
    let dir = unique_dir("release-provenance");
    touch(&dir.join("provenance-20260323T000000Z.json"));
    touch(&dir.join("release-provenance-20260323T000000Z.json"));
    touch(&dir.join("release-sbom-20260323T000000Z.json"));
    touch(&dir.join("release-build-manifest-20260323T000000Z.json"));
    touch(&dir.join("release-signature-status-20260323T000000Z.txt"));

    assert!(has_release_provenance_bundle(&dir));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn production_closure_requires_all_operational_artifacts() {
    let dir = unique_dir("production-closure");
    for file in [
        "production-audit.json",
        "runtime-status.json",
        "soak-plan.json",
        "telemetry-snapshot.json",
        "aoxhub-rollout.json",
        "alert-rules.md",
    ] {
        touch(&dir.join(file));
    }

    assert!(has_production_closure_artifacts(&dir));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn matching_artifact_detects_expected_prefix_and_suffix() {
    let dir = unique_dir("matching-artifact");
    touch(&dir.join("provenance-20260323T000000Z.json"));

    assert!(has_matching_artifact(&dir, "provenance-", ".json"));
    assert!(!has_matching_artifact(&dir, "compat-matrix-", ".json"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn security_drill_artifact_requires_expected_scenarios() {
    let dir = unique_dir("security-drill");
    fs::create_dir_all(&dir).expect("fixture directory should be created");
    fs::write(
        dir.join("security-drill.json"),
        r#"{
  "status": "completed",
  "scenarios": ["penetration-baseline", "rpc-authz", "session-replay"]
}"#,
    )
    .expect("security drill artifact should be written");

    assert!(has_security_drill_artifact(&dir));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn desktop_wallet_compat_artifact_requires_all_surfaces() {
    let dir = unique_dir("desktop-wallet-compat");
    fs::create_dir_all(&dir).expect("fixture directory should be created");
    fs::write(
        dir.join("desktop-wallet-compat.json"),
        r#"{
  "status": "validated",
  "surfaces": ["desktop-wallet", "aoxhub", "mainnet", "testnet"]
}"#,
    )
    .expect("desktop wallet compatibility artifact should be written");

    assert!(has_desktop_wallet_compat_artifact(&dir));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn open_checklist_items_detects_unchecked_entries() {
    let dir = unique_dir("checklist-open");
    let checklist = dir.join("MAINNET_READINESS_CHECKLIST.md");
    fs::create_dir_all(&dir).expect("fixture directory should be created");
    fs::write(
        &checklist,
        "# checklist\n- [x] done\n- [ ] pending-1\n- [ ] pending-2\n",
    )
    .expect("checklist fixture should be written");

    let open = open_checklist_items(&checklist);
    assert_eq!(open.len(), 2);
    assert!(open.iter().any(|item| item == "pending-1"));
    assert!(open.iter().any(|item| item == "pending-2"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn open_checklist_items_returns_missing_marker_when_file_absent() {
    let path = unique_dir("checklist-missing").join("MAINNET_READINESS_CHECKLIST.md");
    let open = open_checklist_items(&path);
    assert_eq!(open.len(), 1);
    assert!(open[0].starts_with("missing-checklist:"));
}
