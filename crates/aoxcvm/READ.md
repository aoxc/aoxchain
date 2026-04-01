# READ.md

> Scope: `crates/aoxcvm`
> System role: AOXChain execution/interoperability kernel surface

## Core identity
This crate is the **AOXCLang Kernel** candidate for AOXChain.

Its role is to provide a deterministic, language-first kernel where interoperability policy is keyed by language/runtime family and enforced uniformly at scheduling and settlement boundaries.

## What makes this non-classic
Traditional bridge stacks are chain-centric. AOXCLang Kernel is:
- language-centric,
- proof-gated,
- replay-safe by design,
- adapter-extensible without changing core deterministic policy.

## Operational model
1. Canonical transaction/intents are routed to a lane.
2. Lane execution follows language policy constraints.
3. Cross-chain settlement is allowed only when finality/proof checks pass.
4. Scheduler rejects replayed or non-finalized relay artifacts.

## Language families (current + target)
- EVM bytecode family,
- Move family,
- WASM contract family,
- UTXO/script validator family,
- future validity-proof execution families (with explicit policy admission).

## Kernel invariants
- deterministic state transition behavior,
- explicit resource and gas accounting,
- domain-separated replay protection,
- auditable evidence for verification and dispute handling.

## Engineering note
The current crate already validates deterministic lane flows in tests, but production relay-grade interoperability still requires broader proof/finality coverage and economic security controls.
