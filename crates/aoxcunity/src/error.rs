use thiserror::Error;

/// Top-level consensus error surface.
///
/// This type provides a stable and explicit failure boundary for callers
/// interacting with block construction, vote admission, and consensus state.
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("block build error: {0}")]
    BlockBuild(#[from] crate::block::BlockBuildError),

    #[error("invalid genesis parent hash")]
    InvalidGenesisParent,

    #[error("unknown parent block")]
    UnknownParent,

    #[error("block height regression detected")]
    HeightRegression,

    #[error("child block height does not match parent height plus one")]
    InvalidParentHeight,

    #[error("duplicate block hash")]
    DuplicateBlock,

    #[error("validator set is empty")]
    EmptyValidatorSet,

    #[error("duplicate validator id in rotation")]
    DuplicateValidator,

    #[error("validator not found")]
    ValidatorNotFound,

    #[error("validator is inactive")]
    InactiveValidator,

    #[error("validator role is not eligible for voting")]
    NonVotingValidator,

    #[error("duplicate vote")]
    DuplicateVote,

    #[error("equivocating vote")]
    EquivocatingVote,

    #[error("vote target block does not exist")]
    VoteForUnknownBlock,

    #[error("vote is stale relative to finalized ancestry")]
    StaleVote,

    #[error("authenticated vote context does not match the active consensus context")]
    InvalidAuthenticatedContext,

    #[error("invalid quorum threshold")]
    InvalidQuorumThreshold,
}
