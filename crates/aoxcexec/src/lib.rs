use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Yürütme orkestrasyonu sırasında oluşabilecek hatalar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionError {
    InvalidPayload,
    GasDepleted,
    LaneUnavailable(String),
    StateTransitionFailed(String),
    ArithmeticOverflow,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayload => write!(f, "invalid transaction payload"),
            Self::GasDepleted => write!(f, "execution depleted available gas"),
            Self::LaneUnavailable(lane) => write!(f, "execution lane '{lane}' is unavailable"),
            Self::StateTransitionFailed(msg) => write!(f, "state transition failed: {msg}"),
            Self::ArithmeticOverflow => write!(f, "arithmetic overflow in gas calculation"),
        }
    }
}

impl Error for ExecutionError {}

/// Güvenli Gaz Birimi (aoxcvm ile uyumlu olması için u64)
pub type Gas = u64;

/// Orkestratöre sağlanan blok/yürütme bağlamı.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    pub block_height: u64,
    pub timestamp: u64,
    pub max_gas_per_block: Gas,
}

/// Yürütülecek soyut işlem paketi (Transaction Payload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    pub tx_hash: [u8; 32],
    pub lane_id: String,
    pub gas_limit: Gas,
    pub data: Vec<u8>,
}

/// Yürütme işleminin sonucu (Receipt).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub tx_hash: [u8; 32],
    pub success: bool,
    pub gas_used: Gas,
    pub error_message: Option<String>,
}

/// Ağa gelen işlemleri uygun Sanal Makine (VM) Lane'lerine dağıtan Çekirdek Arayüz.
pub trait ExecutionOrchestrator {
    fn execute_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<Vec<ExecutionReceipt>, ExecutionError>;
}

/// Gelecekteki `aoxcvm` entegrasyonuna kadar kullanılacak Yer Tutucu (Placeholder) Orkestratör.
#[derive(Debug, Default, Clone, Copy)]
pub struct PlaceholderOrchestrator;

impl PlaceholderOrchestrator {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ExecutionOrchestrator for PlaceholderOrchestrator {
    fn execute_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<Vec<ExecutionReceipt>, ExecutionError> {
        let mut receipts = Vec::with_capacity(payloads.len());
        let mut cumulative_gas: Gas = 0;

        for payload in payloads {
            // Güvenlik Kontrolü: İşlem paketi boş olamaz
            if payload.lane_id.is_empty() || payload.data.is_empty() {
                return Err(ExecutionError::InvalidPayload);
            }

            // Mock Gaz Maliyeti Hesaplaması ve Taşma Koruması
            let mock_gas_cost: Gas = 21_000;
            cumulative_gas = cumulative_gas
                .checked_add(mock_gas_cost)
                .ok_or(ExecutionError::ArithmeticOverflow)?;

            // Blok Gaz Limiti Kontrolü
            if cumulative_gas > context.max_gas_per_block {
                return Err(ExecutionError::GasDepleted);
            }

            // Makbuz Oluşturma (Başarılı Senaryo)
            receipts.push(ExecutionReceipt {
                tx_hash: payload.tx_hash,
                success: true,
                gas_used: mock_gas_cost,
                error_message: None,
            });
        }

        Ok(receipts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_payload(tx_hash: [u8; 32]) -> ExecutionPayload {
        ExecutionPayload {
            tx_hash,
            lane_id: "evm_lane".to_owned(),
            gas_limit: 50_000,
            data: vec![1, 2, 3, 4], // Mock veri
        }
    }

    #[test]
    fn test_successful_batch_execution() {
        let orchestrator = PlaceholderOrchestrator::new();
        let context = ExecutionContext {
            block_height: 1,
            timestamp: 1670000000,
            max_gas_per_block: 100_000,
        };

        let payloads = vec![create_mock_payload([1; 32]), create_mock_payload([2; 32])];

        let receipts = orchestrator
            .execute_batch(&context, &payloads)
            .expect("Execution should succeed");

        assert_eq!(receipts.len(), 2);
        assert_eq!(receipts[0].gas_used, 21_000);
        assert!(receipts[1].success);
    }

    #[test]
    fn test_gas_depletion_rejection() {
        let orchestrator = PlaceholderOrchestrator::new();
        // Sadece 1 işlem alabilecek kadar gaz limiti veriyoruz
        let context = ExecutionContext {
            block_height: 1,
            timestamp: 1670000000,
            max_gas_per_block: 30_000,
        };

        // 2 işlem yolluyoruz (21_000 * 2 = 42_000 Gaz gerekiyor)
        let payloads = vec![create_mock_payload([1; 32]), create_mock_payload([2; 32])];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(result.unwrap_err(), ExecutionError::GasDepleted);
    }

    #[test]
    fn test_invalid_payload_rejection() {
        let orchestrator = PlaceholderOrchestrator::new();
        let context = ExecutionContext {
            block_height: 1,
            timestamp: 1670000000,
            max_gas_per_block: 100_000,
        };

        // Boş datalı hatalı bir paket oluşturuyoruz
        let mut invalid_payload = create_mock_payload([1; 32]);
        invalid_payload.data.clear();

        let payloads = vec![invalid_payload];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(result.unwrap_err(), ExecutionError::InvalidPayload);
    }
}
