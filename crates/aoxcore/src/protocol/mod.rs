use blake3::Hasher;
use serde::{Deserialize, Serialize};

const PROTOCOL_MESSAGE_NAMESPACE: &[u8] = b"AOXC/PROTOCOL/MESSAGE_ENVELOPE";
const MESSAGE_ENVELOPE_HASH_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModuleId {
    RelayCore,
    Identity,
    Asset,
    Execution,
    Interop,
    Proof,
}

impl ModuleId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RelayCore => "relay_core",
            Self::Identity => "identity",
            Self::Asset => "asset",
            Self::Execution => "execution",
            Self::Interop => "interop",
            Self::Proof => "proof",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SovereignRoot {
    Identity,
    Supply,
    Governance,
    Relay,
    Security,
    Settlement,
    Treasury,
}

impl SovereignRoot {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Supply => "supply",
            Self::Governance => "governance",
            Self::Relay => "relay",
            Self::Security => "security",
            Self::Settlement => "settlement",
            Self::Treasury => "treasury",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChainFamily {
    Relay,
    Evm,
    Solana,
    Utxo,
    Ibc,
    Object,
    Wasm,
}

impl ChainFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Relay => "relay",
            Self::Evm => "evm",
            Self::Solana => "solana",
            Self::Utxo => "utxo",
            Self::Ibc => "ibc",
            Self::Object => "object",
            Self::Wasm => "wasm",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FeeClass {
    System,
    Standard,
    Priority,
}

impl FeeClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Standard => "standard",
            Self::Priority => "priority",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageEnvelope {
    pub source_module: ModuleId,
    pub destination_module: ModuleId,
    pub source_chain_family: ChainFamily,
    pub target_chain_family: ChainFamily,
    pub nonce: u64,
    pub payload_type: String,
    pub payload_hash: [u8; 32],
    pub proof_reference: Option<[u8; 32]>,
    pub fee_class: FeeClass,
    pub expiry: Option<u64>,
    pub replay_protection_tag: [u8; 16],
}

impl MessageEnvelope {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.source_module == self.destination_module {
            return Err("source_module and destination_module must differ");
        }

        if self.payload_type.trim().is_empty() {
            return Err("payload_type must not be empty");
        }

        if self.replay_protection_tag == [0u8; 16] {
            return Err("replay_protection_tag must not be zero");
        }

        Ok(())
    }

    #[must_use]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(PROTOCOL_MESSAGE_NAMESPACE);
        hasher.update(&[0x00, MESSAGE_ENVELOPE_HASH_VERSION]);
        hasher.update(self.source_module.as_str().as_bytes());
        hasher.update(&[0x00]);
        hasher.update(self.destination_module.as_str().as_bytes());
        hasher.update(&[0x00]);
        hasher.update(self.source_chain_family.as_str().as_bytes());
        hasher.update(&[0x00]);
        hasher.update(self.target_chain_family.as_str().as_bytes());
        hasher.update(&[0x00]);
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&[0x00]);
        hasher.update(self.payload_type.as_bytes());
        hasher.update(&[0x00]);
        hasher.update(&self.payload_hash);
        hasher.update(&[0x00]);
        hasher.update(&self.proof_reference.unwrap_or([0u8; 32]));
        hasher.update(&[0x00]);
        hasher.update(self.fee_class.as_str().as_bytes());
        hasher.update(&[0x00]);
        hasher.update(&self.expiry.unwrap_or_default().to_le_bytes());
        hasher.update(&[0x00]);
        hasher.update(&self.replay_protection_tag);
        *hasher.finalize().as_bytes()
    }
}

#[must_use]
pub const fn canonical_modules() -> [ModuleId; 6] {
    [
        ModuleId::RelayCore,
        ModuleId::Identity,
        ModuleId::Asset,
        ModuleId::Execution,
        ModuleId::Interop,
        ModuleId::Proof,
    ]
}

#[must_use]
pub const fn canonical_chain_families() -> [ChainFamily; 5] {
    [
        ChainFamily::Evm,
        ChainFamily::Solana,
        ChainFamily::Utxo,
        ChainFamily::Ibc,
        ChainFamily::Object,
    ]
}

#[must_use]
pub const fn canonical_message_envelope_fields() -> [&'static str; 11] {
    [
        "sourceModule",
        "destinationModule",
        "sourceChainFamily",
        "targetChainFamily",
        "nonce",
        "payloadType",
        "payloadHash",
        "proofReference",
        "feeClass",
        "expiry",
        "replayProtectionTag",
    ]
}

#[must_use]
pub const fn canonical_sovereign_roots() -> [SovereignRoot; 7] {
    [
        SovereignRoot::Identity,
        SovereignRoot::Supply,
        SovereignRoot::Governance,
        SovereignRoot::Relay,
        SovereignRoot::Security,
        SovereignRoot::Settlement,
        SovereignRoot::Treasury,
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        ChainFamily, FeeClass, MessageEnvelope, ModuleId, SovereignRoot, canonical_chain_families,
        canonical_message_envelope_fields, canonical_modules, canonical_sovereign_roots,
    };

    fn sample_envelope() -> MessageEnvelope {
        MessageEnvelope {
            source_module: ModuleId::Interop,
            destination_module: ModuleId::Proof,
            source_chain_family: ChainFamily::Evm,
            target_chain_family: ChainFamily::Relay,
            nonce: 7,
            payload_type: "bridge.commitment".to_string(),
            payload_hash: [0x11; 32],
            proof_reference: Some([0x22; 32]),
            fee_class: FeeClass::Priority,
            expiry: Some(42),
            replay_protection_tag: [0x33; 16],
        }
    }

    #[test]
    fn canonical_module_list_matches_six_domain_model() {
        assert_eq!(canonical_modules().len(), 6);
        assert_eq!(canonical_modules()[0], ModuleId::RelayCore);
        assert_eq!(canonical_modules()[5], ModuleId::Proof);
    }

    #[test]
    fn canonical_chain_family_list_matches_five_target_model() {
        let families = canonical_chain_families();
        assert_eq!(families.len(), 5);
        assert!(families.contains(&ChainFamily::Evm));
        assert!(families.contains(&ChainFamily::Solana));
        assert!(families.contains(&ChainFamily::Utxo));
        assert!(families.contains(&ChainFamily::Ibc));
        assert!(families.contains(&ChainFamily::Object));
    }

    #[test]
    fn message_envelope_validation_rejects_invalid_values() {
        let mut envelope = sample_envelope();
        envelope.destination_module = ModuleId::Interop;
        assert!(envelope.validate().is_err());

        let mut envelope = sample_envelope();
        envelope.payload_type.clear();
        assert!(envelope.validate().is_err());

        let mut envelope = sample_envelope();
        envelope.replay_protection_tag = [0u8; 16];
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn message_envelope_hash_is_deterministic_and_sensitive() {
        let envelope = sample_envelope();
        let mut changed = sample_envelope();
        changed.nonce += 1;

        assert_eq!(envelope.hash(), sample_envelope().hash());
        assert_ne!(envelope.hash(), changed.hash());
    }

    #[test]
    fn canonical_message_field_count_remains_stable() {
        let fields = canonical_message_envelope_fields();
        assert_eq!(fields.len(), 11);
        assert_eq!(fields[0], "sourceModule");
        assert_eq!(fields[10], "replayProtectionTag");
    }

    #[test]
    fn canonical_sovereign_roots_match_local_constitutional_model() {
        let roots = canonical_sovereign_roots();
        assert_eq!(roots.len(), 7);
        assert_eq!(roots[0], SovereignRoot::Identity);
        assert_eq!(roots[6], SovereignRoot::Treasury);
    }
}
