# AOXChain — Advanced Omnichain Execution Chain (Canonical Read)

<p align="center">
  <img src="./logos/aoxc.png" alt="AOXChain Logo" width="220" />
</p>

> **KALICI UYARI / PERMANENT WARNING**  
> Bu ürün **deneysel tasarım ve aktif geliştirme** aşamasındadır. Üretim ortamında (mainnet/kurumsal canlı trafik) kullanımdan önce bağımsız güvenlik denetimi, operasyonel drill, yedekleme/geri dönüş testleri ve hukuki/uyumluluk onayı zorunludur.

---

## 1) Zincirin amacı

AOXChain’in amacı; deterministik çekirdek (kernel), çoklu VM yürütme (multi-lane execution), servis katmanı ve operatör düzlemini net sınırlarla ayırarak:

- güvenli finalite,
- tekrar üretilebilir yürütme (replay-stability),
- ölçülebilir operasyon,
- kurumsal denetlenebilirlik

sağlayan bir zincir standardı sunmaktır.

---

## 2) Zincirin farkı (value proposition)

AOXChain’i ayrıştıran temel özellikler:

1. **Deterministik kernel yaklaşımı:** Kanonik durum değişimi yalnızca çekirdek kurallarla oluşur.  
2. **Multi-VM lane modeli:** Native + EVM + WASM + uyumluluk lane’leri politikaya bağlı yürütülür.  
3. **Operator-plane ayrımı:** CLI/desktop kontrol sağlar; konsensüs otoritesi olamaz.  
4. **Release evidence disiplini:** build-manifest, sbom, provenance, audit artefaktlarıyla sürüm doğrulanır.  
5. **Kurumsal çalışma modeli:** runbook, incident, quality gate ve rollback süreçleri dokümante edilir.

---

## 3) Mimari içerik (repo haritası)

- `crates/aoxcore` → blok/tx/state/receipt çekirdeği
- `crates/aoxcunity` → konsensüs, quorum, finality, safety
- `crates/aoxcexec` + `crates/aoxcvm` → VM lane orkestrasyonu ve yürütme
- `crates/aoxcnet` → p2p/gossip/sync/discovery
- `crates/aoxcrpc` → API/RPC yüzeyi
- `crates/aoxcdata` → persistence/index/snapshot alanı
- `crates/aoxcmd` → operasyon CLI
- `crates/aoxchub` → desktop control-plane
- `configs/` → ortam konfigürasyonları (localnet/devnet/testnet/mainnet)
- `tests/` → entegrasyon ve readiness testleri

---

## 4) Neler yapabilir?

AOXChain çalışma yüzeyi aşağıdaki kurumsal ihtiyaçları hedefler:

- ağ ve node bootstrap akışları,
- genesis üretim/doğrulama,
- operator key yönetimi,
- tek seferde blok üretim/doğrulama akışı,
- sağlık ve smoke testleri,
- release readiness ve evidence paketleme,
- desktop wallet/launcher uyumluluk kontrolü.

---

## 5) Komut seti (temel kullanım)

### 5.1 Geliştirici kalite akışı

```bash
make fmt
make check
make test
make clippy
make quality-quick
make quality
```

### 5.2 Sürüm ve kimlik doğrulama

```bash
make build-release
make package-bin
make version
make manifest
make policy
```

`make package-bin` komutu release binary dosyasını varsayılan olarak `~/.aoxc/bin/aoxc` altına kurar ve geriye uyumluluk için `./bin/aoxc` sembolik linkini üretir.

### 5.3 Yerel zincir / operasyon akışı

```bash
make dev-bootstrap
make run-local
make real-chain-run
make real-chain-health
make real-chain-tail
```

### 5.4 CLI komut ailesi (aoxc)

```bash
aoxc key-bootstrap --profile testnet --name validator-01 --password '<SECRET>'
aoxc keys-inspect
aoxc genesis-init --home .aoxc-local --chain-num 1001 --block-time 6 --treasury 1000000000000
aoxc genesis-validate --home .aoxc-local
aoxc node-bootstrap --home .aoxc-local
aoxc produce-once --home .aoxc-local --tx 'hello-aoxc'
aoxc node-health --home .aoxc-local
aoxc network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0 --payload AOXC_REAL_HEALTH
```

> Not: Komut setinin güncel kapsamı için `make help` ve `aoxc help` çıktıları esas alınır.

---

## 6) Arayüz (desktop) ile etkileşim modeli

`aoxchub` zincirin **kontrol düzlemi**dir; protokol otoritesi değildir.

Desktop etkileşim hedefleri:

- ağ profili yönetimi (mainnet/testnet/devnet),
- node sağlık görünürlüğü,
- release-ready checklist takibi,
- wallet uyumluluk ve yönlendirme kontrolü,
- olay (incident) anında operatöre yönlendirici eylem akışı.

Kurumsal tasarım ilkesi: UI yalnızca gözlemlenebilirlik + yönetişim + emniyetli operasyon sağlar; konsensüs kararını etkilemez.

---

## 7) Cüzdan / anahtar üretimi (operator-grade)

AOXChain’de cüzdan/anahtar yaşam döngüsü, operatör anahtar disiplini üzerinden yürütülür:

1. `key-bootstrap` ile operatör anahtar materyali oluşturulur.
2. Anahtar materyali şifre korumalı zarf/envelope mantığıyla saklanır.
3. `keys-inspect` / `keys-show-fingerprint` / `keys-verify` ile doğrulama yapılır.
4. Node bootstrap sonrası imza ve üretim akışında bu materyal kullanılır.

Önerilen kurumsal pratik:

- güçlü parola politikası,
- anahtar rotasyonu,
- erişim ayrıştırması,
- yedekleme + geri yükleme testinin düzenli yapılması,
- üretim anahtarlarının HSM/kurumsal saklama modeline taşınması.

---

## 8) Kimler için?

- Zincir çekirdeği geliştiren protokol ekipleri,
- Kurumsal validator/operator ekipleri,
- Çoklu VM üstünde uygulama geliştiren ekipler,
- Denetlenebilir release süreci isteyen teknik yönetişim organizasyonları.

---

## 9) Deneysel tasarım notu (kalıcı)

**Bu repository’deki zincir tasarımı deneysel/evrimsel bir mühendislik programıdır.**

Aşağıdaki riskleri kabul etmeden canlı kullanım yapılmamalıdır:

- geriye dönük uyumluluk değişebilir,
- konfigürasyon/komut davranışları sürümler arasında güncellenebilir,
- performans ve dayanıklılık profili çevreye göre farklılık gösterebilir,
- ek güvenlik kontrolleri olmadan kurumsal üretime alınmamalıdır.

---

## 10) Tek resmi roadmap ve referanslar

- Tek yol haritası: [ROADMAP.md](./ROADMAP.md)
- Portal: [README.md](./README.md)
- Mimari referans: `docs/ARCHITECTURE.md`
- Execution modeli: `docs/EXECUTION_MODEL.md`
- Sistem invariantları: `docs/SYSTEM_INVARIANTS.md`
- Lisans: [LICENSE](./LICENSE)
