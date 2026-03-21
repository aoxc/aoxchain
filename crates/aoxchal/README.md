# aoxchal

## Purpose

`aoxchal` provides the **Hardware Abstraction Layer (HAL) and Resource Optimization** domain within the AOXChain workspace. It ensures that the sovereign core can safely, securely, and deterministically leverage bare-metal hardware capabilities—without compromising cross-platform portability or consensus determinism.

By abstracting underlying hardware resources, this crate allows high-performance components (like virtual machines and cryptographic verifiers) to utilize advanced CPU instructions and managed memory pools safely.

## Core Components

- **`cpu_opt.rs`**: Manages runtime CPU feature detection (e.g., AVX-512, NEON) to route cryptographic algorithms and hashing functions to their hardware-accelerated paths. It also handles thread-pinning and compute-bound workload distribution.
- **`mem_manager.rs`**: Implements secure memory pooling, zero-copy buffer abstractions, and deterministic memory limit enforcement. It ensures high-throughput state transitions do not cause memory fragmentation or Out-of-Memory (OOM) attack vectors.

## Code Scope

- `src/lib.rs` - Main entry point and feature gates.
- `src/cpu_opt.rs` - Processor optimizations and compute dispatching.
- `src/mem_manager.rs` - Safe memory allocation, wiping, and bounding.

## Security & Operational Notes

- **Software Fallbacks**: Every hardware-accelerated path in `cpu_opt.rs` **must** have a deterministic, fallback software implementation. If a specific CPU feature is missing, the node must continue to operate deterministically.
- **Constant-Time & Memory Wiping**: Any memory managed by `mem_manager.rs` that touches private key material or ZKP proofs must be explicitly zeroed out (wiped) upon deallocation to prevent cold-boot and side-channel attacks.
- **`unsafe` Code Boundaries**: Usage of `unsafe` Rust for hardware intrinsics or memory manipulation is strictly isolated to this crate. All `unsafe` blocks must be heavily documented, strictly bounded, and subjected to rigorous security audits.
- **Determinism Over Speed**: Hardware optimizations must never alter the mathematical outcome of an operation. A block validated on an ARM CPU using NEON must yield the exact same state root as an x86 CPU using AVX.

## Local Validation

Before submitting changes to the HAL, ensure hardware-agnostic tests and static analysis pass flawlessly:

```bash
cargo check -p aoxchal
cargo clippy -p aoxchal --all-targets --all-features -- -D warnings
cargo test -p aoxchal
Related Components
Top-level architecture: ../../README.md

Core cryptographic primitives: ../aoxcore/README.md

Virtual Machine execution lanes: ../aoxcvm/README.md
