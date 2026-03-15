# AOXChain Crate Index

This index maps workspace crates to their responsibilities and primary entry points.

## 1) Protocol Core

- [`aoxcore`](aoxcore/README.md): identity, genesis, transaction, mempool, and foundational state primitives.
- [`aoxcunity`](aoxcunity/README.md): consensus kernel (quorum, voting, proposer rotation, fork-choice, sealing).
- [`aoxcvm`](aoxcvm/README.md): multi-lane execution compatibility layer (EVM/WASM/Sui/Cardano).

## 2) Networking and API

- [`aoxcnet`](aoxcnet/README.md): peer/session/gossip networking and security policies.
- [`aoxcrpc`](aoxcrpc/README.md): HTTP/gRPC/WebSocket ingress surfaces.
- [`aoxcsdk`](aoxcsdk/README.md): SDK integration surface for downstream developers.

## 3) Operations and Tooling

- [`aoxcmd`](aoxcmd/README.md): node bootstrap, smoke operations, economy, and compatibility commands.
- [`aoxckit`](aoxckit/README.md): keyforge, certificate, and identity operational tooling.
- [`aoxconfig`](aoxconfig/README.md): configuration schema and loading components.

## 4) Supporting Modules
Bu dizin, AOXChain workspace içindeki crate'lerin sorumluluk haritasını ve başlangıç noktalarını içerir.

## 1) Protokol Çekirdeği

- [`aoxcore`](aoxcore/README.md): identity, genesis, transaction, mempool ve temel state primitifleri.
- [`aoxcunity`](aoxcunity/README.md): consensus kernel (quorum, vote, proposer, fork-choice, seal).
- [`aoxcvm`](aoxcvm/README.md): çoklu execution lane uyumluluk katmanı (EVM/WASM/Sui/Cardano).

## 2) Ağ ve API Katmanı

- [`aoxcnet`](aoxcnet/README.md): peer/sesssion/gossip ağı ve güvenlik politikaları.
- [`aoxcrpc`](aoxcrpc/README.md): HTTP/gRPC/WebSocket RPC giriş yüzeyleri.
- [`aoxcsdk`](aoxcsdk/README.md): dış geliştiriciler için SDK entegrasyon yüzeyi.

## 3) Operasyon ve Araçlar

- [`aoxcmd`](aoxcmd/README.md): node bootstrap, smoke, ekonomi ve uyumluluk komutları.
- [`aoxckit`](aoxckit/README.md): keyforge, sertifika ve kimlik operasyon araçları.
- [`aoxconfig`](aoxconfig/README.md): konfigürasyon şemaları ve yükleme bileşenleri.

## 4) Yardımcı Modüller

- [`aoxcai`](aoxcai/README.md)
- [`aoxcdata`](aoxcdata/README.md)
- [`aoxcexec`](aoxcexec/README.md)
- [`aoxcenergy`](aoxcenergy/README.md)
- [`aoxclibs`](aoxclibs/README.md)
- [`aoxcmob`](aoxcmob/README.md)
- [`aoxcontract`](aoxcontract/README.md)
- [`aoxchal`](aoxchal/README.md)

## 5) Paired Documentation

- Main project overview: [`../README.md`](../README.md)
- Audit/operations guide: [`../docs/AUDIT_READINESS_AND_OPERATIONS.md`](../docs/AUDIT_READINESS_AND_OPERATIONS.md)
- Risk notice: [`../docs/SECURITY_AND_RISK_NOTICE_TR.md`](../docs/SECURITY_AND_RISK_NOTICE_TR.md)

> Note: Crate boundaries may evolve over time. Critical API changes should be documented and accompanied by tests.
## 5) Dökümantasyonla birlikte kullanım

- Ana proje anlatımı: [`../README.md`](../README.md)
- Audit/operasyon rehberi: [`../docs/AUDIT_READINESS_AND_OPERATIONS.md`](../docs/AUDIT_READINESS_AND_OPERATIONS.md)
- Risk bildirimi: [`../docs/SECURITY_AND_RISK_NOTICE_TR.md`](../docs/SECURITY_AND_RISK_NOTICE_TR.md)

> Not: Crate sınırları zaman içinde evrilebilir; kritik API değişiklikleri ilgili README + testler ile birlikte güncellenmelidir.
