# API Kernel Security Blueprint

## Purpose

This document defines the production security contract for the AOXChain API kernel so that a separate client/application repository can integrate against a stable, auditable, and implementation-aligned interface.

The scope is the API security control plane (authentication, authorization, admission, transport, policy, telemetry, and failure behavior), not UI/client concerns.

## Non-Goals

- Client UX design, wallet flows, and front-end state management.
- Product-level business logic unrelated to API admission and policy enforcement.
- Temporary or best-effort controls without explicit verification criteria.

## API Kernel Completion Criteria

The API kernel is considered complete for downstream integration only when all controls below are implemented and test-gated.

### 1. Identity and Session Admission

- OAuth2/OIDC token intake with strict issuer/audience validation.
- Short-lived access tokens and refresh rotation policy.
- mTLS requirement for service-to-service and privileged lanes.
- Deterministic request identity: `request_id`, `trace_id`, and caller principal attached at admission.

### 2. Authorization and Policy Engine

- Default-deny authorization posture.
- RBAC baseline with policy extension points for attribute/predicate rules.
- Endpoint-method policy map with explicit allowed principals/scopes.
- Policy evaluation result attached to audit event for every denied request.

### 3. Input and Contract Validation

- Strict schema validation for every externally reachable endpoint.
- Deterministic canonicalization of accepted payloads before business execution.
- Maximum payload size, field bounds, and content-type enforcement.
- Unknown fields rejected unless explicitly version-gated.

### 4. Abuse and Resource Protection

- Multi-dimensional rate limiting (principal, IP, route class).
- Concurrency and timeout caps per endpoint class.
- Idempotency key requirement on mutation paths.
- Replay resistance on signed/privileged operations.

### 5. Error and Failure Model

- External error responses are sanitized and stable.
- Internal diagnostics preserve actionable details with correlation fields.
- Dependency failure policy is explicit (`fail-closed` for authz/policy, controlled degradation for non-security paths).
- Error taxonomy versioned and documented.

### 6. Cryptography and Secret Lifecycle

- TLS 1.3 on all public and internal API transport paths.
- Secrets and key material sourced from a managed secret provider.
- Key rotation cadence and emergency revocation runbook defined.
- Crypto agility hooks in place for post-quantum migration compatibility.

### 7. Auditability and Telemetry

- Immutable audit event for authentication, authorization deny, and privileged mutation.
- Metrics for auth failures, policy denies, limiter drops, and timeout/circuit events.
- Structured logs with stable machine-readable fields.
- Security alerts wired to operational response runbooks.

## Interface Contract for the Separate Client Repository

The downstream application repository can be started only after this repository publishes the following artifacts:

1. Stable OpenAPI/gRPC contract references (versioned paths).
2. Authentication profile (issuer, audience, scope matrix, token TTL constraints).
3. Error code catalog and retry/non-retry classification.
4. Idempotency/replay requirements for each mutation endpoint.
5. Rate-limit policy headers and expected client backoff behavior.
6. Compatibility and deprecation policy (minimum supported API version window).

## Required Delivery Artifacts in This Repository

Before client repository kick-off, this repository must contain:

- Updated `docs/API_REFERENCE.md` with endpoint-level security requirements.
- Test evidence for admission, authz, replay, rate-limit, and failure handling.
- Operator runbook for secret rotation and emergency token/key revocation.
- Change log entry that marks the API kernel security baseline version.

## Verification Gates

The API kernel baseline must fail release if any gate below fails:

- Authentication bypass regression tests.
- Authorization matrix completeness tests.
- Input validation and malformed payload tests.
- Replay and idempotency enforcement tests.
- Rate-limit and timeout policy tests.
- Audit-event integrity and correlation tests.

## Handoff Checklist to Client Repository

The handoff is valid only when all checklist items are true:

- [ ] API contract version is tagged and published.
- [ ] Security profile document is published and referenced by API docs.
- [ ] Error catalog is finalized and versioned.
- [ ] Non-breaking versioning and deprecation window is documented.
- [ ] Integration examples are validated against current server implementation.
- [ ] Runbooks for token/key incidents are approved for operations.

## Governance Note

This blueprint is an engineering control document under the repository's MIT-licensed, no-warranty posture and must remain aligned with implemented behavior.
