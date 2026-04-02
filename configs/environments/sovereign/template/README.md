# README.md

> Scope: `configs/environments/sovereign/template`

## Purpose
Contains sovereign network bootstrap templates, including a canonical baseline genesis and an advanced security-oriented genesis example.

## Template Files
- `genesis.v1.json`: baseline sovereign template.
- `genesis.advanced.example.json`: advanced testnet-style example with explicit governance treasury and validator-account threshold.
- `validators.json`, `bootnodes.json`, `certificate.json`: binding references expected by genesis.

## Operator Workflow
For custom genesis authoring and enforcement:

```bash
aoxc genesis-template-advanced --profile testnet --out ./genesis.testnet.advanced.example.json
aoxc genesis-security-audit --profile testnet --genesis ./genesis.testnet.advanced.example.json --enforce
aoxc consensus-profile-audit --profile testnet --genesis ./genesis.testnet.advanced.example.json --strict
```

## Change Discipline
Any modification to template genesis surfaces must keep identity, consensus, bindings, and deterministic-integrity controls internally coherent.
