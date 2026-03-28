# README.md

> Scope: `configs/environments/testnet`

## Bu klasör ne yapar?
Testnet için genesis, validator, profile, release-policy ve operasyonel metadata dosyalarını içerir.

## İçerik özeti
- Bu klasördeki dosyalar testnet kimliğini (`chain_id`, `network_id`, genesis hash) sabitler.
- `validators.json` en az 3 validator topolojisini referans alır.
- `bootnodes.json` seed/bootnode peer keşif giriş noktalarını içerir.
- `network-metadata.json` public RPC/explorer/faucet gibi kullanıcı metadata'sını tek noktada yayınlar.
- Değişiklik yapılırken ilgili test/uyumluluk etkisi birlikte değerlendirilmelidir.

## Hızlı doğrulama

```bash
scripts/validation/persistent_testnet_gate.sh
```
