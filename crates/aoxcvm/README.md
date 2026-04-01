# README.md

> Scope: `crates/aoxcvm`

## AOXCLang Kernel (proposed core name)
`aoxcvm` is positioned as the **AOXCLang Kernel**: a language-first execution and interoperability core for AOXChain.

The design goal is to move from chain-specific bridge logic to a **language-centric deterministic kernel** where:
- execution policy is defined by language family,
- chain differences are isolated in adapter/finality layers,
- cross-chain settlement requires explicit proof and replay safety.

## Why language-first
A chain-first model scales poorly as integrations grow. AOXCLang Kernel instead models interoperability by runtime language semantics:
- EVM bytecode family,
- Move family,
- WASM contract family,
- UTXO validator/script family.

This gives one canonical kernel policy surface that can be reused across many chains implementing similar language semantics.

## Current implementation status (April 1, 2026)
The runtime currently provides a deterministic multi-lane execution skeleton with passing integration tests for:
- EVM deploy + call,
- Sui/Move publish + object create,
- WASM upload + instantiate + execute,
- Cardano UTXO create + spend,
- dispatcher-level routing and gas accounting checks.

This is not yet production-final relay infrastructure.

## AOXCVM-NEXT bootstrap track (April 1, 2026)
The crate now includes a `nextvm` bootstrap module for a ground-up VM path with:
- deterministic instruction execution and explicit gas accounting,
- capability-gated state and host actions,
- crypto-profile validation hooks with post-quantum hybrid requirements,
- transactional checkpoints (`checkpoint/commit/rollback`) and deterministic execution traces.

This surface is intentionally minimal and is intended to be extended through formal execution-spec milestones.

## AOXCLang Kernel architecture (target)

### 1) Language Policy Layer (kernel-native)
Defines deterministic behavior by language family:
- canonical ABI envelope,
- state-model expectations,
- deterministic execution constraints,
- required verification class before cross-chain relay settlement.

### 2) Lane Runtime Layer (execution engines)
Implements transaction execution per lane while conforming to language policy:
- EVM lane,
- Move lane,
- WASM lane,
- UTXO/script lane.

### 3) Finality and Proof Layer (chain adapters)
Per-chain adapters must provide:
- finalized state transition evidence,
- reorg-aware validity windows,
- domain-separated replay protection data.

### 4) Relay Settlement Layer (kernel scheduler)
Consumes canonical intents and proof artifacts:
- no proof => no settlement,
- non-final state => no settlement,
- replayed message ID => reject.

## Language expansion roadmap (full spectrum)
AOXCLang Kernel should progressively support or map to:
- EVM/Solidity/Vyper ecosystems,
- Move ecosystems,
- WASM smart-contract ecosystems,
- UTXO script ecosystems,
- ZK validity-proof backed rollup execution surfaces,
- chain-native DSL adapters where deterministic constraints can be formalized.

New language families should only be admitted when deterministic replay, proof verification, and backward-compatible ABI normalization are defined.

## Security and correctness requirements
A lane or chain integration is not relay-grade unless all conditions are met:
1. deterministic execution profile is documented,
2. proof verification path is implemented,
3. replay protection is enforced,
4. failure and dispute evidence are persisted,
5. compatibility and migration boundaries are explicitly versioned.

## Current gaps
AOXCLang direction is compatible with full interoperability, but still lacks:
- production-grade light clients for all major chains,
- complete consensus-family finality verification matrix,
- slashing/incentive economics for delegated attestations,
- large-scale adversarial interoperability testnets.

## Contents at a glance
- The code and files in this directory define runtime behavior for this scope.
- Any change should be reviewed for deterministic behavior and compatibility impact.
