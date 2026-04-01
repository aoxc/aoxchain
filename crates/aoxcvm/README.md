# AOXCVM

AOXCVM, AOXChain için tek kanonik yürütme katmanıdır: **policy-bound deterministic object machine**.

## Design identity

- **Canonical L1 truth layer:** zincirin resmi execution semantics yüzeyi.
- **Object-native state:** ham slot yerine typed object lifecycle.
- **Capability-native authority:** yetki, uygulama detayı değil VM primitive’i.
- **Crypto agility + PQ readiness:** klasik, PQ ve hibrit kimlik geçişi.
- **Governed evolution:** feature gates + protocol versioning ile kontrollü değişim.

## Why not EVM/WASM/Move parity?

AOXCVM bilinçli olarak "başka bir VM uyumluluk katmanı" değildir.

- EVM’nin slot-centric modelini, object class + policy + owner üst yapısıyla değiştirir.
- WASM’in genel amaçlı sandbox yaklaşımı yerine deterministic, chain-purpose-built host sınırı uygular.
- Move’daki resource disiplinini korur; ancak auth, governance, upgrade ve syscall kurallarını dil seviyesinden protokol seviyesine taşır.

## Canonical execution flow

1. **Admission:** chain/replay/auth/budget/capability intent doğrulaması.
2. **Authorization:** scheme + envelope + nonce + expiry + policy kontrolü.
3. **Resolution:** package/entrypoint/feature-gate/deprecation uyumu.
4. **Verification:** doğrulanmamış bytecode yürütmeye giremez.
5. **Execution:** bounded machine state + overlay/journal mutation.
6. **Policy re-check:** capability + syscall + governance constraints.
7. **Commit & finalize:** diff, receipt, commitment ve post-state sonuçları.

## Product rule

> **Code may execute, but authority must be proven.**
