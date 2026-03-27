// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

mod common;

use std::fs;
use std::path::PathBuf;

use aoxcontract::{ContractId, canonical};

#[test]
fn golden_manifest_and_id_vectors_remain_stable() {
    let manifest = common::sample_manifest();
    let canonical =
        String::from_utf8(canonical::canonical_manifest_bytes(&manifest).unwrap()).unwrap();
    let contract_id = ContractId::derive(&manifest).unwrap();

    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let fixture_manifest = fs::read_to_string(fixture_dir.join("valid_manifest.json")).unwrap();
    let fixture_id = fs::read_to_string(fixture_dir.join("contract_id.txt")).unwrap();

    assert_eq!(canonical, fixture_manifest.trim());
    assert_eq!(contract_id.0, fixture_id.trim());
}
