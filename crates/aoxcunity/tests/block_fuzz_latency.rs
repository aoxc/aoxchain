use aoxcunity::{
    BlockBody, BlockBuilder, BlockSection, ExternalNetwork, ExternalProofRecord,
    ExternalProofSection, ExternalProofType, LaneCommitment, LaneCommitmentSection, LaneType,
};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use std::time::{Duration, Instant};

const FUZZ_CASES: usize = 1_000;
const MAX_SINGLE_BLOCK_BUILD: Duration = Duration::from_millis(250);

#[test]
fn fuzz_block_build_is_deterministic_and_fast() {
    let mut rng = StdRng::seed_from_u64(0xA0C2_2026);
    let mut slowest_case = Duration::ZERO;

    for i in 0..FUZZ_CASES {
        let body = random_body(&mut rng);
        let parent_hash = random_hash(&mut rng);
        let proposer = non_zero_id(&mut rng);
        let network_id = (rng.random::<u64>() % 32_000) as u32 + 1;
        let height = i as u64 + 1;

        let started = Instant::now();
        let block_a = BlockBuilder::build(
            network_id,
            parent_hash,
            height,
            rng.random::<u64>() % 32,
            rng.random::<u64>() % 64,
            1_735_689_600 + i as u64,
            proposer,
            body.clone(),
        )
        .expect("block build should succeed for generated inputs");
        let elapsed = started.elapsed();

        let block_b = BlockBuilder::build(
            network_id,
            parent_hash,
            height,
            block_a.header.era,
            block_a.header.round,
            block_a.header.timestamp,
            proposer,
            body,
        )
        .expect("second build should succeed");

        assert_eq!(block_a, block_b, "non-deterministic output at case {i}");

        slowest_case = slowest_case.max(elapsed);
    }

    println!("fuzz run: slowest single block build = {:?}", slowest_case);

    assert!(
        slowest_case <= MAX_SINGLE_BLOCK_BUILD,
        "slowest case took {:?} (> {:?})",
        slowest_case,
        MAX_SINGLE_BLOCK_BUILD
    );
}

fn random_body(rng: &mut StdRng) -> BlockBody {
    let mut sections = Vec::new();

    let lane_count = ((rng.random::<u64>() % 8) + 1) as usize;
    let mut lanes = Vec::with_capacity(lane_count);
    for i in 0..lane_count {
        lanes.push(LaneCommitment {
            lane_id: i as u32,
            lane_type: random_lane_type(rng),
            tx_count: (rng.random::<u64>() % 2_000) as u32,
            input_root: random_hash(rng),
            output_root: random_hash(rng),
            receipt_root: random_hash(rng),
            state_commitment: random_hash(rng),
            proof_commitment: random_hash(rng),
        });
    }
    sections.push(BlockSection::LaneCommitment(LaneCommitmentSection {
        lanes,
    }));

    let proof_count = (rng.random::<u64>() % 6) as usize;
    if proof_count > 0 {
        let mut proofs = Vec::with_capacity(proof_count);
        for _ in 0..proof_count {
            proofs.push(ExternalProofRecord {
                source_network: random_external_network(rng),
                proof_type: random_external_proof_type(rng),
                subject_hash: random_hash(rng),
                proof_commitment: random_hash(rng),
                finalized_at: 1_700_000_000 + (rng.random::<u64>() % 1_000_000),
            });
        }
        sections.push(BlockSection::ExternalProof(ExternalProofSection { proofs }));
    }

    if rng.random::<u64>() % 2 == 0 {
        sections.reverse();
    }

    BlockBody { sections }
}

fn random_hash(rng: &mut StdRng) -> [u8; 32] {
    rng.random::<[u8; 32]>()
}

fn non_zero_id(rng: &mut StdRng) -> [u8; 32] {
    loop {
        let id = rng.random::<[u8; 32]>();
        if id != [0u8; 32] {
            return id;
        }
    }
}

fn random_lane_type(rng: &mut StdRng) -> LaneType {
    match rng.random::<u64>() % 7 {
        0 => LaneType::Native,
        1 => LaneType::Evm,
        2 => LaneType::SuiMove,
        3 => LaneType::CardanoUtxo,
        4 => LaneType::ZkEvm,
        5 => LaneType::Wasm,
        _ => LaneType::External,
    }
}

fn random_external_network(rng: &mut StdRng) -> ExternalNetwork {
    match rng.random::<u64>() % 6 {
        0 => ExternalNetwork::Ethereum,
        1 => ExternalNetwork::XLayer,
        2 => ExternalNetwork::Sui,
        3 => ExternalNetwork::Cardano,
        4 => ExternalNetwork::Bitcoin,
        _ => ExternalNetwork::Other((rng.random::<u64>() % 500) as u32),
    }
}

fn random_external_proof_type(rng: &mut StdRng) -> ExternalProofType {
    match rng.random::<u64>() % 5 {
        0 => ExternalProofType::Finality,
        1 => ExternalProofType::Inclusion,
        2 => ExternalProofType::Checkpoint,
        3 => ExternalProofType::StateCommitment,
        _ => ExternalProofType::Attestation,
    }
}
