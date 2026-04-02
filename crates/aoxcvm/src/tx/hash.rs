//! Deterministic digest helper for transaction envelope hashing.

use crate::tx::{envelope::TxEnvelope, kind::TxKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TxDigest(pub [u8; 32]);

impl TxDigest {
    pub fn from_envelope(tx: &TxEnvelope) -> Self {
        let mut state = [0u64; 4];
        state[0] = 0xcbf2_9ce4_8422_2325;
        state[1] = 0x9e37_79b1_85eb_ca87;
        state[2] = 0x243f_6a88_85a3_08d3;
        state[3] = 0x1319_8a2e_0370_7344;

        mix_u64(&mut state, tx.chain_id);
        mix_u64(&mut state, tx.nonce);
        mix_u64(&mut state, kind_tag(tx.kind));
        mix_u64(&mut state, tx.fee_budget.gas_limit);
        mix_u64(&mut state, tx.fee_budget.gas_price);
        for b in tx.payload.as_bytes() {
            mix_u8(&mut state, *b);
        }

        let mut out = [0u8; 32];
        for (i, lane) in state.iter().enumerate() {
            out[i * 8..(i + 1) * 8].copy_from_slice(&lane.to_le_bytes());
        }
        Self(out)
    }
}

fn kind_tag(kind: TxKind) -> u64 {
    match kind {
        TxKind::UserCall => 1,
        TxKind::PackagePublish => 2,
        TxKind::Governance => 3,
        TxKind::System => 4,
    }
}

fn mix_u8(state: &mut [u64; 4], byte: u8) {
    for (i, lane) in state.iter_mut().enumerate() {
        *lane ^= (byte as u64) + ((i as u64) << 8);
        *lane = lane.wrapping_mul(0x1000_0000_01b3);
        *lane = lane.rotate_left(5 + (i as u32));
    }
}

fn mix_u64(state: &mut [u64; 4], value: u64) {
    for b in value.to_le_bytes() {
        mix_u8(state, b);
    }
}

#[cfg(test)]
mod tests {
    use crate::tx::{envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload};

    use super::TxDigest;

    #[test]
    fn digest_is_stable_and_sensitive_to_nonce() {
        let a = TxEnvelope::new(
            7,
            1,
            TxKind::UserCall,
            FeeBudget::new(1_000, 2),
            TxPayload::new(vec![1, 2, 3]),
        );
        let b = TxEnvelope::new(
            7,
            2,
            TxKind::UserCall,
            FeeBudget::new(1_000, 2),
            TxPayload::new(vec![1, 2, 3]),
        );

        assert_ne!(TxDigest::from_envelope(&a), TxDigest::from_envelope(&b));
    }
}
