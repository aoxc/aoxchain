//! Transaction kind definitions.

use serde::{Deserialize, Serialize};

/// High-level classification used by admission and policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TxKind {
    UserCall,
    PackagePublish,
    Governance,
    System,
}

impl TxKind {
    /// Returns true when the kind is expected to be submitted by end users.
    pub const fn is_user_visible(self) -> bool {
        matches!(self, Self::UserCall | Self::PackagePublish)
    }
}
