# AOXCVM — AOXChain Canonical Virtual Machine

AOXCVM, AOXChain L1 için tek resmi yürütme katmanıdır. Bu crate bir “genel amaçlı sandbox” değil; **policy-bound deterministic object machine** yaklaşımıyla tasarlanmış, yetkiyi ve state mutation’ı protokol seviyesinde denetleyen VM çekirdeğidir.

---

## 1) Bu crate tam olarak nedir?

`crates/aoxcvm` aşağıdaki sorulara tek noktadan cevap veren kanonik VM yüzeyidir:

- Bir işlem AOXChain tarafından **hangi şartlarda kabul edilir**?
- Yetki (authority) nasıl **imza + capability + policy** ile kanıtlanır?
- Kod doğrulama, yürütme, mutation ve commit süreci **deterministik** nasıl tutulur?
- Post-quantum (PQ) geçişi nasıl **governance kontrollü** yapılır?

Kısaca: AOXCVM = zincirin execution truth engine’i.

---

## 2) AOXCVM tasarım kimliği

AOXCVM’in ana kimliği:

- **Canonical L1 execution layer**
- **Object-centric state model**
- **Capability-native authorization**
- **Crypto-agile, PQ-ready identity/auth**
- **Governed feature gates + controlled upgrade**
- **Determinism-first host boundary**

Bu sayede AOXCVM, EVM/WASM/Move çizgisine birebir kopya olmak yerine AOXChain için kurumsal ve denetlenebilir bir execution protokolü sunar.

---

## 3) EVM / WASM / Move'dan bilinçli ayrım

### EVM’den ayrım
- Slot-centric ham storage yerine typed object lifecycle.
- `msg.sender` merkezli app-level yetki yerine VM-level capability semantics.
- Upgrade/compatibility uygulamaya bırakılmaz; protocol/gov katmanına taşınır.

### WASM’den ayrım
- Genel amaçlı runtime değil, zincir amaçlı deterministic execution surface.
- Host erişimi serbest değil; syscall boundary ve registry ile kontrollü.

### Move’dan ayrım
- Resource disiplinine yakın fakat dil-semantik bağımlılığı azaltılmış.
- PQ auth, feature gates, protocol versioning daha merkezi konumlanır.

---

## 4) Katmanlı mimari (9 katman)

1. **Identity & Authorization Layer**
   - Scheme registry (classical + PQ + hybrid)
   - Nonce/replay domain
   - Rotation / recovery / threshold
2. **Transaction Admission Layer**
   - Chain domain, expiry, budget, target, capability intent
3. **Bytecode Verification Layer**
   - Format, opcode legality, bounds, feature gate, determinism rules
4. **VM Execution Core**
   - Machine state, stack/frame, bounded memory, trap/halt
5. **Object State Layer**
   - Identity/Asset/Capability/Contract/Package/Vault/Governance object’leri
6. **Capability Layer**
   - Kritik mutation için explicit capability zorunluluğu
7. **Host / Syscall Boundary**
   - Registry tabanlı, versioned, deterministic syscall yüzeyi
8. **Gas + Authority Metering Layer**
   - Compute maliyeti + ayrı authority bütçesi
9. **Governance & Upgrade Layer**
   - Feature gates, deprecation window, migration/activation politikaları

---

## 5) Kanonik execution akışı

1. **Admission**: tx envelope temel kabul kontrolleri.
2. **Auth verify**: scheme/policy/nonce/expiry/capability scope doğrulaması.
3. **Target resolution**: package/entrypoint ve uyumluluk kontrolü.
4. **Bytecode verify/cache**: doğrulanmamış kod canonical execution’a giremez.
5. **Execution**: bounded VM üzerinde çalışma, overlay/journal biriktirme.
6. **Policy/capability re-check**: üretilen mutation diff’inin ikinci kontrolü.
7. **Finalize/commit**: receipt, diff, commitment ve post-state çıktısı.

---

## 6) Dizin haritası (okuyucu için hızlı rehber)

- `src/auth/`: imza şemaları, envelope, doğrulama politikaları
- `src/tx/`: tx envelope, admission, validation
- `src/vm/`: machine state, transition, trap/halt yüzeyleri
- `src/engine/`: lifecycle, execute, dry-run, rollback/finalization
- `src/object/`: typed object modeli ve sınıflar
- `src/policy/`: authorization/execution/governance policy kuralları
- `src/state/`: overlay/diff/checkpoint yüzeyleri
- `src/syscall/` + `src/host/`: host boundary ve syscall yönetimi
- `docs/`: mimari ve operasyonel teknik dokümantasyon
- `schemas/`: tx/object/governance şema yüzeyleri
- `audit/`: güvenlik inceleme kapsamı ve release gate materyalleri

---

## 7) Proje durumu

Bu crate aktif olarak **vNext inşa aşamasındadır**.

- Modül sınırları ve mimari kontratlar tanımlanmıştır.
- Çekirdek admission/auth/policy/execution akış primitifleri mevcuttur.
- Tam opcode/bytecode/host entegrasyon implementasyonu aşamalı ilerletilecektir.

Detaylı teslim planı için: `ROADMAP.md`.
Uygulama doğrulama kontrol listesi için: `CHECKLIST.md`.

---

## 8) Tasarım ilkesi (slogan)

> **Code may execute, but authority must be proven.**
