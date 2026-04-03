// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::fs;

#[test]
fn production_closure_gate_enforces_all_required_classes() {
    let script = fs::read_to_string("../scripts/validation/aoxcvm_production_closure_gate.sh")
        .expect("production closure gate script must be readable");

    for required in [
        "run_gate \"test\" ./scripts/validation/aoxcvm_phase3_gate.sh",
        "run_gate \"audit\" cargo audit",
        "run_gate \"rehearsal\" ./scripts/validation/os_compatibility_gate.sh",
        "run_gate \"evidence\" test -f",
        "\"policy\": \"full closure requires test/audit/rehearsal/evidence to all be PASS\"",
    ] {
        assert!(
            script.contains(required),
            "production closure gate is missing required guard: {required}"
        );
    }
}

#[test]
fn quantum_full_flow_routes_through_production_closure_gate() {
    let makefile = fs::read_to_string("../Makefile").expect("Makefile must be readable");

    assert!(
        makefile.contains("@$(MAKE) --no-print-directory aoxcvm-production-closure-gate"),
        "quantum-full flow must invoke full AOXCVM production closure gate"
    );
    assert!(
        makefile.contains("aoxcvm-production-closure-gate:"),
        "Makefile must expose a dedicated production closure target"
    );
}

#[test]
fn makefile_help_and_phony_surfaces_include_production_closure_target() {
    let makefile = fs::read_to_string("../Makefile").expect("Makefile must be readable");

    assert!(
        makefile.contains("aoxcvm-production-closure-gate"),
        "Makefile must expose production closure target in operator-visible surfaces"
    );
    assert!(
        makefile.contains("@printf \"  make aoxcvm-production-closure-gate\\n\""),
        "help output must advertise the production closure target"
    );
}

#[test]
fn production_closure_gate_is_fail_closed_and_emits_summary() {
    let script = fs::read_to_string("../scripts/validation/aoxcvm_production_closure_gate.sh")
        .expect("production closure gate script must be readable");

    for required in [
        "SUMMARY_FILE=\"${ARTIFACT_DIR}/production-closure-summary.json\"",
        "overall_status=\"failed\"",
        "if [[ \"${overall_status}\" != \"passed\" ]]; then",
        "exit 1",
    ] {
        assert!(
            script.contains(required),
            "production closure gate must remain fail-closed with summary evidence: {required}"
        );
    }
}

#[test]
fn production_closure_docs_define_the_same_gate_contract() {
    let docs =
        fs::read_to_string("../scripts/READ.md").expect("scripts READ documentation must exist");

    for required in [
        "scripts/validation/aoxcvm_production_closure_gate.sh",
        "make aoxcvm-production-closure-gate",
        "test (`scripts/validation/aoxcvm_phase3_gate.sh`)",
        "audit (`cargo audit`)",
        "rehearsal (`scripts/validation/os_compatibility_gate.sh`)",
        "production-closure-summary.json",
    ] {
        assert!(
            docs.contains(required),
            "scripts/READ.md must keep closure contract documented: {required}"
        );
    }
}

#[test]
fn generated_inventory_reports_non_zero_workspace_test_surfaces() {
    let inventory = fs::read_to_string("../artifacts/testing/test_inventory.json")
        .expect("test inventory artifact must be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&inventory).expect("test inventory artifact must be valid JSON");

    let workspace_integration_tests = parsed["counts"]["workspace_integration_tests"]
        .as_u64()
        .expect("workspace_integration_tests must be numeric");
    let crate_scoped_tests = parsed["counts"]["crate_scoped_tests"]
        .as_u64()
        .expect("crate_scoped_tests must be numeric");

    assert!(
        workspace_integration_tests > 0,
        "workspace integration test inventory must remain non-zero"
    );
    assert!(
        crate_scoped_tests > 0,
        "crate-scoped test inventory must remain non-zero"
    );
}
