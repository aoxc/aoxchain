// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// Ownership class used by Sui-style objects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuiOwner {
    Address(Vec<u8>),
    Shared,
    Immutable,
}

/// Sui-style object persisted by the object store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiObject {
    pub object_id: [u8; 32],
    pub version: u64,
    pub owner: SuiOwner,
    pub type_tag: String,
    pub bcs_bytes: Vec<u8>,
}

impl SuiObject {
    /// Encodes the object into a deterministic binary layout.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.object_id);
        out.extend_from_slice(&self.version.to_be_bytes());

        match &self.owner {
            SuiOwner::Address(addr) => {
                out.push(0x00);
                out.extend_from_slice(&(addr.len() as u16).to_be_bytes());
                out.extend_from_slice(addr);
            }
            SuiOwner::Shared => {
                out.push(0x01);
            }
            SuiOwner::Immutable => {
                out.push(0x02);
            }
        }

        let type_bytes = self.type_tag.as_bytes();
        out.extend_from_slice(&(type_bytes.len() as u16).to_be_bytes());
        out.extend_from_slice(type_bytes);
        out.extend_from_slice(&(self.bcs_bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.bcs_bytes);
        out
    }

    /// Decodes an object from a deterministic binary layout.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 32 + 8 + 1 + 2 + 4 {
            return None;
        }

        let mut cursor = 0usize;

        let mut object_id = [0u8; 32];
        object_id.copy_from_slice(&bytes[cursor..cursor + 32]);
        cursor += 32;

        let version = u64::from_be_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
        cursor += 8;

        let owner_tag = *bytes.get(cursor)?;
        cursor += 1;

        let owner = match owner_tag {
            0x00 => {
                let len = u16::from_be_bytes(bytes[cursor..cursor + 2].try_into().ok()?) as usize;
                cursor += 2;
                let addr = bytes.get(cursor..cursor + len)?.to_vec();
                cursor += len;
                SuiOwner::Address(addr)
            }
            0x01 => SuiOwner::Shared,
            0x02 => SuiOwner::Immutable,
            _ => return None,
        };

        let type_len = u16::from_be_bytes(bytes[cursor..cursor + 2].try_into().ok()?) as usize;
        cursor += 2;
        let type_tag = String::from_utf8(bytes.get(cursor..cursor + type_len)?.to_vec()).ok()?;
        cursor += type_len;

        let body_len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().ok()?) as usize;
        cursor += 4;
        let bcs_bytes = bytes.get(cursor..cursor + body_len)?.to_vec();

        Some(Self {
            object_id,
            version,
            owner,
            type_tag,
            bcs_bytes,
        })
    }
}
