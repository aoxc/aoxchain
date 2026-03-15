use crate::block::hash::{canonical_section_sort_key, compute_block_hash, compute_body_roots};
use crate::block::types::{
    Block,
    BlockBody,
    BlockBuildError,
    BlockHeader,
    BLOCK_VERSION_V1,
};

/// Deterministic block construction utility.
///
/// Design guarantees:
/// - block construction is fully deterministic for identical inputs,
/// - section ordering is canonicalized before root calculation,
/// - header hashing remains stable and compact,
/// - block construction does not depend on wall-clock side effects.
pub struct BlockBuilder;

impl BlockBuilder {
    /// Builds a canonical coordination block.
    ///
    /// The caller remains responsible for:
    /// - proposer selection,
    /// - timestamp policy enforcement,
    /// - parent validity,
    /// - lane-level semantic validation,
    /// - external proof verification.
    pub fn build(
        network_id: u32,
        parent_hash: [u8; 32],
        height: u64,
        era: u64,
        round: u64,
        timestamp: u64,
        proposer: [u8; 32],
        mut body: BlockBody,
    ) -> Result<Block, BlockBuildError> {
        if timestamp == 0 {
            return Err(BlockBuildError::ZeroTimestamp);
        }

        if proposer == [0u8; 32] {
            return Err(BlockBuildError::ZeroProposer);
        }

        let _ = u64::try_from(body.sections.len())
            .map_err(|_| BlockBuildError::SectionCountOverflow)?;

        body.sections
            .sort_by_key(canonical_section_sort_key);

        let roots = compute_body_roots(&body);

        let header = BlockHeader {
            version: BLOCK_VERSION_V1,
            network_id,
            parent_hash,
            height,
            era,
            round,
            timestamp,
            proposer,
            body_root: roots.body_root,
            authority_root: roots.authority_root,
            lane_root: roots.lane_root,
            proof_root: roots.proof_root,
        };

        let hash = compute_block_hash(&header);

        Ok(Block { hash, header, body })
    }
}

#[cfg(test)]
mod tests {
    use crate::block::types::{
        BlockBody,
        BlockSection,
        ExternalNetwork,
        ExternalProofRecord,
        ExternalProofSection,
        ExternalProofType,
        LaneCommitment,
        LaneCommitmentSection,
        LaneType,
    };

    use super::BlockBuilder;

    #[test]
    fn builds_deterministic_block_for_identical_inputs() {
        let body = BlockBody {
            sections: vec![BlockSection::LaneCommitment(LaneCommitmentSection {
                lanes: vec![LaneCommitment {
                    lane_id: 1,
                    lane_type: LaneType::Native,
                    tx_count: 2,
                    input_root: [1u8; 32],
                    output_root: [2u8; 32],
                    receipt_root: [3u8; 32],
                    state_commitment: [4u8; 32],
                    proof_commitment: [5u8; 32],
                }],
            })],
        };

        let block_a = BlockBuilder::build(
            1,
            [9u8; 32],
            10,
            0,
            7,
            1_735_689_600,
            [8u8; 32],
            body.clone(),
        )
        .unwrap();

        let block_b = BlockBuilder::build(
            1,
            [9u8; 32],
            10,
            0,
            7,
            1_735_689_600,
            [8u8; 32],
            body,
        )
        .unwrap();

        assert_eq!(block_a, block_b);
    }

    #[test]
    fn canonicalizes_section_order() {
        let lane_section = BlockSection::LaneCommitment(LaneCommitmentSection {
            lanes: vec![LaneCommitment {
                lane_id: 7,
                lane_type: LaneType::Evm,
                tx_count: 3,
                input_root: [1u8; 32],
                output_root: [2u8; 32],
                receipt_root: [3u8; 32],
                state_commitment: [4u8; 32],
                proof_commitment: [5u8; 32],
            }],
        });

        let proof_section = BlockSection::ExternalProof(ExternalProofSection {
            proofs: vec![ExternalProofRecord {
                source_network: ExternalNetwork::Sui,
                proof_type: ExternalProofType::Finality,
                subject_hash: [6u8; 32],
                proof_commitment: [7u8; 32],
                finalized_at: 1_735_689_600,
            }],
        });

        let block_a = BlockBuilder::build(
            1,
            [1u8; 32],
            1,
            0,
            1,
            1_735_689_600,
            [2u8; 32],
            BlockBody {
                sections: vec![lane_section.clone(), proof_section.clone()],
            },
        )
        .unwrap();

        let block_b = BlockBuilder::build(
            1,
            [1u8; 32],
            1,
            0,
            1,
            1_735_689_600,
            [2u8; 32],
            BlockBody {
                sections: vec![proof_section, lane_section],
            },
        )
        .unwrap();

        assert_eq!(block_a.hash, block_b.hash);
        assert_eq!(block_a.header.body_root, block_b.header.body_root);
    }
}

