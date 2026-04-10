# README.md

> Scope: `crates/aoxchal`

## Purpose
Contains hardware/infrastructure abstraction and deployment-optimization components.

This crate also provides deterministic crypto-profile selection helpers so
upstream services can move from classical to hybrid and then PQ-primary
verification modes without non-deterministic runtime branching.

## Contents at a glance
- The code and files in this directory define the runtime behavior of this scope.
- The folder contains modules and supporting assets bounded by this responsibility domain.
- Any change should be evaluated together with its testing and compatibility impact.
- `crypto_profile` centralizes migration-stage policy into explicit enums and
  deterministic runtime selection logic.
