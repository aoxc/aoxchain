# AOXCVM Phase-2 Execution Law (Canonical)

This document is the canonical, spec-first law surface for AOXCVM Phase-2 runtime policy.

## 1) Contract Class Constitution

| Class | Allowed capabilities | Forbidden capabilities | Allowed TxKind | Governance requirement | Auth restriction behavior | Upgrade rule | Metadata mutation rule | Registry access rule | Syscall scope |
|---|---|---|---|---|---|---|---|---|---|
| Application | `storage_read`, `storage_write`, `package_dependency_access`, `restricted_syscalls` | `registry_access`, `governance_hooks`, `upgrade_authority`, `metadata_mutation` | `UserCall` | Optional, but `governance_activation_required=true` is invalid | `restricted_to_auth_profile` forbidden | Denied | Denied | Denied | Restricted by capability profile |
| System | All profile capabilities | None by class-law default | `System`, `Governance` | Allowed | `restricted_to_auth_profile` forbidden | Allowed by policy/governance process | Allowed | Allowed | Full system/runtime scope |
| Governed | `storage_read`, `storage_write`, `package_dependency_access`, `registry_access`, `governance_hooks`, `restricted_syscalls`, `metadata_mutation` | `upgrade_authority` | `Governance`, `System` | Allowed; governance lane expected | `restricted_to_auth_profile` forbidden | Denied (class-law) | Allowed | Allowed | Governed syscall subset |
| Package | `storage_read`, `package_dependency_access`, `restricted_syscalls` | `storage_write`, `registry_access`, `governance_hooks`, `upgrade_authority`, `metadata_mutation` | `PackagePublish`, `System`, `Governance` | Optional | `restricted_to_auth_profile` forbidden | Denied | Denied | Denied | Publish/load oriented scope |
| PolicyBound | `storage_read`, `storage_write`, `package_dependency_access`, `registry_access`, `restricted_syscalls` | `governance_hooks`, `upgrade_authority`, `metadata_mutation` | `UserCall`, `Governance`, `System` (if governance activation required => only `Governance`/`System`) | Optional/conditional | `restricted_to_auth_profile` mandatory and canonicalized | Denied | Denied | Allowed | Restricted with profile-bound admission |

## 2) Capability Enforcement Matrix

| Capability | Allowed classes | Denied classes | Manifest deny | Resolver deny | Admission deny | Runtime syscall gate |
|---|---|---|---|---|---|---|
| `storage_read` | Application, System, Governed, Package, PolicyBound | - | No | No | No | Required |
| `storage_write` | Application, System, Governed, PolicyBound | Package | Yes | Yes | Optional class checks | Required |
| `package_dependency_access` | Application, System, Governed, Package, PolicyBound | - | No | No | No | Required |
| `registry_access` | System, Governed, PolicyBound | Application, Package | Yes | Yes | Optional class checks | Required |
| `governance_hooks` | System, Governed | Application, Package, PolicyBound | Yes | Yes | Indirect (governance activation / class lane) | Required |
| `restricted_syscalls` | All (PolicyBound required) | - | PolicyBound missing => Yes | PolicyBound missing => Yes | No | Required |
| `upgrade_authority` | System | Application, Governed, Package, PolicyBound | Yes | Yes | Optional class checks | Required |
| `metadata_mutation` | System, Governed | Application, Package, PolicyBound | Yes | Yes | Optional class checks | Required |

## 3) Policy Enforcement Completion

### `review_required`
- Manifest: field accepted as declarative policy.
- Resolver: rejects review downgrade when contracts config requires review.
- Admission: not evaluated directly (pre-execution policy is already resolved).
- Runtime precondition: descriptor must have passed resolver gate.
- Invariant: review-required network policy cannot be disabled by manifest.

### `governance_activation_required`
- Manifest: class compatibility validated; invalid on `Application`/`Package`.
- Resolver: duplicated fail-closed class compatibility check.
- Admission: requires tx kind `Governance` or `System`.
- Runtime precondition: governance-required descriptor cannot enter user lane.
- Invariant: governance-required + non-governance lane is always rejected.

### `restricted_to_auth_profile`
- Manifest: required for `PolicyBound`; forbidden for others; canonical-id validation enforced.
- Resolver: duplicated fail-closed policy-bound profile validation.
- Admission: active profile required and must match canonical profile id.
- Runtime precondition: profile mismatch never enters execution.
- Invariant: policy-bound contract without valid profile id is non-runnable.

## 4) Tx Kind Law (Canonical)

| ContractClass | TxKind set |
|---|---|
| Application | `UserCall` |
| System | `System`, `Governance` |
| Governed | `Governance`, `System` |
| Package | `PackagePublish`, `System`, `Governance` |
| PolicyBound | `UserCall`, `Governance`, `System` (governance-required narrows to `Governance`/`System`) |

## 5) Static vs Dynamic Boundary

- **Static (manifest validation)**: deterministic class/capability/policy law.
- **Resolution-time (resolver)**: environment-sensitive policy (`review_required`, allowed VM targets) plus fail-closed repetition of static law.
- **Admission-time (runtime gate)**: tx-kind lane legality and auth-profile binding.
- **Execution-time**: syscall capability checks must enforce runtime capabilities before side effects.

## 6) Failure Provenance

Rejection provenance is expected to map to:
- `ManifestValidationError::*` (static manifest law),
- `ContractError::Policy(...)` during resolver law gate,
- `AdmissionError::*` during admission/runtime gate.

This phase keeps fail-closed duplication intentionally: resolver/admission may reject descriptors even if upstream validation was bypassed.
