use super::{Block, BlockError, BlockType};
use serde::{Deserialize, Serialize};

/// Stable event types emitted by block validation workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationEventType {
    Info,
    Warning,
    Error,
}

/// Human-readable descriptor for a block-domain error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDescriptor {
    pub code: &'static str,
    pub title: &'static str,
    pub plain_message: &'static str,
    pub probable_cause: &'static str,
    pub operator_action: &'static str,
}

/// Single serializable event entry inside a validation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEvent {
    pub event_type: ValidationEventType,
    pub code: String,
    pub title: String,
    pub message: String,
    pub action: String,
}

/// Serializable, operator-friendly validation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockValidationReport {
    pub accepted: bool,
    pub block_height: u64,
    pub block_type: BlockType,
    pub task_count: usize,
    pub total_payload_bytes: usize,
    pub primary_error_code: Option<String>,
    pub events: Vec<ValidationEvent>,
}

impl BlockValidationReport {
    /// Returns a stable pretty JSON document for desktop/CLI rendering.
    pub fn to_pretty_json(&self) -> Result<String, BlockError> {
        serde_json::to_string_pretty(self).map_err(|_| BlockError::SerializationFailed)
    }
}

/// Converts a block-domain error into a plain-language descriptor.
#[must_use]
pub fn describe_block_error(err: BlockError) -> ErrorDescriptor {
    match err {
        BlockError::InvalidSystemTime => ErrorDescriptor {
            code: err.code(),
            title: "Sistem saati geçersiz",
            plain_message: "Node saati doğru çalışmadığı için güvenli zaman damgası üretilemedi.",
            probable_cause: "Sunucu saati geri gitmiş olabilir veya NTP/saat senkronu bozuk olabilir.",
            operator_action: "NTP senkronunu düzeltin ve node saatini doğrulayıp işlemi tekrar deneyin.",
        },
        BlockError::ActiveBlockRequiresTasks => ErrorDescriptor {
            code: err.code(),
            title: "Aktif blok boş olamaz",
            plain_message: "Aktif blokta en az bir işlem/görev bulunmalıdır.",
            probable_cause: "Blok üretim hattı boş görev listesi ile çağrıldı.",
            operator_action: "Mempool ve blok üretim çağrısını kontrol edin; boş liste ile üretimi engelleyin.",
        },
        BlockError::HeartbeatBlockMustNotContainTasks => ErrorDescriptor {
            code: err.code(),
            title: "Heartbeat blok görev içeremez",
            plain_message: "Heartbeat blok sadece canlılık sinyali içindir, işlem taşıyamaz.",
            probable_cause: "Heartbeat türü yanlışlıkla normal işlem bloğu gibi üretildi.",
            operator_action: "Blok türü seçimini düzeltin; işlemler için Active blok kullanın.",
        },
        BlockError::EpochPruneBlockMustNotContainTasks => ErrorDescriptor {
            code: err.code(),
            title: "Epoch-prune blok görev içeremez",
            plain_message: "Bakım/temizlik bloğunda kullanıcı işlemi yer alamaz.",
            probable_cause: "Bakım bloğuna transaction eklenmiş.",
            operator_action: "Bakım akışını işlem akışından ayırın ve blok tipini doğrulayın.",
        },
        BlockError::HeartbeatBlockMustUseZeroStateRoot => ErrorDescriptor {
            code: err.code(),
            title: "Heartbeat state-root hatalı",
            plain_message: "Heartbeat blok için state_root sıfır kök olmalıdır.",
            probable_cause: "State root alanı yanlış dolduruldu.",
            operator_action: "Heartbeat üretiminde ZERO_STATE_ROOT sabitini zorunlu kullanın.",
        },
        BlockError::EmptyTaskPayload => ErrorDescriptor {
            code: err.code(),
            title: "Boş payload",
            plain_message: "Bir görevin veri alanı boş olamaz.",
            probable_cause: "İşlem verisi serialize edilmeden gönderilmiş olabilir.",
            operator_action: "İşlem oluşturma katmanında payload varlığını zorunlu kılın.",
        },
        BlockError::TaskPayloadTooLarge { .. } => ErrorDescriptor {
            code: err.code(),
            title: "Tek görev payload çok büyük",
            plain_message: "Tek bir görevin veri boyutu izin verilen sınırı geçti.",
            probable_cause: "Uygulama büyük veri blob'unu tek işlemde göndermiş.",
            operator_action: "Veriyi parçalara bölün veya zincir dışı depolama + referans yaklaşımı kullanın.",
        },
        BlockError::TooManyTasks { .. } => ErrorDescriptor {
            code: err.code(),
            title: "Blokta çok fazla görev var",
            plain_message: "Bloktaki görev sayısı güvenli işlem limitini aştı.",
            probable_cause: "Mempool paketleme limiti yanlış ayarlanmış olabilir.",
            operator_action: "Blok paketleme sırasında görev sayısı üst limitini uygulayın.",
        },
        BlockError::TotalPayloadTooLarge { .. } => ErrorDescriptor {
            code: err.code(),
            title: "Toplam blok payload sınırı aşıldı",
            plain_message: "Blok içindeki toplam veri boyutu güvenlik limitini geçti.",
            probable_cause: "Çok sayıda büyük payload aynı blokta birikti.",
            operator_action: "Paketleme politikasını sıkılaştırın ve payload toplamını blok öncesi hesaplayın.",
        },
        BlockError::LengthOverflow => ErrorDescriptor {
            code: err.code(),
            title: "Boyut hesaplaması taştı",
            plain_message: "Toplam boyut hesaplanırken sayı taşması tespit edildi.",
            probable_cause: "Beklenmeyen/bozuk veri büyüklüğü veya saldırı trafiği.",
            operator_action: "Girdi limitlerini daraltın, anomali kaydını inceleyin.",
        },
        BlockError::InvalidBlockHeight => ErrorDescriptor {
            code: err.code(),
            title: "Blok yüksekliği hatalı",
            plain_message: "Blok yüksekliği beklenen sıra ile uyumlu değil.",
            probable_cause: "Fork, replay veya yanlış parent bağlantısı olabilir.",
            operator_action: "Parent zinciri ve fork-choice kararını yeniden doğrulayın.",
        },
        BlockError::InvalidPreviousHash => ErrorDescriptor {
            code: err.code(),
            title: "Önceki hash uyuşmuyor",
            plain_message: "Blok, parent blok hash’i ile tutarlı değil.",
            probable_cause: "Yanlış parent seçimi veya veri bozulması.",
            operator_action: "Blok link doğrulamasını ve network kaynaklarını kontrol edin.",
        },
        BlockError::DuplicateTaskId => ErrorDescriptor {
            code: err.code(),
            title: "Tekrarlanan görev kimliği",
            plain_message: "Aynı task_id bir blokta birden fazla kez kullanılmış.",
            probable_cause: "Mempool deduplikasyon katmanı atlanmış olabilir.",
            operator_action: "Task ekleme sırasında task_id tekilliğini zorunlu denetleyin.",
        },
        BlockError::InvalidTimestamp => ErrorDescriptor {
            code: err.code(),
            title: "Geçersiz zaman damgası",
            plain_message: "Blok zaman damgası sıfır veya geçersiz bir değerde.",
            probable_cause: "Saat kaynağı bozuk veya üretim kodunda yanlış timestamp kullanılmış.",
            operator_action: "Timestamp üretimini merkezi yardımcı fonksiyona sabitleyin.",
        },
        BlockError::InvalidProducer => ErrorDescriptor {
            code: err.code(),
            title: "Geçersiz üretici kimliği",
            plain_message: "Blok üreticisi kimliği boş/geçersiz veya beklenen anahtarla uyuşmuyor.",
            probable_cause: "Anahtar yönetimi veya imzalama kimliği yanlış olabilir.",
            operator_action: "Validator anahtar setini ve role mapping yapılandırmasını doğrulayın.",
        },
        BlockError::InvalidStateRoot => ErrorDescriptor {
            code: err.code(),
            title: "Geçersiz state root",
            plain_message: "Blok state_root değeri beklenen taahhütle uyumlu değil.",
            probable_cause: "State hesaplama adımı hatalı veya veri bozulmuş.",
            operator_action: "State transition ve Merkle hesaplama zincirini tekrar çalıştırın.",
        },
        BlockError::InvalidTaskRoot => ErrorDescriptor {
            code: err.code(),
            title: "Geçersiz task root",
            plain_message: "Görev kök hash'i beklenen değerle uyuşmuyor.",
            probable_cause: "Task sırası/serializasyonu değişmiş olabilir.",
            operator_action: "Task canonical sıralama ve hash pipeline'ını doğrulayın.",
        },
        BlockError::HashingFailed => ErrorDescriptor {
            code: err.code(),
            title: "Hash hesaplama hatası",
            plain_message: "Kriptografik hash üretimi güvenli şekilde tamamlanamadı.",
            probable_cause: "Hash pipeline girişleri beklenen formatta değil.",
            operator_action: "Hash öncesi canonical encoding ve giriş boyutlarını denetleyin.",
        },
        BlockError::SerializationFailed => ErrorDescriptor {
            code: err.code(),
            title: "Serileştirme hatası",
            plain_message: "Rapor/çıktı güvenli biçimde serialize edilemedi.",
            probable_cause: "Beklenmeyen veri alanı veya format uyuşmazlığı.",
            operator_action: "Serileştirme şemasını ve alan uyumluluğunu kontrol edin.",
        },
    }
}

/// Produces a full, user-friendly report for block validation outcomes.
#[must_use]
pub fn build_block_validation_report(block: &Block) -> BlockValidationReport {
    let mut events = vec![ValidationEvent {
        event_type: ValidationEventType::Info,
        code: "BLOCK_VALIDATION_STARTED".to_string(),
        title: "Doğrulama başlatıldı".to_string(),
        message: "Blok protokol kurallarına göre doğrulanıyor.".to_string(),
        action: "İşlem tamamlanana kadar bekleyin.".to_string(),
    }];

    let result = block.validate();
    let mut primary_error_code = None;

    match result {
        Ok(()) => {
            events.push(ValidationEvent {
                event_type: ValidationEventType::Info,
                code: "BLOCK_VALIDATION_ACCEPTED".to_string(),
                title: "Blok kabul edildi".to_string(),
                message: "Blok doğrulama kontrollerinden başarıyla geçti.".to_string(),
                action: "Bloku zincire dahil etme aşamasına devam edebilirsiniz.".to_string(),
            });
        }
        Err(err) => {
            let desc = describe_block_error(err);
            primary_error_code = Some(desc.code.to_string());
            events.push(ValidationEvent {
                event_type: ValidationEventType::Error,
                code: desc.code.to_string(),
                title: desc.title.to_string(),
                message: format!(
                    "{} Olası neden: {}",
                    desc.plain_message, desc.probable_cause
                ),
                action: desc.operator_action.to_string(),
            });
        }
    }

    BlockValidationReport {
        accepted: primary_error_code.is_none(),
        block_height: block.header.height,
        block_type: block.header.block_type,
        task_count: block.task_count(),
        total_payload_bytes: block.total_payload_bytes(),
        primary_error_code,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Capability, TargetOutpost, Task, ZERO_HASH};

    fn bytes32(v: u8) -> [u8; 32] {
        [v; 32]
    }

    #[test]
    fn report_is_human_readable_for_invalid_block() {
        let block =
            Block::new_active_with_timestamp(7, 1, bytes32(1), bytes32(2), bytes32(3), vec![])
                .expect_err("active block without tasks should fail");

        let desc = describe_block_error(block);
        assert_eq!(desc.code, "BLOCK_ACTIVE_REQUIRES_TASKS");
        assert!(!desc.plain_message.is_empty());
        assert!(!desc.operator_action.is_empty());
    }

    #[test]
    fn validation_report_serializes_for_desktop_panels() {
        let task = Task::new(
            bytes32(9),
            Capability::UserSigned,
            TargetOutpost::AovmNative,
            vec![1, 2, 3],
        )
        .expect("task should build");

        let block =
            Block::new_active_with_timestamp(2, 100, ZERO_HASH, bytes32(8), bytes32(7), vec![task])
                .expect("block should build");

        let report = block.validate_with_report();
        assert!(report.accepted);

        let json = report
            .to_pretty_json()
            .expect("json serialization must succeed");
        assert!(json.contains("BLOCK_VALIDATION_ACCEPTED"));
        assert!(json.contains("\"accepted\": true"));
    }
}
