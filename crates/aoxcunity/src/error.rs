use thiserror::Error;

/// Top-level consensus error surface.
///
/// This type provides a stable and explicit failure boundary for callers
/// interacting with block construction, vote admission, and consensus state.
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("block build error: {0}")]
    BlockBuild(#[from] crate::block::BlockBuildError),

    #[error("unknown parent block")]
    UnknownParent,

    #[error("block height regression detected")]
    HeightRegression,

    #[error("validator set is empty")]
    EmptyValidatorSet,

    #[error("validator not found")]
    ValidatorNotFound,

    #[error("duplicate vote")]
    DuplicateVote,

    #[error("vote target block does not exist")]
    VoteForUnknownBlock,

    #[error("invalid quorum threshold")]
    InvalidQuorumThreshold,
}

