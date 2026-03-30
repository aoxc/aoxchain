# AOXC Kalıcı Testnet (Full) Planı

Bu doküman, **3 node'lu kalıcı testnet** için zorunlu (P0), güçlü öneri (P1) ve profesyonel seviye (P2) gereksinimleri operasyonel olarak uygulanabilir hale getirir.

## Hedef Topoloji (Minimum Doğru Tasarım)

- **Node-1:** validator + seed (public p2p açık, public rpc kapalı)
- **Node-2:** validator (internal yönetim, public p2p açık)
- **Node-3:** validator + public RPC (kullanıcı giriş noktası)

Referans dosyalar:
- `configs/environments/testnet/validators.json`
- `configs/environments/testnet/bootnodes.json`
- `configs/environments/testnet/network-metadata.json`

## P0 — Kesin Zorunlu Kontroller

### 1) Sabit ağ kimliği

Aşağıdaki kimlik bir kez belirlenir ve değiştirilmez:
- `chain_id`: `2626010001`
- `network_id`: `aoxc-testnet-2626-002`
- genesis dosyası + sha256 hash
- başlangıç validator seti
- bootnode adresleri

Dosyalar:
- `configs/environments/testnet/manifest.v1.json`
- `configs/environments/testnet/genesis.v1.json`
- `configs/environments/testnet/genesis.v1.sha256`

### 2) En az 3 node

Validator seti en az 3 aktif node içerir.

### 3) 7/24 servis yönetimi (systemd)

Her node için önerilen unit:

```ini
[Unit]
Description=AOXC Testnet Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=aoxc
WorkingDirectory=/opt/aoxc
Environment=AOXC_HOME=/var/lib/aoxc/testnet
ExecStart=/opt/aoxc/bin/aoxc node-run --home /var/lib/aoxc/testnet
Restart=always
RestartSec=3
LimitNOFILE=65535
StandardOutput=append:/var/log/aoxc/node.log
StandardError=append:/var/log/aoxc/node.err.log

[Install]
WantedBy=multi-user.target
```

Uygulama:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now aoxc-testnet.service
sudo systemctl status aoxc-testnet.service
```

### 4) Public RPC

En az bir sabit domain ile public RPC yayınlanır:
- `https://rpc1.testnet.aoxchain.io`

### 5) Peer discovery / sabit peer modeli

- `bootnodes.json` içinde seed node listesi
- node config içinde persistent peer listesi
- p2p portları (örn. `26656`) firewall'da açık

### 6) Kalıcı blok üretimi

- reboot sonrası üretim devam eder
- tek node kaybında zincir tamamen durmaz (3 node quorum)
- node state diskte korunur (`/var/lib/aoxc/...`)

### 7) Key yönetimi

- her validator için ayrı consensus/network key
- offline şifreli yedek
- erişim kontrolü (en az dosya izinleri + secret manager)
- key kaybı / key leak runbook'u

### 8) Backup / restore

Minimum yedek seti:
- genesis
- manifest/profile/release-policy
- validators/bootnodes
- key material
- snapshot/state

Restore tatbikatı zorunludur (en az aylık).

### 9) Ağ metadata yayını

`network-metadata.json` ile tek noktadan yayın:
- network name
- chain id
- rpc url
- symbol
- explorer/faucet/status url
- genesis hash referansı

### 10) Operasyon runbook

Runbook en az şu başlıkları içermelidir:
- node kurulum
- node restart
- key kaybı/leak
- node düşmesi
- sürüm yükseltme ve rollback

## P1 — Güçlü Önerilenler

1. Monitoring: block height, peer count, disk/ram/cpu, rpc health, validator liveness
2. Alerting: block durmuş, peer düşmüş, disk doluyor, rpc down
3. Yedek RPC: `rpc2.testnet...`
4. Reverse proxy + TLS + rate limit
5. Faucet (daily/IP/captcha limit)
6. Explorer/indexer
7. Sürümleme politikası (tag + binary hash + rollout + rollback)

## P2 — Profesyonel Seviye

1. Multi-region / multi-provider node dağıtımı
2. Immutable artifact paketi (genesis hash, binary hash, SBOM, provenance)
3. Validator rotation prosedürü
4. Snapshot dağıtım servisi
5. Public status page + incident duyuruları
6. Incident response playbook (compromise/leak/partition/halt)

## “Tamamdır” Kriteri (10/10)

Aşağıdakilerin hepsi `evet` ise kalıcı testnet kabul edilir:

1. Genesis sabit ve hash yayımlandı
2. 3 node aynı ağda birbirine bağlı
3. Reboot sonrası ağ devam ediyor
4. En az 1 public RPC açık
5. Cüzdan ile bağlanabiliyor
6. Blok üretimi sürekli sürüyor
7. Config + key yedekleri var
8. Snapshot/restore test edildi
9. Monitoring + alarm var
10. Runbook yazıldı

## Otomatik Dosya Geçidi (Gate)

Repository içindeki minimum P0 konfigürasyon bütünlüğünü doğrulamak için:

```bash
scripts/validation/persistent_testnet_gate.sh
```

> Bu gate; dosya varlığı, min. validator sayısı, bootnode ve metadata alanlarını doğrular. Canlı node health için monitoring/alerting pipeline'ı ayrıca gereklidir.
