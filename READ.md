# AOXChain Hızlı Operasyon Rehberi (TR)

Bu dosya, gerçek ağ (mainnet) ve test ağı (testnet) için temel kullanım komutlarını kısa ve kullanıcı dostu şekilde sunar.

## 1) Derleme ve paketleme

```bash
make quality-quick
make package-bin
```

## 2) Testnet (önerilen ilk adım)

```bash
./bin/aoxc key-bootstrap --profile testnet --password 'TEST#Secure2026!'
./bin/aoxc node-bootstrap
./bin/aoxc produce-once --tx 'testnet-smoke-1'
./bin/aoxc runtime-status --trace standard --tps 25.0 --peers 8 --error-rate 0.001
```

## 3) Mainnet (açık onay gerekir)

```bash
./bin/aoxc key-bootstrap --profile mainnet --allow-mainnet --password 'AOXc#Mainnet2026!'
```

veya

```bash
AOXC_ALLOW_MAINNET_KEYS=true ./bin/aoxc key-bootstrap --profile mainnet --password 'AOXc#Mainnet2026!'
```

## 4) Sürekli blok üretimi + detaylı log

```bash
MAX_ROUNDS=0 SLEEP_SECS=2 LOG_FILE=./logs/continuous-producer.log ./scripts/continuous_producer.sh
```

## 5) Log inceleme

```bash
tail -n 100 ./logs/continuous-producer.log
rg "ERROR|OK|round=" ./logs/continuous-producer.log
```

Detaylı İngilizce operasyon akışı için `README.md` içindeki **Mainnet/Testnet Operational Playbook** bölümüne bakın.
