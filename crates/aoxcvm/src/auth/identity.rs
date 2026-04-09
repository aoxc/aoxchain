//! Identity and account object model for the quantum auth surface.

use crate::{
    auth::scheme::SignatureAlgorithm,
    crypto::hash::{QuantumHardenedDigest, quantum_hardened_digest},
};

/// Domain-separated account identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountId([u8; 32]);

impl AccountId {
    /// Returns raw bytes for storage and indexing.
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Account class. Governance and user accounts share the same abstract object model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// User/application account.
    User,
    /// Governance authority account.
    GovernanceAuthority,
    /// Validator account.
    Validator,
}

/// Nonce and replay strategy attached to an account object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonceReplayModel {
    /// Strictly increasing single global nonce.
    MonotonicGlobal,
    /// Strictly increasing nonce per auth domain lane.
    MonotonicPerDomain,
}

/// Canonical account object used by all auth principals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountObject {
    /// Stable derived account id.
    pub account_id: AccountId,
    /// Account class (user/governance/validator).
    pub kind: AccountKind,
    /// Active signature scheme for this account.
    pub scheme_id: SignatureAlgorithm,
    /// Commitment to signer material and validation policy config.
    pub key_commitment: Vec<u8>,
    /// Root of validation policy program/configuration.
    pub policy_root: QuantumHardenedDigest,
    /// Optional recovery policy root.
    pub recovery_root: QuantumHardenedDigest,
    /// Replay protection model.
    pub nonce_replay_model: NonceReplayModel,
}

impl AccountObject {
    /// Constructs an account object using canonical id derivation:
    /// H(domain || scheme_id || key_commitment || policy_root).
    pub fn new(
        account_domain: &str,
        kind: AccountKind,
        scheme_id: SignatureAlgorithm,
        key_commitment: Vec<u8>,
        policy_root: QuantumHardenedDigest,
        recovery_root: QuantumHardenedDigest,
        nonce_replay_model: NonceReplayModel,
    ) -> Self {
        let account_id = derive_account_id(
            account_domain,
            scheme_id,
            key_commitment.as_slice(),
            &policy_root.to_bytes(),
        );

        Self {
            account_id,
            kind,
            scheme_id,
            key_commitment,
            policy_root,
            recovery_root,
            nonce_replay_model,
        }
    }
}

/// Validator object under the same account abstraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorObject {
    /// Underlying account abstraction.
    pub account: AccountObject,
    /// Stake identity commitment (cold authority root).
    pub stake_identity_commitment: Vec<u8>,
    /// Online consensus key commitment.
    pub consensus_key_commitment: Vec<u8>,
}

/// Governance authority object under the same account abstraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceAuthorityObject {
    /// Underlying account abstraction.
    pub account: AccountObject,
    /// Governance policy root for constitutional authority powers.
    pub authority_policy_root: QuantumHardenedDigest,
}

/// Derives account id using the constitutional formula:
/// `AccountId = H(domain || scheme_id || key_commitment || policy_root)`.
pub fn derive_account_id(
    domain: &str,
    scheme_id: SignatureAlgorithm,
    key_commitment: &[u8],
    policy_root: &[u8],
) -> AccountId {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(domain.len() as u16).to_be_bytes());
    payload.extend_from_slice(domain.as_bytes());
    payload.extend_from_slice(&(scheme_id.wire_id().len() as u16).to_be_bytes());
    payload.extend_from_slice(scheme_id.wire_id().as_bytes());
    payload.extend_from_slice(&(key_commitment.len() as u32).to_be_bytes());
    payload.extend_from_slice(key_commitment);
    payload.extend_from_slice(&(policy_root.len() as u32).to_be_bytes());
    payload.extend_from_slice(policy_root);

    let digest = quantum_hardened_digest(b"auth/account-id/v1", payload.as_slice());
    AccountId(digest.blake3_256)
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{
            identity::{AccountKind, AccountObject, NonceReplayModel, derive_account_id},
            scheme::SignatureAlgorithm,
        },
        crypto::hash::quantum_hardened_digest,
    };

    #[test]
    fn account_id_is_deterministic() {
        let policy_root = quantum_hardened_digest(b"policy", b"v1");
        let a = derive_account_id(
            "AOX/ACCOUNT/V1",
            SignatureAlgorithm::MlDsa65,
            b"key-commitment-1",
            policy_root.to_bytes().as_slice(),
        );
        let b = derive_account_id(
            "AOX/ACCOUNT/V1",
            SignatureAlgorithm::MlDsa65,
            b"key-commitment-1",
            policy_root.to_bytes().as_slice(),
        );
        assert_eq!(a, b);
    }

    #[test]
    fn account_id_is_domain_separated() {
        let policy_root = quantum_hardened_digest(b"policy", b"v1");
        let user = derive_account_id(
            "AOX/ACCOUNT/V1",
            SignatureAlgorithm::MlDsa65,
            b"key-commitment-1",
            policy_root.to_bytes().as_slice(),
        );
        let governance = derive_account_id(
            "AOX/GOV_ACCOUNT/V1",
            SignatureAlgorithm::MlDsa65,
            b"key-commitment-1",
            policy_root.to_bytes().as_slice(),
        );
        assert_ne!(user, governance);
    }

    #[test]
    fn governance_and_user_share_same_account_abstraction() {
        let policy_root = quantum_hardened_digest(b"policy", b"v1");
        let recovery_root = quantum_hardened_digest(b"recovery", b"v1");

        let user = AccountObject::new(
            "AOX/ACCOUNT/V1",
            AccountKind::User,
            SignatureAlgorithm::MlDsa65,
            b"user-key".to_vec(),
            policy_root.clone(),
            recovery_root.clone(),
            NonceReplayModel::MonotonicPerDomain,
        );
        let governance = AccountObject::new(
            "AOX/ACCOUNT/V1",
            AccountKind::GovernanceAuthority,
            SignatureAlgorithm::MlDsa65,
            b"gov-key".to_vec(),
            policy_root,
            recovery_root,
            NonceReplayModel::MonotonicPerDomain,
        );

        assert_eq!(user.scheme_id, governance.scheme_id);
        assert_ne!(user.account_id, governance.account_id);
    }
}
