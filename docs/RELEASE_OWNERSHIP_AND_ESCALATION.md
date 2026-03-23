# AOX Chain Release Ownership and Escalation Matrix

**Governance Version:** `aoxc.v.0.1.0-testnet.1`

## Required Roles
A release is not eligible for promotion until the following owners are explicitly assigned:
- release owner,
- security approver,
- consensus approver,
- networking approver,
- execution/runtime approver,
- key custody approver,
- on-call incident commander,
- communications owner.

## Minimum Approval Rules
- security-critical changes require security approver sign-off,
- consensus-critical changes require consensus approver sign-off,
- release publication requires release owner approval,
- emergency rollback requires release owner plus incident commander approval.

## Escalation Policy
If a high-severity issue affects finality, validator safety, key custody, or artifact trust, the issue must escalate immediately to the release owner, security approver, and incident commander.

## Documentation Rule
This matrix must be updated in the same pull request whenever team ownership or escalation flow changes.
