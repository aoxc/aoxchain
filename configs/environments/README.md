# README.md

> Scope: `configs/environments`

## Purpose
Collects environment directories such as mainnet, testnet, devnet, and localnet.

## Contents at a glance
- The code and files in this directory define the runtime behavior of this scope.
- The folder contains modules and supporting assets bounded by this responsibility domain.
- Any change should be evaluated together with its testing and compatibility impact.

## Topology Baseline Files

Each environment now carries a `topology/` subdirectory with:

- `role-topology.toml`
- `socket-matrix.toml`
- `consensus-policy.toml`
- `aoxcq-consensus.toml`

These files define environment-scoped activation and policy overlays for the shared AOXC-Q topology model.
