# AOXChain Frontend/Client Entegrasyon Rehberi (Adres + Port + API + RPC)

Bu doküman, **frontend (arayüz)** ve **client** ekiplerinin `aoxcore.com` domain’i altında AOXChain servislerini tam uyumlu şekilde bağlayabilmesi için hazırlanmıştır.

> Not: Depodaki mevcut kod tabanında HTTP route’ları tam bir web framework router’ı olarak tanımlı değil; sağlık (health), metrik ve RPC payload modelleri crate seviyesinde veriliyor. Bu rehberde bunlar için üretime uygun ve tutarlı bir gateway eşleme şeması önerilmiştir.

---

## 1) Canonical Port Haritası (AOXChain standart)

AOXChain ağında kanonik portlar aşağıdaki gibidir:

| Servis | Port | Protokol | Varsayılan bind | Amaç |
|---|---:|---|---|---|
| `rpc_http` | `2626` | TCP | `0.0.0.0` | Genel JSON-RPC / HTTP API |
| `p2p_primary` | `2727` | TCP/QUIC | `0.0.0.0` | Ana P2P transport |
| `p2p_gossip` | `2828` | UDP | `0.0.0.0` | Gossip trafiği |
| `p2p_discovery` | `2929` | UDP | `0.0.0.0` | Peer discovery |
| `rpc_ws` | `3030` | TCP | `0.0.0.0` | Realtime WebSocket |
| `rpc_grpc` | `3131` | TCP | `0.0.0.0` | gRPC API |
| `metrics` | `3232` | TCP | `127.0.0.1` | Prometheus exporter |
| `admin_api` | `3333` | TCP | `127.0.0.1` | Operasyonel admin endpointleri |
| `profiler` | `3434` | TCP | `127.0.0.1` | Profiling/diagnostics |
| `storage_api` | `3535` | TCP | `127.0.0.1` | Storage/index API |
| `live_smoke_test` | `3636` | TCP | `127.0.0.1` | Deterministik smoke test |

---

## 2) RPC crate default bind bilgileri (geliştirme varsayılanı)

`aoxcrpc` crate default değerleri:

- HTTP bind: `127.0.0.1:8080`
- WebSocket bind: `127.0.0.1:8081`
- gRPC bind: `127.0.0.1:50051`
- TLS cert/key: `./tls/server.crt` + `./tls/server.key`
- mTLS CA: `./tls/ca.crt`
- `chain_id`: `AOX-MAIN`
- Rate limit: `600 req/min`

> Üretimde tavsiye: yukarıdaki local default’lar yerine kanonik ağ portları (`2626/3030/3131`) ile tek bir ingress katmanı üzerinden yayın yapın.

---

## 3) aoxcore.com için önerilen domain + prefix planı (frontend tam uyum)

Frontend’in tek bir ana domain üzerinden servisleri tüketebilmesi için önerilen adresleme:

- Public API base: `https://api.aoxcore.com`
- Public WS base: `wss://ws.aoxcore.com`
- Public gRPC host: `grpc.aoxcore.com:443` (TLS, h2)

### 3.1 Prefix standardı

- REST/HTTP yardımcı uçlar: `/api/v1/...`
- JSON-RPC: `/rpc/v1`
- WebSocket stream: `/ws/v1`
- Metrics (public açmayın): `/metrics` (sadece iç ağ/allowlist)

### 3.2 Gateway eşleme (reverse proxy)

- `https://api.aoxcore.com/api/v1/health` -> `127.0.0.1:2626` (health handler)
- `https://api.aoxcore.com/api/v1/metrics` -> `127.0.0.1:3232` veya RPC HTTP metrics surface
- `https://api.aoxcore.com/rpc/v1` -> `127.0.0.1:2626` (JSON-RPC)
- `wss://ws.aoxcore.com/ws/v1` -> `127.0.0.1:3030`
- `grpc.aoxcore.com:443` -> `127.0.0.1:3131`

---

## 4) HTTP payloadları (frontend health + ops ekranı)

### 4.1 Health response şeması

Health response alanları:

- `status`: `ok | degraded | error`
- `chain_id`: string
- `genesis_hash`: string/null
- `tls_enabled`: bool
- `mtls_enabled`: bool
- `tls_cert_sha256`: string/null
- `readiness_score`: `0..100`
- `warnings`: string[]
- `errors`: string[]
- `recommendations`: string[]
- `uptime_secs`: number

Örnek:

```json
{
  "status": "degraded",
  "chain_id": "AOX-MAIN",
  "genesis_hash": null,
  "tls_enabled": false,
  "mtls_enabled": true,
  "tls_cert_sha256": null,
  "readiness_score": 70,
  "warnings": ["genesis_hash is not configured"],
  "errors": [],
  "recommendations": ["Set a canonical 0x-prefixed genesis_hash in RpcConfig and enforce it at node startup"],
  "uptime_secs": 12345
}
```

### 4.2 Prometheus metrics snapshot

Önemli metrik adları:

- `aox_rpc_requests_total`
- `aox_rpc_rejected_total`
- `aox_rpc_rate_limited_total`
- `aox_rpc_rate_limiter_active_keys`
- `aox_rpc_health_readiness_score`

---

## 5) gRPC API (proto sözleşmesi)

Paket: `aoxchain.api.v1`

### 5.1 Servisler

1. `QueryService`
   - `GetChainStatus(ChainStatusRequest) returns (ChainStatusReply)`
   - `GetTxStatus(TxStatusRequest) returns (TxStatusReply)`

2. `TxSubmissionService`
   - `SubmitTx(SubmitTxRequest) returns (SubmitTxReply)`

### 5.2 Mesajlar

- `ChainStatusReply`: `chain_id`, `height`, `syncing`
- `TxStatusRequest`: `tx_id`
- `TxStatusReply`: `tx_id`, `state`
- `SubmitTxRequest`: `actor_id`, `tx_payload(bytes)`, `zkp_proof(bytes)`
- `SubmitTxReply`: `tx_id`, `result`

---

## 6) WebSocket event formatı

Mevcut event tipi:

- `BLOCK_CONFIRMED`

Örnek WS mesajı:

```json
{
  "type": "BLOCK_CONFIRMED",
  "block_hash": "0x...",
  "height": 1024
}
```

---

## 7) JSON-RPC / EVM method desteği (mevcut aşama)

Şu anda destekli Ethereum JSON-RPC methodları:

- `eth_chainId`
- `eth_call`
- `eth_estimateGas`
- `eth_getTransactionReceipt`

Frontend tarafında method whitelist’i buna göre sınırlandırılmalıdır.

---

## 8) Standart hata modeli (UI/SDK için)

RPC hata payload alanları:

- `code`
- `message`
- `retry_after_ms` (rate limit durumunda)
- `request_id`
- `user_hint`

Bilinen hata kodları:

- `INVALID_REQUEST`
- `METHOD_NOT_FOUND`
- `RATE_LIMIT_EXCEEDED`
- `MTLS_AUTH_FAILED`
- `ZKP_VALIDATION_FAILED`
- `INTERNAL_ERROR`

Örnek:

```json
{
  "code": "RATE_LIMIT_EXCEEDED",
  "message": "RATE_LIMIT_EXCEEDED: retry_after_ms=250",
  "retry_after_ms": 250,
  "request_id": "req-42",
  "user_hint": "Apply retry_after_ms with exponential backoff and jitter."
}
```

---

## 9) Frontend bağlantı checklist (tam uyumluluk için)

1. **Base URL sabitleme**
   - REST: `https://api.aoxcore.com/api/v1`
   - JSON-RPC: `https://api.aoxcore.com/rpc/v1`
   - WS: `wss://ws.aoxcore.com/ws/v1`

2. **TLS/mTLS hazır olma**
   - Sunucu cert + key dosyaları yüklü olmalı.
   - B2B/private client’lar için mTLS aktif olmalı.

3. **Rate limit uyumu**
   - 429/limit hatalarında `retry_after_ms` parse edin.
   - Exponential backoff + jitter uygulayın.

4. **ZKP zorunluluğu**
   - Tx submission’da `zkp_proof` boş bırakmayın.

5. **Chain kimliği doğrulama**
   - İlk açılışta health veya chain status’tan `chain_id` doğrulayın (`AOX-MAIN` veya deployment chain id).

---

## 10) Operasyonel komutlar (node/API tarafı doğrulama)

```bash
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo check -p aoxcrpc
```

Bu komutlarla port haritası, runtime durum çıktısı ve RPC crate derleme doğrulaması alınır.
