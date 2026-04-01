# AOXCVM Delivery Checklist

Bu kontrol listesi, AOXCVM değişikliklerinin mimari hedefe ve üretim güvenlik standartlarına uyumunu doğrulamak için kullanılır.

## A) Mimari Uyum
- [ ] Değişiklik AOXCVM’in tek kanonik execution modeliyle uyumlu.
- [ ] State değişimi object-centric model dışında eklenmiyor.
- [ ] Yetki denetimi capability/policy katmanını bypass etmiyor.
- [ ] Determinism-first yaklaşımı korunuyor.

## B) Authorization ve Admission
- [ ] Tx admission kontrolleri (domain, expiry, budget, target) korunuyor.
- [ ] Auth envelope doğrulaması (nonce/replay/expiry/scheme) güncel.
- [ ] PQ/hybrid policy beklentileri açıkça tanımlı.
- [ ] Hata kodları `errors.rs` üzerinden kanonik biçimde dönüyor.

## C) Execution ve State
- [ ] Execution overlay/journal üstünde çalışıyor, doğrudan commit yapılmıyor.
- [ ] Gas ve authority bütçe kontrolleri atlanmıyor.
- [ ] Object mutation sonrası policy/capability re-check yapılıyor.
- [ ] Finalization çıktıları (diff/receipt/outcome) tutarlı.

## D) Host / Syscall Boundary
- [ ] Yeni host erişimi varsa registry/policy ile sınırlanmış.
- [ ] Nondeterministic kaynaklar (clock/random/network) canonical yola girmiyor.
- [ ] Syscall değişikliği versioning ve compatibility etkisiyle dokümante.

## E) Governance / Upgrade
- [ ] Feature gate etkisi varsa açıkça belirtilmiş.
- [ ] Protocol/version compatibility etkisi değerlendirilmiş.
- [ ] Migration/deprecation etkisi varsa plan ve geri dönüş yöntemi yazılmış.

## F) Test / Doğrulama
- [ ] Unit veya integration test ile davranış doğrulanmış.
- [ ] En az `cargo test -p aoxcvm` çalıştırılmış.
- [ ] Hata senaryoları (replay, policy violation, budget exceed vb.) test edilmiş.

## G) Dokümantasyon
- [ ] README değişiklikleri mimariyi doğru yansıtıyor.
- [ ] ROADMAP etkileniyorsa faz güncellemesi yapıldı.
- [ ] Gerekli `docs/` dosyaları güncellendi.
- [ ] Güvenlik/audit etkileri ilgili dokümana işlendi.
