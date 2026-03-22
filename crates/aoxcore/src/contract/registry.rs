use std::collections::BTreeMap;

use chrono::Utc;
use sha2::{Digest, Sha256};

use aoxcontract::{ContractDescriptor, ContractId, ContractManifest, ContractStatus};

use crate::contract::error::ContractRegistryError;
use crate::contract::receipt::{
    ContractActivated, ContractDeprecated, ContractReceipt, ContractRegistered, ContractRevoked,
};
use crate::contract::record::{ManifestDigest, OnChainContractRecord, RegisteredAtHeight};

/// In-memory registry for canonical contract records.
///
/// # Design intent
/// This type provides a deterministic and minimal registry abstraction for
/// contract descriptors that have already passed domain-level validation.
///
/// The registry is responsible for:
/// - preventing duplicate registration for the same canonical `ContractId`,
/// - tracking lifecycle state transitions for registered contracts,
/// - preserving the manifest associated with each registered contract, and
/// - recording registration metadata required by higher-layer consumers.
///
/// # Scope limitation
/// This implementation is intentionally in-memory and non-persistent.
/// It does not, by itself:
/// - provide storage durability,
/// - enforce governance approval semantics,
/// - validate transition authorization,
/// - coordinate with consensus, or
/// - execute contracts.
///
/// Those responsibilities are expected to be enforced by higher-level AOXChain
/// subsystems that orchestrate state transition validity and operator policy.
///
/// # Determinism note
/// A `BTreeMap` is used to preserve stable iteration order for callers that
/// enumerate records. This reduces incidental nondeterminism in tests and in
/// read-oriented tooling that depends on ordered traversal.
///
/// # Security note
/// This registry assumes that the supplied `ContractDescriptor` and
/// `ContractId` originate from trusted canonical construction paths. Callers
/// must not bypass upstream validation guarantees.
#[derive(Debug, Default)]
pub struct ContractRegistry {
    records: BTreeMap<ContractId, OnChainContractRecord>,
}

impl ContractRegistry {
    /// Registers a new canonical contract record.
    ///
    /// # Behavior
    /// - Rejects registration if the `ContractId` is already present.
    /// - Computes and stores a manifest digest derived from the manifest's
    ///   identity material.
    /// - Initializes lifecycle status as `Registered`.
    /// - Captures the provided registration height and the current update time.
    ///
    /// # Parameters
    /// - `descriptor`: Canonical contract descriptor containing the validated
    ///   manifest and derived contract identity.
    /// - `height`: Chain height at which the registration is being recorded.
    ///
    /// # Errors
    /// Returns `ContractRegistryError::AlreadyRegistered` if a record already
    /// exists for the supplied contract identifier.
    ///
    /// # Security considerations
    /// This method does not verify authorization or governance approval.
    /// It only enforces registry-level uniqueness and record construction.
    pub fn register_contract(
        &mut self,
        descriptor: ContractDescriptor,
        height: u64,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let contract_id = descriptor.contract_id.clone();

        if self.records.contains_key(&contract_id) {
            return Err(ContractRegistryError::AlreadyRegistered(contract_id.0));
        }

        let manifest_digest = manifest_digest_hex(&descriptor.manifest);

        let record = OnChainContractRecord {
            contract_id: contract_id.clone(),
            manifest: descriptor.manifest,
            status: ContractStatus::Registered,
            manifest_digest: ManifestDigest(manifest_digest),
            registered_at_height: RegisteredAtHeight(height),
            updated_at: Utc::now(),
        };

        self.records.insert(contract_id.clone(), record);

        Ok(ContractReceipt::Registered(ContractRegistered {
            contract_id,
        }))
    }

    /// Marks an existing contract as active.
    ///
    /// # Behavior
    /// Updates the lifecycle status to `Active` and refreshes the modification
    /// timestamp.
    ///
    /// # Errors
    /// Returns `ContractRegistryError::NotFound` if the target contract does
    /// not exist in the registry.
    ///
    /// # Security considerations
    /// This method does not validate whether activation is permissible under
    /// governance, review, rollout, or dependency policy. Such checks must be
    /// enforced before calling this method.
    pub fn activate_contract(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let record = self
            .records
            .get_mut(contract_id)
            .ok_or_else(|| ContractRegistryError::NotFound(contract_id.0.clone()))?;

        record.status = ContractStatus::Active;
        record.updated_at = Utc::now();

        Ok(ContractReceipt::Activated(ContractActivated {
            contract_id: contract_id.clone(),
        }))
    }

    /// Marks an existing contract as deprecated.
    ///
    /// # Behavior
    /// Updates the lifecycle status to `Deprecated` and refreshes the
    /// modification timestamp.
    ///
    /// # Errors
    /// Returns `ContractRegistryError::NotFound` if the target contract does
    /// not exist in the registry.
    ///
    /// # Security considerations
    /// Deprecation is treated here as a registry state mutation only. Any
    /// release-management, migration, sunset, or governance requirements must
    /// be validated externally.
    pub fn deprecate_contract(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let record = self
            .records
            .get_mut(contract_id)
            .ok_or_else(|| ContractRegistryError::NotFound(contract_id.0.clone()))?;

        record.status = ContractStatus::Deprecated;
        record.updated_at = Utc::now();

        Ok(ContractReceipt::Deprecated(ContractDeprecated {
            contract_id: contract_id.clone(),
        }))
    }

    /// Marks an existing contract as revoked.
    ///
    /// # Behavior
    /// Updates the lifecycle status to `Revoked` and refreshes the modification
    /// timestamp.
    ///
    /// # Errors
    /// Returns `ContractRegistryError::NotFound` if the target contract does
    /// not exist in the registry.
    ///
    /// # Security considerations
    /// Revocation is a high-impact lifecycle transition. This method does not
    /// determine whether revocation is justified or authorized; it only records
    /// the resulting state change.
    pub fn revoke_contract(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let record = self
            .records
            .get_mut(contract_id)
            .ok_or_else(|| ContractRegistryError::NotFound(contract_id.0.clone()))?;

        record.status = ContractStatus::Revoked;
        record.updated_at = Utc::now();

        Ok(ContractReceipt::Revoked(ContractRevoked {
            contract_id: contract_id.clone(),
        }))
    }

    /// Returns the canonical registry record for the supplied contract ID, if present.
    ///
    /// This is a read-only accessor and does not clone underlying state.
    pub fn get_contract(&self, contract_id: &ContractId) -> Option<&OnChainContractRecord> {
        self.records.get(contract_id)
    }

    /// Returns all registered contract records in stable key order.
    ///
    /// Because the underlying storage is a `BTreeMap`, iteration order follows
    /// the canonical ordering of `ContractId`.
    pub fn all_contracts(&self) -> Vec<&OnChainContractRecord> {
        self.records.values().collect()
    }
}

/// Computes a hexadecimal SHA-256 digest over the manifest identity material.
///
/// # Rationale
/// The registry stores a compact digest alongside the full manifest in order to
/// provide an immediately comparable integrity fingerprint for downstream
/// consumers, operators, and tooling.
///
/// # Fallback behavior
/// If identity material cannot be produced, this function hashes an empty byte
/// sequence via `unwrap_or_default()`. This preserves total function behavior
/// and avoids panics, but callers should treat such a case as an upstream data
/// quality concern rather than a successful integrity guarantee.
///
/// # Security note
/// This helper is not a substitute for canonical manifest validation. It
/// assumes that `identity_material()` reflects the intended canonical identity
/// projection defined by the domain crate.
fn manifest_digest_hex(manifest: &ContractManifest) -> String {
    let bytes = manifest.identity_material().unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
