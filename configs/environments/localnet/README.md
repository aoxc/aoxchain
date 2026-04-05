# Localnet Environment Guide

> Scope: `configs/environments/localnet`

## Purpose

Defines the deterministic multi-node local development fixture used for operator workflows, bootstrap rehearsal, and integration checks.

## Identity and Policy Position

`localnet` is a governed environment class with canonical identity metadata in:

- `configs/registry/network-registry.toml`
- `configs/environments/localnet/release-policy.toml`
- `configs/environments/localnet/profile.toml`

This identity tuple (`chain_id`, `network_id`, `network_serial`) is policy authority.

## Fixture Legacy Note

Some local fixture assets still use legacy chain label strings and historical node labels for compatibility with existing scripts and test fixtures.

- Treat legacy fixture labels as **non-authoritative**.
- For new assets and new runbooks, use canonical identity tuple values and role-first node naming.

## Node Naming Baseline

Preferred naming pattern:

```text
localnet-<role>-<ordinal>
```

Example set:

- `localnet-validator-01`
- `localnet-validator-02`
- `localnet-rpc-01`

## Change Discipline

When modifying localnet assets:

1. keep release-policy, profile, and genesis identity fields synchronized;
2. avoid introducing new legacy identity formats;
3. document compatibility impact if fixture aliases are removed or renamed.
