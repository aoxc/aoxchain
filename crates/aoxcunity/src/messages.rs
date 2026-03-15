use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::seal::BlockSeal;
use crate::vote::Vote;

/// Canonical consensus message surface.
///
/// Network transports remain free to wrap these messages as needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusMessage {
    BlockProposal { block: Block },
    Vote(Vote),
    Finalize { seal: BlockSeal },
}
