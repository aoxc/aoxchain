# AOXCHub Full Product Specification

## 1) Aoxchub tam olarak ne olmalı?

### Tanım

AOXCHub, AOXChain için aşağıdaki zorunlu nitelikleri taşıyan bir operator console olmalıdır:

- local-only
- outside-closed
- operator-first
- audit-aware
- CLI-backed
- fail-closed
- node and network management console

### One-line definition

**AOXCHub is a local-only operator console for chain creation, node lifecycle, genesis management, validator operations, wallet actions, release installation, runtime verification, and audit-aware network control.**

## 2) Aoxchub içinde neler kesin olmalı?

### A. Dashboard / Home

Must show:

- chain name
- network kind
- network id
- current height
- finalized height
- current round
- validator count
- observer count
- connected peers
- local node status
- RPC status
- P2P status
- genesis fingerprint
- health status
- installed versions (`aoxc`, `aoxchub`, runtime)
- last 10 events
- last 10 tx
- last 10 warnings
- quick actions

### B. Binary / Release Manager

Must show:

- current installed `aoxc` version
- current installed `aoxchub` version
- latest official release
- compatible release channel
- checksum status
- signature status
- install path
- previous version backup

Must do:

- official release check
- download official release
- verify release
- install release
- rollback previous release

Mandatory controls:

- official GitHub release source only
- signed/checksummed assets only
- no blind latest pull
- no arbitrary URL
- no unsigned install

### C. Chain Creator

Must include screens:

- Create New Chain
- Open Existing Chain
- Import Chain Bundle
- Demo Chain
- Localnet Wizard
- Testnet Wizard

Create flow inputs:

- chain name
- network kind: `demo | localnet | devnet | testnet | mainnet`
- chain id / network id
- token symbol
- token decimals
- initial supply
- faucet enabled
- validator count
- observer count
- PQ policy mode
- governance mode
- staking enabled
- reward policy
- genesis timestamp policy

Must produce:

- `manifest.json`
- `genesis.json`
- `validators.json`
- `bootnodes.json`
- `certificate.json`
- `profile.toml`
- `release-policy.toml`
- chain fingerprint
- install receipt

### D. Genesis Studio

Must do:

- genesis create / inspect
- validator add
- account add
- faucet create
- token parameter set
- bootstrap policy set
- stake distribution set
- genesis verify
- genesis sign
- genesis fingerprint show

Genesis required fields:

- chain metadata
- token metadata
- initial accounts
- validators
- bootnodes
- PQ attestation bindings
- quorum policy
- slashing policy
- staking parameters
- transfer policy
- runtime compatibility markers

Validator form required fields:

- validator name
- validator identity
- consensus key
- PQ key
- operator key
- initial self-bond
- commission
- role
- active/inactive initial status
- node address / P2P address
- bootnode flag

### E. Node Manager

Must show:

- node list
- node id
- node role
- P2P address
- RPC address
- sync status
- peer count
- produced blocks
- finalized blocks
- storage path
- runtime version
- uptime
- CPU/RAM/disk health
- validator binding status

Must do:

- node init
- node start
- node stop
- node restart
- node remove
- node inspect
- node doctor
- node logs
- node sync status
- node export diagnostics

Node init must include:

- key dir
- config dir
- db dir
- runtime dir
- genesis fingerprint bind
- validator binding
- role binding
- P2P/RPC port assignment
- health receipt

### F. Network Manager

Must include:

- topology map
- node connectivity
- peer mesh
- validator topology
- bootnodes
- RPC endpoints
- health map
- network doctor
- network start/stop/restart

Must do:

- network create
- network start
- network stop
- network restart
- network verify
- network expand
- node join
- node remove
- topology export

Must expose real-chain indicators:

- block production live
- finality live
- validator quorum active
- peer convergence
- chain split detection
- fork anomaly detection

### G. Wallet & Address Manager

Must include:

- wallet create/import/export
- address list
- balance
- tx history
- label/address book
- faucet fund
- offline sign
- hardware signer support (future)

Address creation is mandatory for a credible live-chain experience.

### H. Validator Manager

Must include:

- validator create/import/inspect
- validator activate/deactivate
- validator jail view/unjail
- validator rotate key
- validator set PQ
- validator rewards
- validator liveness
- validator voting power

Validator creation flow must include:

- validator identity creation
- consensus signing key
- PQ signing key
- operator key
- attestation commitment
- node binding
- initial self-bond
- genesis inclusion or post-genesis onboarding

Critical boundary:

- validator identity, node identity, and operator access must remain separated.

### I. Staking Panel

Must do:

- self-bond
- delegate
- undelegate
- withdraw
- rewards
- pending unbonding
- validator APR/weight/commission
- stake distribution view

Must show:

- effective voting power
- self bonded
- delegated
- unbonding
- slashed total
- jailed status
- validator eligibility

### J. Transfer / Transaction Center

Must do:

- transfer
- batch transfer
- tx sign
- tx submit
- tx status
- wait for inclusion
- wait for finality
- tx history
- tx inspect

Transfer screen must show:

- from
- to
- amount
- fee
- nonce/sequence
- sign
- submit
- included
- finalized
- failure reason

### K. Faucet Manager

Required for demo/localnet:

- faucet account create
- faucet seed
- faucet send
- faucet limits
- faucet history

### L. Runtime / Bundle Manager

Must do:

- runtime install
- runtime verify
- runtime activate
- runtime reset
- runtime fingerprint
- runtime source bundle compare
- profile inspect

Must show:

- manifest
- genesis hash
- validator set root
- PQ attestation root
- profile
- release policy
- active runtime marker

### M. Logs / Audit / Evidence

Must show:

- runtime logs
- node logs
- consensus warnings
- invariant violations
- operator actions
- release installation logs
- validator events
- transfer events
- doctor results

Must do:

- export audit bundle
- export incident bundle
- evidence archive
- per-node diagnostics

### N. Doctor / Health Center

Must run checks:

- genesis drift
- validator duplication
- PQ readiness
- key mismatch
- port conflict
- peer health
- quorum liveness
- finality lag
- storage corruption
- runtime mismatch
- release mismatch
- consensus replay integrity

### O. Embedded Terminal

Terminal is required.

Modes:

1. **Safe Terminal** (allowlist):
   - `aoxc status`
   - `aoxc doctor`
   - `aoxc wallet balance`
   - `aoxc node logs`
   - `aoxc network status`
2. **Advanced Terminal**:
   - raw CLI commands
   - command preview
   - explicit confirmation
   - audit log
   - destructive operation warning

## 3) Tam kullanıcı akışları

### Flow 1 — Demo Chain

Single action: **Create Demo Chain**.

Backend flow:

1. official binary verification
2. runtime readiness check
3. local validator set creation (4 validators + observer)
4. genesis / validators / bootnodes generation
5. runtime install/verify/activate
6. network start
7. health check
8. faucet seed
9. Alice/Bob wallets
10. sample transfer and finality check
11. success dashboard

### Flow 2 — Real Localnet Wizard

1. chain name
2. token settings
3. validator count
4. observer count
5. staking enabled
6. faucet enabled
7. PQ mode
8. genesis review
9. build
10. verify
11. start network

### Flow 3 — Validator Creation

1. validator name
2. identity generate
3. consensus key generate
4. PQ key generate
5. operator key generate
6. self-bond set
7. node bind
8. genesis include or post-genesis join
9. confirm
10. audit log write

### Flow 4 — Address Creation

1. wallet name
2. mnemonic generate
3. address generate
4. label set
5. local keystore save
6. faucet fund

### Flow 5 — Stake

1. wallet select
2. validator select
3. amount enter
4. review
5. sign
6. submit
7. wait inclusion/finality
8. updated stake view

### Flow 6 — Transfer

1. from select
2. to select
3. amount enter
4. fee show
5. sign
6. submit
7. included
8. finalized
9. balances update

## 4) Gerçek zincir için zorunlu varlıklar

Minimum required:

- genesis
- manifest
- validators
- bootnodes
- profile
- release policy
- runtime activation
- node identities
- P2P addresses
- RPC addresses
- persistent state
- health checks

Address creation, validator creation, and node creation are all mandatory for real-chain claims.

For stake-based chains, staking is mandatory.
For living-chain UX, transfers are mandatory.

## 5) CLI ve Hub ilişkisi

Hard rule:

- `aoxchub` = UI/operator console
- `aoxc` = execution authority
- node/runtime = chain engine

AOXCHub must not become an alternate execution truth.

Critical operations must resolve to canonical command families:

- `aoxc chain ...`
- `aoxc genesis ...`
- `aoxc validator ...`
- `aoxc wallet ...`
- `aoxc tx ...`
- `aoxc stake ...`
- `aoxc node ...`
- `aoxc network ...`

## 6) Güvenlik şartları

Required:

- local-only bind
- outside closed
- auth required
- command preview
- destructive confirmation
- audit logging
- binary verification
- no silent fallback
- no private key leakage
- no bypass of CLI/kernel validation
- no public admin exposure

Prohibited:

- remote web admin
- anonymous access
- raw DB edit
- force-finalize control
- ignore-validation bypass
- unsigned binary install
- genesis reset without confirmation
- silent wallet export

## 7) Module tree

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

## 8) Final decisions

- Embedded terminal: **required**
- Address creation: **required**
- Validator creation: **required**
- Node creation: **required**
- Genesis creation: **required**
- Stake and transfer: **required for real-chain operations**
- Binary source: **verified official GitHub release only**
- Public admin panel: **prohibited**

## Relationship to crate-level blueprint

`crates/aoxchub/OPERATOR_BLUEPRINT.md` remains the crate-local implementation-oriented blueprint.
This document is the repository-level full product specification surface for AOXCHub.
