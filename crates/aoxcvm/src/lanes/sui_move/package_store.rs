// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// Published Move package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiPackage {
    pub package_id: [u8; 32],
    pub modules: Vec<u8>,
}

impl SuiPackage {
    /// Encodes the package into a deterministic binary layout.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + 4 + self.modules.len());
        out.extend_from_slice(&self.package_id);
        out.extend_from_slice(&(self.modules.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.modules);
        out
    }

    /// Decodes the package from binary.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 36 {
            return None;
        }

        let mut package_id = [0u8; 32];
        package_id.copy_from_slice(&bytes[..32]);

        let len = u32::from_be_bytes(bytes[32..36].try_into().ok()?) as usize;
        if bytes.len() != 36 + len {
            return None;
        }

        Some(Self {
            package_id,
            modules: bytes[36..].to_vec(),
        })
    }
}
