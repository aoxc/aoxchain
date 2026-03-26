# AOXC Sovereign Environments

This directory contains sovereign AOXC private network materials.

At the current baseline, the directory provides a canonical template bundle under:

- `template/`

## Purpose

The sovereign template exists to support future AOXC private Layer 1 deployments while preserving the same high-level operating model used across the AOXC family:

- single binary,
- environment-derived identity,
- manifest-governed bundle structure,
- registry-controlled naming and compatibility.

## Template Rule

The template is not a deployable final network identity.

It is a controlled starting point for future sovereign deployments.

Template identity must not be copied into a real deployment unchanged.

A real sovereign network must generate and document its own:
- `network_serial`
- `chain_id`
- `network_id`
- chain display name
- validator set
- bootnode set
- certificate material

## Governance Rule

Future sovereign networks must remain consistent with:
- `configs/registry/network-registry.toml`
- `configs/registry/binary-compatibility.toml`

Any officially promoted sovereign deployment should receive its own environment directory rather than reusing the template directory as-is.
