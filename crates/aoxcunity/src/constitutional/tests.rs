use crate::seal::QuorumCertificate;

use super::{
    ConstitutionalFinalityStage, ConstitutionalSeal, ConstitutionalValidationError,
    ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
};

fn sample_qc(block_hash: [u8; 32], height: u64, round: u64) -> QuorumCertificate {
    QuorumCertificate::new(
        block_hash,
        height,
        round,
        vec![[2u8; 32], [1u8; 32]],
        20,
        30,
        2,
        3,
    )
}

#[test]
fn legitimacy_certificate_hash_is_deterministic_for_signer_order() {
    let a = LegitimacyCertificate::new(
        [1u8; 32],
        7,
        [2u8; 32],
        [3u8; 32],
        [4u8; 32],
        vec![[9u8; 32], [8u8; 32]],
    );
    let b = LegitimacyCertificate::new(
        [1u8; 32],
        7,
        [2u8; 32],
        [3u8; 32],
        [4u8; 32],
        vec![[8u8; 32], [9u8; 32]],
    );

    assert_eq!(a.certificate_hash, b.certificate_hash);
    assert_eq!(a.signers, b.signers);
}

#[test]
fn legitimacy_validation_rejects_empty_signers() {
    let certificate =
        LegitimacyCertificate::new([1u8; 32], 1, [2u8; 32], [3u8; 32], [4u8; 32], vec![]);

    assert_eq!(
        certificate.validate(),
        Err(ConstitutionalValidationError::EmptySignerSet)
    );
}

#[test]
fn continuity_validation_rejects_empty_signers() {
    let certificate = ContinuityCertificate::new([1u8; 32], 4, 2, 7, 3, 10, vec![]);

    assert_eq!(
        certificate.validate(),
        Err(ConstitutionalValidationError::EmptySignerSet)
    );
}

#[test]
fn continuity_validation_rejects_zero_power() {
    let certificate = ContinuityCertificate::new([1u8; 32], 4, 2, 7, 3, 0, vec![[9u8; 32]]);

    assert_eq!(
        certificate.validate(),
        Err(ConstitutionalValidationError::ZeroObservedPower)
    );
}

#[test]
fn continuity_validation_rejects_non_forward_timeout_round() {
    let certificate = ContinuityCertificate::new([1u8; 32], 4, 2, 7, 2, 10, vec![[9u8; 32]]);

    assert_eq!(
        certificate.validate(),
        Err(ConstitutionalValidationError::InvalidTimeoutRound)
    );
}

#[test]
fn constitutional_seal_requires_matching_block_and_epoch() {
    let qc = sample_qc([5u8; 32], 11, 3);
    let execution = ExecutionCertificate::new(4, [6u8; 32], qc);
    let legitimacy = LegitimacyCertificate::new(
        [5u8; 32],
        4,
        [7u8; 32],
        [8u8; 32],
        [9u8; 32],
        vec![[1u8; 32]],
    );
    let continuity = ContinuityCertificate::new([5u8; 32], 11, 3, 5, 4, 10, vec![[1u8; 32]]);

    assert!(ConstitutionalSeal::compose(&execution, &legitimacy, &continuity).is_none());
    assert_eq!(
        ConstitutionalSeal::compose_strict(&execution, &legitimacy, &continuity),
        Err(ConstitutionalValidationError::ExecutionContinuityEpochMismatch)
    );
}

#[test]
fn constitutional_seal_composition_is_deterministic() {
    let qc = sample_qc([5u8; 32], 11, 3);
    let execution = ExecutionCertificate::new(4, [6u8; 32], qc);
    let legitimacy = LegitimacyCertificate::new(
        [5u8; 32],
        4,
        [7u8; 32],
        [8u8; 32],
        [9u8; 32],
        vec![[2u8; 32], [1u8; 32]],
    );
    let continuity =
        ContinuityCertificate::new([5u8; 32], 11, 3, 4, 4, 20, vec![[2u8; 32], [1u8; 32]]);

    let a = ConstitutionalSeal::compose_strict(&execution, &legitimacy, &continuity).unwrap();
    let b = ConstitutionalSeal::compose_strict(&execution, &legitimacy, &continuity).unwrap();

    assert_eq!(a, b);
}

#[test]
fn execution_certificate_hash_changes_when_epoch_or_validator_set_changes() {
    let qc = sample_qc([5u8; 32], 11, 3);
    let a = ExecutionCertificate::new(4, [6u8; 32], qc.clone());
    let b = ExecutionCertificate::new(5, [6u8; 32], qc.clone());
    let c = ExecutionCertificate::new(4, [7u8; 32], qc);

    assert_ne!(a.certificate_hash, b.certificate_hash);
    assert_ne!(a.certificate_hash, c.certificate_hash);
}

#[test]
fn continuity_certificate_hash_changes_when_timeout_round_or_power_changes() {
    let a = ContinuityCertificate::new([5u8; 32], 11, 3, 4, 4, 20, vec![[1u8; 32]]);
    let b = ContinuityCertificate::new([5u8; 32], 11, 3, 4, 5, 20, vec![[1u8; 32]]);
    let c = ContinuityCertificate::new([5u8; 32], 11, 3, 4, 4, 21, vec![[1u8; 32]]);

    assert_ne!(a.certificate_hash, b.certificate_hash);
    assert_ne!(a.certificate_hash, c.certificate_hash);
}

#[test]
fn constitutional_seal_hash_changes_when_input_certificate_changes() {
    let qc = sample_qc([5u8; 32], 11, 3);
    let execution_a = ExecutionCertificate::new(4, [6u8; 32], qc.clone());
    let execution_b = ExecutionCertificate::new(5, [6u8; 32], qc);
    let legitimacy = LegitimacyCertificate::new(
        [5u8; 32],
        4,
        [7u8; 32],
        [8u8; 32],
        [9u8; 32],
        vec![[1u8; 32]],
    );
    let continuity = ContinuityCertificate::new([5u8; 32], 11, 3, 4, 4, 20, vec![[1u8; 32]]);

    let seal_a =
        ConstitutionalSeal::compose_strict(&execution_a, &legitimacy, &continuity).unwrap();

    let legitimacy_b = LegitimacyCertificate::new(
        [5u8; 32],
        5,
        [7u8; 32],
        [8u8; 32],
        [9u8; 32],
        vec![[1u8; 32]],
    );
    let continuity_b = ContinuityCertificate::new([5u8; 32], 11, 3, 5, 4, 20, vec![[1u8; 32]]);
    let seal_b =
        ConstitutionalSeal::compose_strict(&execution_b, &legitimacy_b, &continuity_b).unwrap();

    assert_ne!(seal_a.seal_hash, seal_b.seal_hash);
}

#[test]
fn stage_evaluation_reports_constitutional_finality_only_when_all_inputs_exist() {
    let qc = sample_qc([5u8; 32], 11, 3);
    let execution = ExecutionCertificate::new(4, [6u8; 32], qc);
    let legitimacy = LegitimacyCertificate::new(
        [5u8; 32],
        4,
        [7u8; 32],
        [8u8; 32],
        [9u8; 32],
        vec![[1u8; 32]],
    );
    let continuity = ContinuityCertificate::new([5u8; 32], 11, 3, 4, 4, 20, vec![[1u8; 32]]);

    let report = ConstitutionalSeal::evaluate_stage(
        Some(&execution),
        Some(&legitimacy),
        Some(&continuity),
    );

    assert!(report.has_execution);
    assert!(report.has_legitimacy);
    assert!(report.has_continuity);
    assert_eq!(
        report.stage,
        ConstitutionalFinalityStage::ConstitutionallyFinal
    );
}

#[test]
fn stage_evaluation_distinguishes_legitimate_and_continuous_paths() {
    let qc = sample_qc([5u8; 32], 11, 3);
    let execution = ExecutionCertificate::new(4, [6u8; 32], qc);
    let legitimacy = LegitimacyCertificate::new(
        [5u8; 32],
        4,
        [7u8; 32],
        [8u8; 32],
        [9u8; 32],
        vec![[1u8; 32]],
    );
    let continuity = ContinuityCertificate::new([5u8; 32], 11, 3, 4, 4, 20, vec![[1u8; 32]]);

    let legitimate_only =
        ConstitutionalSeal::evaluate_stage(Some(&execution), Some(&legitimacy), None);
    let continuous_only =
        ConstitutionalSeal::evaluate_stage(Some(&execution), None, Some(&continuity));

    assert_eq!(
        legitimate_only.stage,
        ConstitutionalFinalityStage::LegitimatelyFinal
    );
    assert_eq!(
        continuous_only.stage,
        ConstitutionalFinalityStage::ContinuousFinal
    );
}

#[test]
fn compatibility_helpers_match_expected_context() {
    let qc = sample_qc([0xAA; 32], 22, 7);
    let execution = ExecutionCertificate::new(9, [0xBB; 32], qc);

    let legitimacy = LegitimacyCertificate::new(
        [0xAA; 32],
        9,
        [1u8; 32],
        [2u8; 32],
        [3u8; 32],
        vec![[4u8; 32]],
    );
    let continuity = ContinuityCertificate::new([0xAA; 32], 22, 7, 9, 8, 100, vec![[4u8; 32]]);

    assert!(legitimacy.is_compatible_with_execution(&execution));
    assert!(continuity.is_compatible_with_execution(&execution));
}
