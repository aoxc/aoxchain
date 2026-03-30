# Architecture

## Main Components
`aoxcsdk` is organized around focused modules under `src/`, with responsibilities separated by domain concern.

## Data Flow
Inputs enter through public APIs, are validated and transformed through internal modules, and produce deterministic outputs consumed by upstream crates or services.

## Dependencies
This crate should depend on lower-level shared primitives where possible and expose stable interfaces to higher-level crates.

## Boundaries
Security-sensitive operations, deterministic state transitions, and external I/O boundaries must remain explicit and minimal.
