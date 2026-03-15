/// Minimal UTxO representation persisted by the Cardano-style lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    pub utxo_id: [u8; 32],
    pub owner: Vec<u8>,
    pub value: Vec<u8>,
    pub datum: Option<Vec<u8>>,
}

impl Utxo {
    /// Encodes the UTxO into a deterministic binary layout.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.utxo_id);
        out.extend_from_slice(&(self.owner.len() as u16).to_be_bytes());
        out.extend_from_slice(&self.owner);
        out.extend_from_slice(&(self.value.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.value);

        match &self.datum {
            Some(datum) => {
                out.push(0x01);
                out.extend_from_slice(&(datum.len() as u32).to_be_bytes());
                out.extend_from_slice(datum);
            }
            None => out.push(0x00),
        }

        out
    }

    /// Decodes a UTxO from binary.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 32 + 2 + 4 + 1 {
            return None;
        }

        let mut cursor = 0usize;
        let mut utxo_id = [0u8; 32];
        utxo_id.copy_from_slice(&bytes[cursor..cursor + 32]);
        cursor += 32;

        let owner_len = u16::from_be_bytes(bytes[cursor..cursor + 2].try_into().ok()?) as usize;
        cursor += 2;
        let owner = bytes.get(cursor..cursor + owner_len)?.to_vec();
        cursor += owner_len;

        let value_len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().ok()?) as usize;
        cursor += 4;
        let value = bytes.get(cursor..cursor + value_len)?.to_vec();
        cursor += value_len;

        let datum_flag = *bytes.get(cursor)?;
        cursor += 1;

        let datum = match datum_flag {
            0x00 => None,
            0x01 => {
                let len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().ok()?) as usize;
                cursor += 4;
                Some(bytes.get(cursor..cursor + len)?.to_vec())
            }
            _ => return None,
        };

        Some(Self {
            utxo_id,
            owner,
            value,
            datum,
        })
    }
}
