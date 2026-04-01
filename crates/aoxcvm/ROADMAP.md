# AOXCVM Roadmap

Bu yol haritası AOXCVM’in vNext hedefini teslim fazlarına böler. Her faz, önceki fazın güvenlik ve determinism koşullarını koruyarak ilerler.

## Phase 0 — Baseline (Tamamlandı)
- Modül ayrımı ve mimari yüzey tanımları.
- Admission/auth/policy/execution için çekirdek primitifler.
- Object-centric state modeli başlangıç tipleri.
- Temel dokümantasyon, şema ve audit dosya iskeleti.

## Phase 1 — Canonical Validation Hardening
- Bytecode format doğrulama kurallarının sıkılaştırılması.
- Feature gate uyumluluk denetimi.
- Replay ve nonce domain kurallarının kapsam genişletmesi.
- Determinism ihlali sınıflarının error modeline tam bağlanması.

## Phase 2 — Execution Core Expansion
- Instruction sınıfları (compute/object/capability/syscall) için gerçek uygulama.
- Bounded stack/heap/pages yönetimi.
- Trap/halt semantics netleştirmesi.
- Dry-run ile canonical run davranış farklarının formal tanımı.

## Phase 3 — Object + Capability Enforcement
- Object lifecycle (create/mutate/lock/tombstone) kurallarının tamamlanması.
- Capability scope çözümleyici ve delegation mekanizması.
- Unauthorized mutation engelleri için kapsamlı invariant seti.

## Phase 4 — Host/Syscall Determinism
- Versioned syscall registry gerçek implementasyonu.
- Host import/export whitelist modeli.
- Nondeterministic yüzeylerin tamamen kapatılması.
- Syscall maliyetlendirme ve authority risk sınıfları.

## Phase 5 — Governance & Upgrade Protocol
- Protocol version registry.
- Deprecation pencereleri ve migration planı formatı.
- Delayed activation epoch mekanizması.
- Package upgrade compatibility denetimleri.

## Phase 6 — PQ Migration Readiness
- ML-DSA / SLH-DSA / hybrid doğrulama pipeline’larının üretim sertleştirmesi.
- Recovery/rotation/threshold yönetim kurallarının finalize edilmesi.
- PQ rollout ve rollback operasyon prosedürleri.

## Phase 7 — Audit & Release Gates
- Tehdit modeli, DOS yüzeyleri ve güven sınırlarının final audit’i.
- Fuzz/bench/test coverage hedeflerinin release eşiğine bağlanması.
- Üretim geçişi için güvenlik ve uyumluluk onay süreçlerinin tamamlanması.

## Success Criteria
- Deterministic replay garanti edilir.
- Unverified bytecode canonical execution’a girmez.
- Capability olmadan kritik mutation yapılamaz.
- Governance dışı feature/protocol değişikliği mümkün değildir.
- PQ geçişi kontrollü ve geri alınabilir şekilde yönetilir.
