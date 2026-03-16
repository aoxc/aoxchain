<div align="center">
  <a href="https://github.com/aoxc/aoxcore">
    <img src="logos/aoxc_transparent.png" alt="AOXCHAIN Logo" width="170" />
  </a>

# 🚀 AOXChain — Operator Guide (Chronological, Practical, Production-Oriented)

**AOXChain**, deterministic ve güvenlik odaklı, çok-crate Rust blockchain çalışma alanıdır.  
Bu README, “ilk kurulumdan 3 doğrulayıcı role kadar” adım adım **kronolojik operatör akışı** verir.

</div>

---

## 📚 İçindekiler

1. [Mimari ve Node Rolleri](#-1-mimari-ve-node-rolleri)
2. [Hızlı Başlangıç (Build + Kalite)](#-2-hızlı-başlangıç-build--kalite)
3. [Kronolojik Main Flow (Genesis → Node → Wallet → Stake)](#-3-kronolojik-main-flow-genesis--node--wallet--stake)
4. [Node Bağlantısı / Peer Adresi / Node ID Toplama](#-4-node-bağlantısı--peer-adresi--node-id-toplama)
5. [3 Doğrulayıcı Tipi: Validator / DAO / AI](#-5-3-doğrulayıcı-tipi-validator--dao--ai)
6. [Stake ve Ekonomi Komutları](#-6-stake-ve-ekonomi-komutları)
7. [Make Komutlarıyla Operasyon](#-7-make-komutlarıyla-operasyon)
8. [Operasyon Güvenliği ve Gerçek Ağ Notları](#-8-operasyon-güvenliği-ve-gerçek-ağ-notları)
9. [Komut Referansı (aoxcmd)](#-9-komut-referansı-aoxcmd)

---

## 🧠 1) Mimari ve Node Rolleri

AOXChain ana bileşenleri:

- `aoxcore`: identity, genesis, tx, mempool, temel protokol primitives
- `aoxcunity`: quorum/round/vote/finality akışları
- `aoxcnet`: discovery/gossip/sync + transport yüzeyleri
- `aoxcrpc`: RPC ingress (HTTP/gRPC/WebSocket)
- `aoxcvm`: lane bazlı execution compatibility
- `aoxcmd` ve `aoxckit`: operatör CLI ve key/tooling

### 🎭 3 doğrulayıcı tipi (operasyonel model)

- **Validator Node**
  - Blok üretimi / doğrulama / ağ canlılığı.
- **DAO Governance Node**
  - Ağ yönetimi, oylama, denetim, yönetişim süreçlerine katılım.
- **AI Security Node**
  - Kurallı/model kontrollü güvenlik katkıları (anomali gözlemi, policy sinyali, risk analizi).
  - Ağa güvenlik katkısı ölçülebilir olduğunda ödül mekanizmasına dahil edilir.

> Not: Bu rollerin ekonomik/policy enforcement detayları release policy + governance dokümanlarında netleştirilmelidir.

---

## 🛠️ 2) Hızlı Başlangıç (Build + Kalite)

Önkoşullar:
- Rust stable
- cargo

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

Opsiyonel kısa kalite akışı:

```bash
make quality-quick
```

---

## 🧭 3) Kronolojik Main Flow (Genesis → Node → Wallet → Stake)

Aşağıdaki sıra, sıfırdan “çalışan operatör akışı” için önerilen kronolojidir.

### 3.1 Veri dizini belirle (izole test)

```bash
export AOXC_HOME=$PWD/.aoxc-local
```

### 3.2 Cüzdan / node kimliği üret (key-bootstrap)

> Bu adım hem node identity hem de wallet-benzeri key materyalini üretir.

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --profile testnet \
  --name validator-01 \
  --password "TEST#Secure2026!"
```

### 3.3 Genesis oluştur

```bash
cargo run -p aoxcmd -- genesis-init --chain-num 1001 --block-time 6 --treasury 1000000000000
```

### 3.4 Node bootstrap

```bash
cargo run -p aoxcmd -- node-bootstrap
```

### 3.5 İlk blok üretimi (smoke)

```bash
cargo run -p aoxcmd -- produce-once --tx "boot-sequence-1"
```

### 3.6 Sürekli local üretim (node-run)

```bash
cargo run -p aoxcmd -- node-run --rounds 20 --sleep-ms 1000 --tx-prefix AOXC_RUN
```

### 3.7 Ağ probe (real-network)

```bash
cargo run -p aoxcmd -- real-network \
  --rounds 10 \
  --timeout-ms 3000 \
  --pause-ms 250 \
  --bind-host 127.0.0.1 \
  --port 0
```

### 3.8 Runtime ve readiness kontrol

```bash
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- interop-readiness
cargo run -p aoxcmd -- interop-gate \
  --audit-complete true \
  --fuzz-complete true \
  --replay-complete true \
  --finality-matrix-complete true \
  --slo-complete true \
  --enforce
```

---

## 🌐 4) Node Bağlantısı / Peer Adresi / Node ID Toplama

Operatör için temel yaklaşım:

1. Her node’da `key-bootstrap` ile kimlik üret.
2. `port-map` ile network ve RPC portlarını doğrula.
3. Node metadata (sertifika/pasaport/ID türevleri) çıktılarından peer listesi çıkar.
4. Farklı host/IP üstünde çok node ayağa kaldırıp karşılıklı erişimi test et.

Port görünümü:

```bash
cargo run -p aoxcmd -- port-map
```

Canlı TCP smoke:

```bash
cargo run -p aoxcmd -- network-smoke --bind-host 127.0.0.1 --port 9600 --payload "HELLO"
```

> Üretim için loopback değil, farklı host’larda ve gerçek firewall/policy ile test edilmesi gerekir.

---

## 🧩 5) 3 Doğrulayıcı Tipi: Validator / DAO / AI

### ✅ Validator Node

- Amaç: blok üretimi, doğrulama, network canlılığı.
- Önerilen komutlar:

```bash
cargo run -p aoxcmd -- node-bootstrap
cargo run -p aoxcmd -- node-run --rounds 0 --sleep-ms 2000 --tx-prefix VALIDATOR
```

> `--rounds 0` sonsuz loop davranışı hedefleyen operasyon yaklaşımıdır (release davranışına göre doğrulanmalı).

### 🏛️ DAO Governance Node

- Amaç: yönetişim katılımı, oylama, denetim süreçleri.
- Operasyonel görevler:
  - release gate ve production-audit takibi,
  - karar kayıtlarının izlenmesi,
  - politika/parametre değişim teklifleri ve oy doğrulaması.

Örnek kontrol:

```bash
cargo run -p aoxcmd -- production-audit --ai-model-signed true --ai-prompt-guard true --ai-anomaly-detection true --ai-human-override true
```

### 🤖 AI Security Node

- Amaç: policy ile sınırlandırılmış model katkısı.
- Kural çerçevesi:
  - yalnız izinli/model-imzalı AI pipeline,
  - prompt-guard + anomaly detection + human override,
  - güvenlik katkısı ölçümlenebilir telemetri.

Örnek gate:

```bash
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```

---

## 💰 6) Stake ve Ekonomi Komutları

### 6.1 Ekonomi state başlat

```bash
cargo run -p aoxcmd -- economy-init --treasury-supply 1000000000000
```

### 6.2 Treasury transfer

```bash
cargo run -p aoxcmd -- treasury-transfer --to wallet-user-01 --amount 100000
```

### 6.3 Validator stake delege et

```bash
cargo run -p aoxcmd -- stake-delegate --staker wallet-user-01 --validator validator-01 --amount 25000
```

### 6.4 Stake geri çek

```bash
cargo run -p aoxcmd -- stake-undelegate --staker wallet-user-01 --validator validator-01 --amount 5000
```

### 6.5 Durum kontrol

```bash
cargo run -p aoxcmd -- economy-status
```

---

## ⚙️ 7) Make Komutlarıyla Operasyon

### Günlük kalite

```bash
make quality-quick
make quality
```

### Release öncesi

```bash
make quality-release
make package-bin
```

### Güvenlik ve audit

```bash
make audit-install
make audit
```

### Local supervisor

```bash
make supervise-local
```

---

## 🔐 8) Operasyon Güvenliği ve Gerçek Ağ Notları

- Mainnet key üretimi bilinçli olarak korumalıdır:
  - `--allow-mainnet` veya `AOXC_ALLOW_MAINNET_KEYS=true`
- Gerçek ağ iddiası için zorunlu başlıklar:
  - multi-node farklı host testi,
  - partition/rejoin/recovery tatbikatı,
  - TLS/mTLS + RPC access policy,
  - backup/restore runbook,
  - signed artifact + provenance,
  - enforced CI/CD gate.

Türkçe go/no-go dokümanı:

- [`docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`](docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md)

---

## 🧾 9) Komut Referansı (aoxcmd)

```text
vision
compat-matrix
port-map
version
key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
node-bootstrap
produce-once [--tx <payload>]
node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
economy-status [--home <dir>] [--state <file>]
runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
interop-readiness
interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
help
```

Language:
- `--lang <en|tr|es|de>`
- `AOXC_LANG=<code>`

---

## 📄 License

MIT (`LICENSE`)
