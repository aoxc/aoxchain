use super::{Block, BlockError, BlockType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Supported locales for operator-facing report text.
///
/// English is the global default. Additional languages can be added
/// by extending `localized` call sites in this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportLocale {
    En,
    Tr,
}

impl Default for ReportLocale {
    fn default() -> Self {
        Self::En
    }
}

impl ReportLocale {
    /// Parses a locale string (e.g. `en`, `en-US`, `tr`, `tr-TR`).
    #[must_use]
    pub fn parse(input: &str) -> Self {
        let normalized = input.trim().to_ascii_lowercase();
        if normalized.starts_with("tr") {
            Self::Tr
        } else {
            Self::En
        }
    }
}

fn localized(locale: ReportLocale, en: &'static str, tr: &'static str) -> String {
    match locale {
        ReportLocale::En => en.to_string(),
        ReportLocale::Tr => tr.to_string(),
    }
}

/// Stable event types emitted by block validation workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationEventType {
    Info,
    Warning,
    Error,
}

/// Human-readable descriptor for a block-domain error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDescriptor {
    pub code: String,
    pub global_code: String,
    pub title: String,
    pub plain_message: String,
    pub probable_cause: String,
    pub operator_action: String,
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
    pub locale: ReportLocale,
    pub accepted: bool,
    pub block_height: u64,
    pub block_type: BlockType,
    pub task_count: usize,
    pub total_payload_bytes: usize,
    pub primary_error_code: Option<String>,
    pub events: Vec<ValidationEvent>,
}

/// Global AOXC block-domain error codes for operator dashboards and support workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalErrorCode {
    AoxcBlk0001,
    AoxcBlk0002,
    AoxcBlk0003,
    AoxcBlk0004,
    AoxcBlk0005,
    AoxcBlk0006,
    AoxcBlk0007,
    AoxcBlk0008,
    AoxcBlk0009,
    AoxcBlk0010,
    AoxcBlk0011,
    AoxcBlk0012,
    AoxcBlk0013,
    AoxcBlk0014,
    AoxcBlk0015,
    AoxcBlk0016,
    AoxcBlk0017,
    AoxcBlk0018,
    AoxcBlk0019,
    AoxcBlk0020,
}

impl GlobalErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AoxcBlk0001 => "AOXC-BLK-0001",
            Self::AoxcBlk0002 => "AOXC-BLK-0002",
            Self::AoxcBlk0003 => "AOXC-BLK-0003",
            Self::AoxcBlk0004 => "AOXC-BLK-0004",
            Self::AoxcBlk0005 => "AOXC-BLK-0005",
            Self::AoxcBlk0006 => "AOXC-BLK-0006",
            Self::AoxcBlk0007 => "AOXC-BLK-0007",
            Self::AoxcBlk0008 => "AOXC-BLK-0008",
            Self::AoxcBlk0009 => "AOXC-BLK-0009",
            Self::AoxcBlk0010 => "AOXC-BLK-0010",
            Self::AoxcBlk0011 => "AOXC-BLK-0011",
            Self::AoxcBlk0012 => "AOXC-BLK-0012",
            Self::AoxcBlk0013 => "AOXC-BLK-0013",
            Self::AoxcBlk0014 => "AOXC-BLK-0014",
            Self::AoxcBlk0015 => "AOXC-BLK-0015",
            Self::AoxcBlk0016 => "AOXC-BLK-0016",
            Self::AoxcBlk0017 => "AOXC-BLK-0017",
            Self::AoxcBlk0018 => "AOXC-BLK-0018",
            Self::AoxcBlk0019 => "AOXC-BLK-0019",
            Self::AoxcBlk0020 => "AOXC-BLK-0020",
        }
    }
}

/// Canonical proof/evidence metadata for a validation run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEvidence {
    pub schema_version: String,
    pub ruleset_version: String,
    pub header_hash_hex: String,
    pub report_hash_hex: String,
}

/// Full envelope: machine report + proof metadata + human-friendly text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEnvelope {
    pub report: BlockValidationReport,
    pub evidence: ValidationEvidence,
    pub cli_summary: String,
}

/// Extension point for kernel-level language customization without
/// modifying core reporting logic.
pub trait ReportLanguagePack {
    fn validation_started_title(&self, locale: ReportLocale) -> String;
    fn validation_started_message(&self, locale: ReportLocale) -> String;
    fn wait_action(&self, locale: ReportLocale) -> String;
    fn accepted_title(&self, locale: ReportLocale) -> String;
    fn accepted_message(&self, locale: ReportLocale) -> String;
    fn accepted_action(&self, locale: ReportLocale) -> String;
    fn probable_cause_label(&self, locale: ReportLocale) -> String;
    fn describe_error(&self, err: BlockError, locale: ReportLocale) -> ErrorDescriptor;
}

/// Built-in language pack. English is default; Turkish is bundled.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultReportLanguagePack;

impl BlockValidationReport {
    /// Returns a stable pretty JSON document for desktop/CLI rendering.
    pub fn to_pretty_json(&self) -> Result<String, BlockError> {
        serde_json::to_string_pretty(self).map_err(|_| BlockError::SerializationFailed)
    }

    /// Human-first text output for non-technical CLI and desktop logs.
    #[must_use]
    pub fn to_human_text(&self) -> String {
        let status = if self.accepted {
            "ACCEPTED"
        } else {
            "REJECTED"
        };
        let code = self.primary_error_code.as_deref().unwrap_or("NONE");

        let mut out = format!(
            "Status: {status}\nBlock: #{}\nType: {:?}\nTasks: {}\nTotal Payload: {} bytes\nPrimary Error: {code}\n",
            self.block_height, self.block_type, self.task_count, self.total_payload_bytes
        );

        for event in &self.events {
            out.push_str(&format!(
                "\n[{:?}] {} - {}\n{}\nAction: {}\n",
                event.event_type, event.code, event.title, event.message, event.action
            ));
        }

        out
    }
}

/// Maps internal `BlockError` values to globally stable AOXC error codes.
#[must_use]
pub const fn global_error_code(err: BlockError) -> GlobalErrorCode {
    match err {
        BlockError::InvalidSystemTime => GlobalErrorCode::AoxcBlk0001,
        BlockError::ActiveBlockRequiresTasks => GlobalErrorCode::AoxcBlk0002,
        BlockError::HeartbeatBlockMustNotContainTasks => GlobalErrorCode::AoxcBlk0003,
        BlockError::EpochPruneBlockMustNotContainTasks => GlobalErrorCode::AoxcBlk0004,
        BlockError::HeartbeatBlockMustUseZeroStateRoot => GlobalErrorCode::AoxcBlk0005,
        BlockError::EmptyTaskPayload => GlobalErrorCode::AoxcBlk0006,
        BlockError::TaskPayloadTooLarge { .. } => GlobalErrorCode::AoxcBlk0007,
        BlockError::TooManyTasks { .. } => GlobalErrorCode::AoxcBlk0008,
        BlockError::TotalPayloadTooLarge { .. } => GlobalErrorCode::AoxcBlk0009,
        BlockError::LengthOverflow => GlobalErrorCode::AoxcBlk0010,
        BlockError::InvalidBlockHeight => GlobalErrorCode::AoxcBlk0011,
        BlockError::InvalidPreviousHash => GlobalErrorCode::AoxcBlk0012,
        BlockError::DuplicateTaskId => GlobalErrorCode::AoxcBlk0013,
        BlockError::InvalidTimestamp => GlobalErrorCode::AoxcBlk0014,
        BlockError::InvalidProducer => GlobalErrorCode::AoxcBlk0015,
        BlockError::InvalidStateRoot => GlobalErrorCode::AoxcBlk0016,
        BlockError::InvalidTaskRoot => GlobalErrorCode::AoxcBlk0017,
        BlockError::HashingFailed => GlobalErrorCode::AoxcBlk0019,
        BlockError::SerializationFailed => GlobalErrorCode::AoxcBlk0020,
    }
}

/// Converts a block-domain error into a plain-language descriptor.
#[must_use]
pub fn describe_block_error(err: BlockError) -> ErrorDescriptor {
    describe_block_error_with_locale(err, ReportLocale::En)
}

/// Localized variant for operator messages.
#[must_use]
pub fn describe_block_error_with_locale(err: BlockError, locale: ReportLocale) -> ErrorDescriptor {
    let global = global_error_code(err).as_str().to_string();
    let (title_en, title_tr, msg_en, msg_tr, cause_en, cause_tr, action_en, action_tr) = match err {
        BlockError::InvalidSystemTime => (
            "Invalid system clock",
            "Sistem saati geçersiz",
            "The node could not produce a safe timestamp.",
            "Node güvenli bir zaman damgası üretemedi.",
            "Clock drift or NTP desynchronization is likely.",
            "Saat kayması veya NTP senkron bozulması olası.",
            "Fix host time sync and retry block production.",
            "Host saat senkronunu düzeltip üretimi tekrar deneyin.",
        ),
        BlockError::ActiveBlockRequiresTasks => (
            "Active block cannot be empty",
            "Aktif blok boş olamaz",
            "An active block must contain at least one task.",
            "Aktif blok en az bir görev içermeli.",
            "Block builder submitted an empty task list.",
            "Blok üretici boş görev listesi gönderdi.",
            "Enforce non-empty mempool selection before assembly.",
            "Assembly öncesi mempool seçiminde boşluğu engelleyin.",
        ),
        BlockError::HeartbeatBlockMustNotContainTasks => (
            "Heartbeat block carries tasks",
            "Heartbeat blok görev taşıyor",
            "Heartbeat blocks are liveness-only and cannot include tasks.",
            "Heartbeat bloklar sadece canlılık içindir, görev taşıyamaz.",
            "Wrong block type selected during assembly.",
            "Assembly sırasında yanlış blok tipi seçilmiş.",
            "Use Active block type for task-carrying blocks.",
            "Görev taşıyan bloklar için Active tipi kullanın.",
        ),
        BlockError::EpochPruneBlockMustNotContainTasks => (
            "Epoch-prune block carries tasks",
            "Epoch-prune blok görev taşıyor",
            "Maintenance blocks must not include user tasks.",
            "Bakım blokları kullanıcı görevi içeremez.",
            "Maintenance and execution flows were mixed.",
            "Bakım ve yürütme akışları karışmış.",
            "Separate maintenance block assembly from transaction packing.",
            "Bakım block assembly ile transaction paketlemeyi ayırın.",
        ),
        BlockError::HeartbeatBlockMustUseZeroStateRoot => (
            "Heartbeat state root mismatch",
            "Heartbeat state root uyumsuz",
            "Heartbeat blocks must use the zero state root.",
            "Heartbeat bloklar sıfır state root kullanmalıdır.",
            "Non-zero state root was injected into heartbeat path.",
            "Heartbeat akışına sıfır olmayan state root verilmiş.",
            "Force ZERO_STATE_ROOT in heartbeat constructor.",
            "Heartbeat constructor'da ZERO_STATE_ROOT zorunlu olsun.",
        ),
        BlockError::EmptyTaskPayload => (
            "Task payload is empty",
            "Görev payload boş",
            "Task payload must contain data.",
            "Görev payload veri içermelidir.",
            "Task creation omitted payload bytes.",
            "Görev oluşturma adımı payload baytlarını atladı.",
            "Validate payload presence before signing/submission.",
            "İmza/gönderim öncesi payload varlığını doğrulayın.",
        ),
        BlockError::TaskPayloadTooLarge { .. } => (
            "Task payload too large",
            "Görev payload çok büyük",
            "A single task exceeded the per-task size limit.",
            "Tek görev izin verilen boyutu aştı.",
            "Large blob was submitted as one task.",
            "Büyük veri tek görev olarak gönderildi.",
            "Split payload or use off-chain storage with references.",
            "Payload'ı bölün veya zincir dışı depolama + referans kullanın.",
        ),
        BlockError::TooManyTasks { .. } => (
            "Too many tasks in block",
            "Blokta çok fazla görev",
            "Task count exceeded block safety limit.",
            "Görev sayısı blok güvenlik limitini aştı.",
            "Packing policy allowed oversized task count.",
            "Paketleme politikası görev sınırını aştırdı.",
            "Apply strict max-task cap before block finalization.",
            "Blok finalizasyonundan önce maksimum görev sınırını uygulayın.",
        ),
        BlockError::TotalPayloadTooLarge { .. } => (
            "Total block payload too large",
            "Toplam blok payload çok büyük",
            "The aggregate payload size exceeds the block safety cap.",
            "Toplam payload boyutu blok güvenlik sınırını aşıyor.",
            "Multiple heavy tasks were grouped into one block.",
            "Çok sayıda büyük görev tek blokta birikti.",
            "Add pre-pack payload budgeting before finalize.",
            "Finalize öncesi toplam payload bütçesini kontrol edin.",
        ),
        BlockError::LengthOverflow => (
            "Payload length overflow",
            "Payload uzunluk taşması",
            "Size accounting overflow was detected.",
            "Boyut hesaplamasında taşma tespit edildi.",
            "Unexpected data magnitude or malformed input.",
            "Beklenmeyen veri büyüklüğü veya bozuk girdi.",
            "Tighten input limits and inspect anomaly traffic.",
            "Girdi limitlerini daraltın ve anomali trafiğini inceleyin.",
        ),
        BlockError::InvalidBlockHeight => (
            "Invalid block height",
            "Geçersiz blok yüksekliği",
            "Block height is inconsistent with parent linkage.",
            "Blok yüksekliği parent bağlantısı ile tutarsız.",
            "Fork/replay/wrong parent selection.",
            "Fork/replay/yanlış parent seçimi.",
            "Recompute chain link and fork-choice decision.",
            "Zincir bağlantısını ve fork-choice kararını tekrar hesaplayın.",
        ),
        BlockError::InvalidPreviousHash => (
            "Invalid previous hash",
            "Geçersiz previous hash",
            "Previous hash does not match the parent header hash.",
            "Previous hash parent header hash ile uyuşmuyor.",
            "Wrong parent or data corruption.",
            "Yanlış parent veya veri bozulması.",
            "Verify parent selection and source integrity.",
            "Parent seçimini ve veri bütünlüğünü doğrulayın.",
        ),
        BlockError::DuplicateTaskId => (
            "Duplicate task id",
            "Tekrarlanan task id",
            "A task id appears more than once in the same block.",
            "Aynı task id aynı blokta birden fazla kez yer alıyor.",
            "Mempool deduplication was bypassed.",
            "Mempool deduplikasyon adımı atlanmış.",
            "Reject duplicate ids before block assembly.",
            "Block assembly öncesi duplicate id'leri reddedin.",
        ),
        BlockError::InvalidTimestamp => (
            "Invalid timestamp",
            "Geçersiz zaman damgası",
            "Timestamp is zero or outside accepted semantics.",
            "Zaman damgası sıfır veya kabul edilen semantik dışında.",
            "Clock source or timestamp assignment bug.",
            "Saat kaynağı veya timestamp atama hatası.",
            "Standardize timestamp generation through one helper path.",
            "Timestamp üretimini tek yardımcı fonksiyon üzerinden standardize edin.",
        ),
        BlockError::InvalidProducer => (
            "Invalid producer identity",
            "Geçersiz üretici kimliği",
            "Producer key is invalid or mismatched.",
            "Üretici anahtarı geçersiz veya uyuşmuyor.",
            "Wrong key material or role mapping.",
            "Yanlış anahtar materyali veya role mapping.",
            "Re-check validator key bundle and consensus role mapping.",
            "Validator key bundle ve consensus role mapping'i tekrar doğrulayın.",
        ),
        BlockError::InvalidStateRoot => (
            "Invalid state root",
            "Geçersiz state root",
            "State root commitment does not match expected transition.",
            "State root taahhüdü beklenen state transition ile uyuşmuyor.",
            "State transition or Merkle computation mismatch.",
            "State transition veya Merkle hesaplaması uyumsuz.",
            "Replay transition and compare commitment pipeline.",
            "Transition'ı replay edin ve taahhüt hattını karşılaştırın.",
        ),
        BlockError::InvalidTaskRoot => (
            "Invalid task root",
            "Geçersiz task root",
            "Task root commitment is inconsistent.",
            "Task root taahhüdü tutarsız.",
            "Task ordering or canonical serialization drifted.",
            "Task sıralaması veya canonical serialization kaymış.",
            "Audit task ordering and canonical hash pipeline.",
            "Task sıralaması ve canonical hash hattını denetleyin.",
        ),
        BlockError::HashingFailed => (
            "Hashing failed",
            "Hash hesaplama başarısız",
            "Cryptographic hash pipeline failed.",
            "Kriptografik hash hattı başarısız oldu.",
            "Unexpected hash input format or boundary issue.",
            "Beklenmeyen hash girdi formatı veya sınır problemi.",
            "Verify canonical encoding and hash preconditions.",
            "Canonical encoding ve hash önkoşullarını doğrulayın.",
        ),
        BlockError::SerializationFailed => (
            "Serialization failed",
            "Serileştirme başarısız",
            "Report could not be serialized safely.",
            "Rapor güvenli şekilde serileştirilemedi.",
            "Unexpected field or incompatible schema path.",
            "Beklenmeyen alan veya uyumsuz şema akışı.",
            "Validate schema contract and serializer assumptions.",
            "Şema sözleşmesini ve serializer varsayımlarını doğrulayın.",
        ),
    };

    ErrorDescriptor {
        code: err.code().to_string(),
        global_code: global,
        title: localized(locale, title_en, title_tr),
        plain_message: localized(locale, msg_en, msg_tr),
        probable_cause: localized(locale, cause_en, cause_tr),
        operator_action: localized(locale, action_en, action_tr),
    }
}

impl ReportLanguagePack for DefaultReportLanguagePack {
    fn validation_started_title(&self, locale: ReportLocale) -> String {
        localized(locale, "Validation started", "Doğrulama başlatıldı")
    }

    fn validation_started_message(&self, locale: ReportLocale) -> String {
        localized(
            locale,
            "The block is being validated against protocol rules.",
            "Blok protokol kurallarına göre doğrulanıyor.",
        )
    }

    fn wait_action(&self, locale: ReportLocale) -> String {
        localized(locale, "Wait for completion.", "Tamamlanmasını bekleyin.")
    }

    fn accepted_title(&self, locale: ReportLocale) -> String {
        localized(locale, "Block accepted", "Blok kabul edildi")
    }

    fn accepted_message(&self, locale: ReportLocale) -> String {
        localized(
            locale,
            "The block passed all validation checks.",
            "Blok tüm doğrulama kontrollerinden geçti.",
        )
    }

    fn accepted_action(&self, locale: ReportLocale) -> String {
        localized(
            locale,
            "Continue to chain inclusion phase.",
            "Zincire dahil etme adımına devam edin.",
        )
    }

    fn probable_cause_label(&self, locale: ReportLocale) -> String {
        localized(locale, "Probable cause", "Olası neden")
    }

    fn describe_error(&self, err: BlockError, locale: ReportLocale) -> ErrorDescriptor {
        describe_block_error_with_locale(err, locale)
    }
}

/// Produces a full, user-friendly report for block validation outcomes.
#[must_use]
pub fn build_block_validation_report(block: &Block) -> BlockValidationReport {
    build_block_validation_report_with_locale(block, ReportLocale::En)
}

/// Localized report constructor for CLI/Desktop callers.
#[must_use]
pub fn build_block_validation_report_with_locale(
    block: &Block,
    locale: ReportLocale,
) -> BlockValidationReport {
    build_block_validation_report_with_pack(block, locale, &DefaultReportLanguagePack)
}

/// Localized report constructor that accepts an external language pack.
#[must_use]
pub fn build_block_validation_report_with_pack(
    block: &Block,
    locale: ReportLocale,
    pack: &dyn ReportLanguagePack,
) -> BlockValidationReport {
    let mut events = vec![ValidationEvent {
        event_type: ValidationEventType::Info,
        code: "BLOCK_VALIDATION_STARTED".to_string(),
        title: pack.validation_started_title(locale),
        message: pack.validation_started_message(locale),
        action: pack.wait_action(locale),
    }];

    let result = block.validate();
    let mut primary_error_code = None;

    match result {
        Ok(()) => {
            events.push(ValidationEvent {
                event_type: ValidationEventType::Info,
                code: "BLOCK_VALIDATION_ACCEPTED".to_string(),
                title: pack.accepted_title(locale),
                message: pack.accepted_message(locale),
                action: pack.accepted_action(locale),
            });
        }
        Err(err) => {
            let desc = pack.describe_error(err, locale);
            primary_error_code = Some(desc.code.clone());
            events.push(ValidationEvent {
                event_type: ValidationEventType::Error,
                code: format!("{} ({})", desc.code, desc.global_code),
                title: desc.title.clone(),
                message: format!(
                    "{} {}: {}",
                    desc.plain_message,
                    pack.probable_cause_label(locale),
                    desc.probable_cause
                ),
                action: desc.operator_action.clone(),
            });
        }
    }

    BlockValidationReport {
        locale,
        accepted: primary_error_code.is_none(),
        block_height: block.header.height,
        block_type: block.header.block_type,
        task_count: block.task_count(),
        total_payload_bytes: block.total_payload_bytes(),
        primary_error_code,
        events,
    }
}

/// Produces full validation evidence for CLI/desktop integrations.
pub fn build_validation_envelope(block: &Block) -> Result<ValidationEnvelope, BlockError> {
    build_validation_envelope_with_locale(block, ReportLocale::En)
}

/// Localized validation envelope constructor.
pub fn build_validation_envelope_with_locale(
    block: &Block,
    locale: ReportLocale,
) -> Result<ValidationEnvelope, BlockError> {
    build_validation_envelope_with_pack(block, locale, &DefaultReportLanguagePack)
}

/// Localized validation envelope constructor with custom language pack.
pub fn build_validation_envelope_with_pack(
    block: &Block,
    locale: ReportLocale,
    pack: &dyn ReportLanguagePack,
) -> Result<ValidationEnvelope, BlockError> {
    let report = build_block_validation_report_with_pack(block, locale, pack);
    let report_json = serde_json::to_vec(&report).map_err(|_| BlockError::SerializationFailed)?;
    let report_hash_hex = format!("{:x}", Sha256::digest(report_json));
    let header_hash_hex = hex::encode(block.header_hash());

    let evidence = ValidationEvidence {
        schema_version: "aoxc.block.validation.v1".to_string(),
        ruleset_version: "aoxc.block.ruleset.v1".to_string(),
        header_hash_hex,
        report_hash_hex,
    };

    Ok(ValidationEnvelope {
        cli_summary: report.to_human_text(),
        report,
        evidence,
    })
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
        let err =
            Block::new_active_with_timestamp(7, 1, bytes32(1), bytes32(2), bytes32(3), vec![])
                .expect_err("active block without tasks should fail");

        let desc = describe_block_error(err);
        assert_eq!(desc.code, "BLOCK_ACTIVE_REQUIRES_TASKS");
        assert_eq!(desc.global_code, "AOXC-BLK-0002");
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
        assert!(json.contains("\"locale\": \"En\""));
    }

    #[test]
    fn envelope_contains_global_proof_fields_and_human_summary() {
        let task = Task::new(
            bytes32(11),
            Capability::UserSigned,
            TargetOutpost::AovmNative,
            vec![9, 9, 9],
        )
        .expect("task should build");

        let block =
            Block::new_active_with_timestamp(3, 100, ZERO_HASH, bytes32(7), bytes32(6), vec![task])
                .expect("block should build");

        let envelope = build_validation_envelope(&block).expect("envelope must build");
        assert_eq!(envelope.evidence.schema_version, "aoxc.block.validation.v1");
        assert!(!envelope.evidence.header_hash_hex.is_empty());
        assert!(!envelope.evidence.report_hash_hex.is_empty());
        assert!(envelope.cli_summary.contains("Status: ACCEPTED"));
    }

    #[test]
    fn report_supports_secondary_locale_with_simple_switch() {
        let err = BlockError::DuplicateTaskId;
        let desc = describe_block_error_with_locale(err, ReportLocale::Tr);
        assert_eq!(desc.global_code, "AOXC-BLK-0013");
        assert!(desc.title.contains("task"));

        let en = ReportLocale::parse("en-US");
        let tr = ReportLocale::parse("tr-TR");
        assert_eq!(en, ReportLocale::En);
        assert_eq!(tr, ReportLocale::Tr);
    }

    struct PiratePack;

    impl ReportLanguagePack for PiratePack {
        fn validation_started_title(&self, _: ReportLocale) -> String {
            "Ahoy validation".into()
        }
        fn validation_started_message(&self, _: ReportLocale) -> String {
            "Checking block rules on the high seas.".into()
        }
        fn wait_action(&self, _: ReportLocale) -> String {
            "Hold steady.".into()
        }
        fn accepted_title(&self, _: ReportLocale) -> String {
            "Ship is approved".into()
        }
        fn accepted_message(&self, _: ReportLocale) -> String {
            "All checks passed.".into()
        }
        fn accepted_action(&self, _: ReportLocale) -> String {
            "Sail to inclusion.".into()
        }
        fn probable_cause_label(&self, _: ReportLocale) -> String {
            "Likely reason".into()
        }
        fn describe_error(&self, err: BlockError, _: ReportLocale) -> ErrorDescriptor {
            ErrorDescriptor {
                code: err.code().to_string(),
                global_code: global_error_code(err).as_str().to_string(),
                title: "Storm error".into(),
                plain_message: "The block hit rough weather.".into(),
                probable_cause: "Unknown waters".into(),
                operator_action: "Check instruments".into(),
            }
        }
    }

    #[test]
    fn custom_language_pack_can_override_texts_without_kernel_edits() {
        let task = Task::new(
            bytes32(22),
            Capability::UserSigned,
            TargetOutpost::AovmNative,
            vec![1],
        )
        .expect("task should build");
        let block =
            Block::new_active_with_timestamp(9, 100, ZERO_HASH, bytes32(1), bytes32(2), vec![task])
                .expect("block should build");

        let report = build_block_validation_report_with_pack(&block, ReportLocale::En, &PiratePack);
        assert!(report.events[0].title.contains("Ahoy"));
    }
}
