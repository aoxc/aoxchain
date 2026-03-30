# AOXChain Klasör Bazlı Gap Analizi ve Tamamlama Planı (TR)

Bu doküman, repository'nin her ana klasörünü **ana sorumluluk**, **mevcut durum**, **eksik/gap**, ve **mainnet üstü kalite hedefi** açısından özetler.

## 1) Kök dizin

### `Cargo.toml`
- **Rol:** Workspace crate kompozisyonu ve ortak dependency yönetimi.
- **Durum:** Çok-crate mimari net; modüler büyümeye uygun.
- **Gap:** Core workspace ile desktop (Tauri) kalite kapıları ayrı yönetilmediğinde Linux sistem paketleri eksik ortamlarda CI kırılabiliyor.
- **Hedef:** branch protection ile eşleşen, core + desktop ayrışmış required checks matrisi.

### `README.md`
- **Rol:** Yeni geliştirici ve operatör için giriş kapısı.
- **Durum:** Bu PR ile bozuk/tekrarlı içerik temizlenip yeniden yapılandırıldı.
- **Gap:** Kurumsal release süreci (SBOM, imzalı artefact) adım adım akış henüz ayrı runbook olarak linklenmeli.
- **Hedef:** 10 dakikada onboarding + deterministic smoke tamamlanabilir olmalı.

## 2) `crates/` (çekirdek kod tabanı)

### `crates/aoxcore`
- **Rol:** Identity, tx, mempool, genesis gibi temel protokol primitifleri.
- **Güçlü taraf:** Domain model ayrımı yüksek, çekirdek bağımlılık noktası net.
- **Gap:** Property-based test kapsamı artırılmalı (özellikle tx hash/serialization kritik yüzeyleri).

### `crates/aoxcunity`
- **Rol:** Consensus state machine, quorum, vote pool, proposer rotation, fork-choice.
- **Güçlü taraf:** Ayrık modül tasarımı finality akışını okunur kılıyor.
- **Gap:** Çok-node adversarial test senaryoları (eşzamanlı çatallı block yayınları) daha kapsamlı olmalı.

### `crates/aoxcvm`
- **Rol:** EVM/WASM/Sui/Cardano lane uyumluluk katmanı.
- **Güçlü taraf:** Lane bazlı ayrım ileriye dönük birlikte çalışabilirlik için doğru.
- **Gap:** Lane'ler arası deterministik gas accounting karşılaştırmalı test matrisi gerekli.

### `crates/aoxcnet`
- **Rol:** Gossip, discovery, sync yüzeyleri.
- **Güçlü taraf:** Ağ sorumluluğu ayrı crate’te izole.
- **Gap:** Gerçek transport + dayanıklılık testleri (latency/jitter/partition) genişletilmeli.

### `crates/aoxcrpc`
- **Rol:** HTTP/gRPC/WebSocket API giriş noktası.
- **Güçlü taraf:** Çoklu protokol erişim mimarisi mevcut.
- **Gap:** API versiyonlama ve backward-compat policy netleşmeli.

### `crates/aoxcmd`
- **Rol:** Node bootstrap, lifecycle, runtime wiring, operasyonel komutlar.
- **Bu PR katkısı:** `node/engine.rs` tekrar eden kopya bloklardan arındırıldı, tekil/derlenebilir hale getirildi.
- **Gap:** Runtime/consensus entegrasyonunda integration test sayısı artırılmalı.

### `crates/aoxckit`
- **Rol:** Operatör araçları ve keyforge komutları.
- **Gap:** Key yaşam döngüsü için revocation/rotation senaryolarına “golden test” seti eklenmeli.

### Diğer destek crate'ler (`aoxcdata`, `aoxcai`, `aoxconfig`, `aoxcsdk`, `aoxcexec`, `aoxcontract`, `aoxcenergy`, `aoxclibs`, `aoxcmob`)
- **Rol:** Data, AI policy, konfigürasyon, SDK, yardımcı runtime bileşenleri.
- **Gap:** Cross-crate compatibility matrix (hangi sürüm hangi sürümle garanti) dokümante edilmeli.

## 3) `docs/`
- **Rol:** Mimari, operasyon, audit, mainnet hazırlığı.
- **Durum:** Teknik analiz ve mainnet blueprint mevcut.
- **Gap:** Bazı dökümanlarda kapanan maddeler (örn. on-call ve incident runbook) hâlâ açık eksik gibi raporlanıyor; içerik senkronizasyonu gerekli.

## 4) `models/`
- **Rol:** Örnek risk/policy model dosyaları.
- **Gap:** Model schema versioning ve migration yönergeleri eklenmeli.

## 5) `tests/`
- **Rol:** Entegrasyon testi için bağımsız workspace yüzeyi.
- **Gap:** Multi-node deterministic senaryolar + fault injection (drop/delay/reorder) genişletilmeli.

---

## Mainnet'i aşacak kalite için önerilen zorunlu kalite kapıları

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --exclude aoxchub --all-targets`
4. `cargo check -p aoxchub --all-targets` (desktop bağımlılıkları kurulu runner üzerinde)
5. Deterministic multi-node simulation suite (en az 3 node, byzantine-lite senaryo)
6. Release artefact signing + provenance attestation (SLSA seviyesi hedeflenerek)

## Bu PR'daki net kazanım

- `aoxcmd` engine katmanında compile kıran tekrarlar temizlenerek workspace bütünlüğü iyileştirildi.
- README üretim-vizyonu ve operatör akışı açısından yeniden kullanılabilir hale getirildi.
- Klasör bazlı gap analizi dokümante edilerek eksiklerin kapatılmasına net bir icra planı oluşturuldu.
