# AOXCHub Operator Blueprint

## Objective

AOXCHub is the local-only, operator-first control surface for AOXChain lifecycle management. It must remain fail-closed, audit-aware, CLI-backed, and outside-closed by default.

One-line definition:

> AOXCHub is a local-only operator console for chain creation, node lifecycle, genesis management, validator operations, wallet actions, release installation, runtime verification, and audit-aware network control.

## Product Invariants

The following constraints are mandatory for every AOXCHub feature:

- Local-only binding (`127.0.0.1` by default).
- No public administrative exposure.
- Explicit authentication and operator intent confirmation for destructive actions.
- All critical execution delegated to `aoxc` and approved `make` targets.
- Immutable preview-first command model.
- Full audit logging for operator actions and execution outcomes.
- Fail-closed behavior on verification, policy, signature, or compatibility uncertainty.

## Required Modules

### 1. Dashboard / Home

The first surface must provide immediate operational state:

- Chain name, network kind, network id.
- Current height, finalized height, current round.
- Validator count, observer count, connected peers.
- Local node status, RPC status, P2P status.
- Genesis fingerprint and health status.
- Installed versions (`aoxc`, `aoxchub`, runtime).
- Last 10 events, transactions, and warnings.
- Quick actions for high-frequency workflows.

### 2. Binary / Release Manager

Required capability:

- Show installed `aoxc` and `aoxchub` versions.
- Show latest official release and compatible release channel.
- Show checksum status, signature status, install path, and rollback backup.
- Execute official release check, download, verify, install, and rollback.

Mandatory policy:

- Official AOXChain release source only.
- Signed/checksummed assets only.
- No arbitrary URL installation.
- No unsigned installation path.
- No blind “latest” install without compatibility and policy checks.

### 3. Chain Creator

Must support:

- Create New Chain.
- Open Existing Chain.
- Import Chain Bundle.
- Demo Chain.
- Localnet/Testnet guided wizards.

Creation inputs:

- Chain name.
- Network kind (`demo`, `localnet`, `devnet`, `testnet`, `mainnet`).
- Chain id / network id.
- Token symbol, decimals, initial supply.
- Faucet toggle.
- Validator and observer counts.
- PQ mode, governance mode, staking mode, reward policy.
- Genesis timestamp policy.

Expected outputs:

- `manifest.json`.
- `genesis.json`.
- `validators.json`.
- `bootnodes.json`.
- `certificate.json`.
- `profile.toml`.
- `release-policy.toml`.
- Chain fingerprint and install receipt.

### 4. Genesis Studio

Must support:

- Genesis creation and inspection.
- Validator/account/faucet population.
- Token and bootstrap policy edits.
- Stake distribution configuration.
- Genesis verify/sign/fingerprint workflows.

Genesis schema must include:

- Chain metadata.
- Token metadata.
- Initial accounts.
- Validators and bootnodes.
- PQ attestation bindings.
- Quorum/slashing/staking/transfer policies.
- Runtime compatibility markers.

Validator entry requirements:

- Name, identity, consensus key, PQ key, operator key.
- Initial self-bond and commission.
- Role and initial active status.
- Node/P2P address and bootnode flag.

### 5. Node Manager

Must expose:

- Node id, role, P2P/RPC addresses.
- Sync status, peer count.
- Produced/finalized block counts.
- Storage path, runtime version, uptime.
- CPU/RAM/disk health and validator binding state.

Must execute:

- `node init/start/stop/restart/remove/inspect`.
- `node doctor`, logs, sync status, diagnostics export.

Initialization requirements:

- Key/config/db/runtime directory assignment.
- Genesis fingerprint binding.
- Validator and role binding.
- Port assignment with conflict detection.
- Health receipt generation.

### 6. Network Manager

Must include:

- Topology map, connectivity map, peer mesh.
- Validator topology and bootnode inventory.
- RPC endpoints and health map.
- Network doctor and lifecycle controls.

Must execute:

- `network create/start/stop/restart/verify/expand`.
- Node join/remove.
- Topology export.

Operational truth indicators:

- Block production state.
- Finality progression.
- Validator quorum activation.
- Peer convergence.
- Split/fork anomaly detection.

### 7. Wallet & Address Manager

Must support:

- Wallet create/import/export.
- Address list and labels.
- Balance and history views.
- Faucet funding.
- Offline signing.
- Future hardware signer integration.

Address creation is mandatory for a credible live-chain operator experience.

### 8. Validator Manager

Must support:

- Validator create/import/inspect.
- Activate/deactivate lifecycle.
- Jail and unjail workflows.
- Key rotation and PQ key updates.
- Rewards, liveness, and voting power views.

Identity boundary rule:

- Validator identity, node identity, and operator access credentials must remain explicitly separated.

### 9. Staking Panel

Must support:

- Self-bond, delegate, undelegate, rewards, withdraw.
- Pending unbonding state.
- Validator APR/weight/commission visibility.
- Stake distribution analytics.

Must display:

- Effective voting power.
- Self-bonded vs delegated stake.
- Unbonding, slashed total, jailed status.
- Validator eligibility.

### 10. Transaction Center

Must support:

- Transfer and batch transfer.
- Sign/submit/status flows.
- Inclusion and finality waits.
- History and deep inspection.

Transfer form state must include:

- Source, destination, amount, fee, nonce/sequence.
- Signed/submitted/included/finalized state.
- Explicit failure reason when rejected.

### 11. Faucet Manager

For demo/localnet surfaces:

- Faucet account creation.
- Seed/fund operations.
- Rate and quota controls.
- Historical dispense log.

### 12. Runtime / Bundle Manager

Must support:

- Runtime install/verify/activate/reset.
- Runtime fingerprinting.
- Source bundle comparison.
- Profile inspection.

Must display:

- Manifest, genesis hash, validator-set root.
- PQ attestation root.
- Active profile, release policy, runtime marker.

### 13. Audit / Evidence Surface

Must provide:

- Runtime and node logs.
- Consensus warnings and invariant violations.
- Operator action history.
- Release install trace and validator/transfer events.
- Doctor output history.

Must export:

- Audit bundle.
- Incident bundle.
- Evidence archive.
- Per-node diagnostics.

### 14. Doctor / Health Center

Must execute at minimum:

- Genesis drift checks.
- Validator duplication checks.
- PQ readiness checks.
- Key mismatch checks.
- Port conflict checks.
- Peer health and quorum liveness checks.
- Finality lag detection.
- Storage corruption checks.
- Runtime/release mismatch checks.
- Consensus replay integrity checks.

### 15. Embedded Terminal

Terminal is required, with two policy modes:

1. **Safe Terminal**: allowlisted commands for standard operations (`status`, `doctor`, `wallet balance`, `node logs`, `network status`).
2. **Advanced Terminal**: raw CLI access with mandatory preview, explicit confirmation, destructive-op warnings, and audit logging.

## Canonical Execution Model

AOXCHub is an orchestration and visibility layer. It is not the execution authority.

Critical operations must resolve to canonical command paths, for example:

- `aoxc chain ...`
- `aoxc genesis ...`
- `aoxc validator ...`
- `aoxc wallet ...`
- `aoxc tx ...`
- `aoxc stake ...`
- `aoxc node ...`
- `aoxc network ...`

This rule preserves one execution truth across UI and terminal workflows and improves auditability, testability, and policy enforcement.

## End-to-End Operator Flows

### Demo Chain (single action)

1. Verify official binaries and runtime readiness.
2. Create local validator and observer set.
3. Build genesis, validators, and bootnodes artifacts.
4. Install/verify/activate runtime.
5. Start network and run health checks.
6. Seed faucet and bootstrap sample wallets.
7. Execute sample transfer and confirm finality.
8. Render success state in dashboard.

### Guided Localnet/Testnet Build

1. Collect chain and token parameters.
2. Collect validator/observer and staking/faucet options.
3. Select PQ and governance profile.
4. Review generated genesis inputs.
5. Build and verify artifacts.
6. Start network and execute doctor checks.

### Validator Onboarding

1. Generate identity and signing keys.
2. Generate PQ and operator keys.
3. Configure self-bond and node binding.
4. Add to genesis (or post-genesis join flow).
5. Persist audit log and evidence.

### Wallet and Address Onboarding

1. Create/import wallet.
2. Generate address and label metadata.
3. Store keystore locally.
4. Seed via faucet and verify balance.

### Stake and Transfer

Stake:

1. Select wallet, validator, amount.
2. Review, sign, submit.
3. Wait for inclusion/finality.
4. Refresh stake state and validator power.

Transfer:

1. Select source/destination/amount.
2. Review fees and sequence.
3. Sign and submit.
4. Wait for inclusion/finality.
5. Confirm updated balances.

## Security Non-Negotiables

Required:

- Local-only network bind.
- Authenticated operator session.
- Command preview before execution.
- Destructive action confirmation.
- Audit logging and evidence export.
- Binary verification and signature policy enforcement.
- No silent fallback when verification fails.
- No private key leakage in UI, logs, or API output.
- No bypass path around `aoxc` validation logic.

Prohibited:

- Remote public admin panel.
- Anonymous administrative access.
- Raw database mutation controls.
- Unsafe force-finalize controls.
- “Ignore validation” bypass switches.
- Unsigned binary install path.
- Genesis reset without explicit confirmation and evidence logging.
- Silent wallet export.

## Minimum Artifact Set for “Real Chain” Claims

AOXCHub must not represent a chain as operational unless the following exist and verify:

- Genesis artifact.
- Manifest.
- Validators and bootnodes configuration.
- Profile and release policy.
- Runtime activation marker.
- Node identities with P2P and RPC addressing.
- Persistent state directory.
- Health and doctor outputs with actionable status.

## Module Tree Reference

```text
AOXCHub
├── Dashboard
├── Releases
│   ├── Check
│   ├── Verify
│   ├── Install
│   └── Rollback
├── Chain
│   ├── Demo
│   ├── Create
│   ├── Open
│   ├── Export
│   └── Reset
├── Genesis
│   ├── Inspect
│   ├── Validators
│   ├── Accounts
│   ├── Token
│   ├── Policies
│   ├── Build
│   ├── Verify
│   └── Sign
├── Nodes
│   ├── List
│   ├── Init
│   ├── Start
│   ├── Stop
│   ├── Restart
│   ├── Logs
│   └── Doctor
├── Network
│   ├── Topology
│   ├── Start
│   ├── Stop
│   ├── Status
│   ├── Verify
│   └── Expand
├── Validators
│   ├── Create
│   ├── Inspect
│   ├── Bond
│   ├── Rotate Keys
│   ├── Set PQ
│   ├── Rewards
│   └── Lifecycle
├── Wallets
│   ├── Create
│   ├── Import
│   ├── Balance
│   ├── Fund
│   ├── Export
│   └── History
├── Transactions
│   ├── Transfer
│   ├── Sign
│   ├── Submit
│   ├── Wait
│   └── History
├── Staking
│   ├── Validators
│   ├── Delegate
│   ├── Undelegate
│   ├── Rewards
│   └── Withdraw
├── Runtime
│   ├── Install
│   ├── Verify
│   ├── Activate
│   ├── Status
│   └── Fingerprint
├── Audit
│   ├── Logs
│   ├── Evidence
│   ├── Export
│   └── Incidents
├── Doctor
└── Terminal
```
