# aoxcmob

## Purpose

`aoxcmob` is the **native mobile secure-connection core** for AOXChain.

This crate is intentionally scoped to the responsibilities that belong on a
mobile or portable client:

- device-bound key provisioning
- signed session handshake
- replay-resistant request envelopes
- light chain health reads
- task / witness receipt signing
- transport abstraction for relay or RPC integration

This crate is **not** a validator, consensus engine, or full node runtime.
Those responsibilities remain outside the mobile trust boundary.

## Design Goals

1. **Device-bound trust**  
   Private signing material stays within the device security boundary.
2. **Transport independence**  
   Mobile security logic must not depend on a single relay implementation.
3. **Deterministic auditability**  
   Session and task signatures must be reproducible from canonical payloads.
4. **Operational safety**  
   The crate must fail closed for missing device state, invalid challenge data,
   or expired session conditions.

## Main Components

- `config`: runtime policy and timeout model
- `types`: public mobile-facing domain types
- `security`: device provisioning and signing helpers
- `session`: challenge / permit protocol objects
- `transport`: relay or RPC abstraction and local mock transport
- `gateway`: high-level secure native gateway for mobile flows

## Integration Rule

The recommended integration order is:

1. provision device key
2. bind public identity to the device off-chain or on-chain
3. open signed session
4. fetch lightweight tasks or chain state
5. sign task receipts or governance witness actions
6. add contract adapters only after the native session boundary is stable

## Local Validation

```bash
cargo check -p aoxcmob
cargo test -p aoxcmob
```
