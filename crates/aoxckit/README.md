# aoxckit

## Purpose

`aoxckit` (Keyforge) is the production-grade **Cryptographic Toolkit and Command-Line Interface (CLI)** for the AOXChain workspace. 

Designed to be executed in strictly offline, air-gapped environments, this tool serves as the sovereign entry point for node operators and validators. It handles all pre-network cryptographic operations including post-quantum key generation, sovereign identity issuance, threshold quorums, and Zero-Knowledge Proof (ZKP) parameter setups. It intentionally contains **zero network code** to guarantee that private key material is never accidentally transmitted.

## Core Components

The CLI is structured into deterministic, highly focused subcommands:
- **`cmd_key.rs` & `cmd_keyfile.rs`**: Generates post-quantum (e.g., Dilithium3) cryptographic keypairs and safely exports them into encrypted, deterministically formatted wallet files.
- **`cmd_actor_id.rs`, `cmd_passport.rs`, `cmd_cert.rs`**: Manages the creation of Sovereign Identities. It binds raw public keys to network-recognized Actor IDs, Passports, and Certificates required for governance and validator registration.
- **`cmd_zkp_setup.rs` & `cmd_quorum.rs`**: Handles advanced cryptographic ceremonies, including generating ZKP public parameters/proofs and configuring threshold multi-signature quorums.
- **`cmd_registry.rs` & `cmd_revoke.rs`**: Generates the exact payload structures required to register a new identity on-chain or revoke a compromised one.

## Code Scope

- `src/main.rs` - The CLI entry point and argument router (powered by `clap`).
- `src/keyforge/` - The isolated command handlers enforcing strict output formatting.

## Security & Operational Notes

- **Air-Gapped Execution**: This tool must never attempt to make HTTP, RPC, or P2P connections. It operates entirely on local inputs and outputs.
- **JSON-First Determinism**: To ensure machine-readability and prevent parsing errors in automated deployment pipelines, all successful command outputs must be deterministically formatted JSON.
- **Secret Wiping**: Temporary memory allocations holding secret keys or mnemonic phrases during generation must be explicitly zeroed out (via `aoxchal` or similar secure memory primitives) before the process exits.
- **No Silent Failures**: If key generation, file writing, or parameter validation fails, the CLI must exit with a non-zero status code and print a deterministic error to `stderr`.

## Local Validation

Because this crate interacts with the command line and generates highly sensitive outputs, it requires both strict unit tests and end-to-end CLI integration tests (located in the `tests/` directory):

```bash
cargo fmt --all -- --check
cargo check -p aoxckit
cargo clippy -p aoxckit --all-targets --all-features -- -D warnings
cargo test -p aoxckit -- --nocapture
Related Components
Top-level architecture: ../../README.md

Core cryptographic primitives: ../aoxcore/README.md

Hardware Abstraction & Secure Memory: ../aoxchal/README.md
