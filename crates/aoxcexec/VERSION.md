# AOXCEXEC - Version Governance

**Current Version:** `aoxc.v.0.0.0-alpha.2`
**Previous Version:** `aoxc.v.0.0.0-alpha.1`

## Strict Versioning Policy
This directory follows a mandatory version-increment rule. Every material change to code, configuration, contracts, tests, automation, or documentation must advance the directory release label and append an explanatory ledger entry before merge approval.

## Nine Version Governance Rules
1. Every merge-worthy change must increment the directory version.
2. Semantic version values in manifests must remain compatible with the canonical documentation label.
3. Binary artifacts, containers, release notes, and audit records must cite the same version.
4. Compatibility declarations must be updated in the same change set as the implementation.
5. Security fixes must never ship under a reused version identifier.
6. Documentation changes that alter operator behavior must receive a new version and ledger entry.
7. Tests added for new behavior must reference the release they protect when relevant.
8. Rollback instructions must identify the previous trusted version explicitly.
9. Production promotion is forbidden if any touched artifact still references an older incompatible version.

## Upgrade Procedure
- Start from the last trusted version recorded below.
- Apply the implementation change.
- Update READ.md, VERSION.md, manifests, and compatibility fixtures together.
- Execute the required checks and record the results.
- Approve promotion only after evidence is attached to the new version entry.

## Version Ledger
- `aoxc.v.0.0.0-alpha.2`: documentation expanded to a full audit guide, strict version governance introduced for this directory, and the release baseline advanced to alpha.2.
- `aoxc.v.0.0.0-alpha.1`: alpha roadmap and initial directory-level version tracking introduced.
