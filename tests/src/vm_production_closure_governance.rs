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
