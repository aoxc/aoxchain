/// Canonical EVM contract account representation persisted by the lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmContractAccount {
    pub address: [u8; 20],
    pub code: Vec<u8>,
}

impl EvmContractAccount {
    /// Encodes the account into a stable binary layout.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(20 + 4 + self.code.len());
        out.extend_from_slice(&self.address);
        out.extend_from_slice(&(self.code.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.code);
        out
    }

    /// Decodes a contract account from binary.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 24 {
            return None;
        }

        let mut address = [0u8; 20];
        address.copy_from_slice(&bytes[..20]);

        let len = u32::from_be_bytes(bytes[20..24].try_into().ok()?) as usize;
        if bytes.len() != 24 + len {
            return None;
        }

        let code = bytes[24..].to_vec();
        Some(Self { address, code })
    }
}
