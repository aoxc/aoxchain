/// Uploaded WASM code artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmCode {
    pub code_id: [u8; 32],
    pub bytes: Vec<u8>,
}

impl WasmCode {
    /// Encodes the uploaded code artifact.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + 4 + self.bytes.len());
        out.extend_from_slice(&self.code_id);
        out.extend_from_slice(&(self.bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.bytes);
        out
    }

    /// Decodes the uploaded code artifact.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 36 {
            return None;
        }

        let mut code_id = [0u8; 32];
        code_id.copy_from_slice(&bytes[..32]);

        let len = u32::from_be_bytes(bytes[32..36].try_into().ok()?) as usize;
        if bytes.len() != 36 + len {
            return None;
        }

        Some(Self {
            code_id,
            bytes: bytes[36..].to_vec(),
        })
    }
}

/// Instantiated WASM contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmInstance {
    pub instance_id: [u8; 32],
    pub code_id: [u8; 32],
    pub state: Vec<u8>,
}

impl WasmInstance {
    /// Encodes the instance into a deterministic binary layout.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(64 + 4 + self.state.len());
        out.extend_from_slice(&self.instance_id);
        out.extend_from_slice(&self.code_id);
        out.extend_from_slice(&(self.state.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.state);
        out
    }

    /// Decodes the instance from binary.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 68 {
            return None;
        }

        let mut instance_id = [0u8; 32];
        instance_id.copy_from_slice(&bytes[..32]);

        let mut code_id = [0u8; 32];
        code_id.copy_from_slice(&bytes[32..64]);

        let len = u32::from_be_bytes(bytes[64..68].try_into().ok()?) as usize;
        if bytes.len() != 68 + len {
            return None;
        }

        Some(Self {
            instance_id,
            code_id,
            state: bytes[68..].to_vec(),
        })
    }
}
