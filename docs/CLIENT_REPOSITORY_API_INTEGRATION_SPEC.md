# Client Repository API Integration Specification

## Purpose

This document defines exactly what the newly created client-facing repository must implement against AOXChain's API kernel.

It is intended to remove ambiguity during cross-repository handoff.

## Upstream Contract Source of Truth

The client repository must treat the following as canonical:

1. `docs/API_REFERENCE.md` (HTTP/gRPC route surface and error model)
2. `docs/API_KERNEL_SECURITY_BLUEPRINT.md` (security controls and closure criteria)
3. Version-tagged API contract artifacts (OpenAPI/gRPC descriptors when published)

## Required Client Repository Deliverables

## 1. API Access Layer

- A dedicated API SDK/client module (no direct ad-hoc HTTP calls scattered across UI code).
- Typed request/response models for every consumed endpoint.
- Central request middleware for authentication headers, request IDs, and retry policies.

## 2. Authentication and Identity

- Token lifecycle handling (short-lived access token + refresh flow).
- Explicit support for secure machine/client identity lanes where required.
- Deterministic request ID generation attached to every outbound call.

## 3. Error Handling Contract

The client must explicitly map and handle:

- `INVALID_REQUEST`
- `METHOD_NOT_FOUND`
- `RATE_LIMIT_EXCEEDED` (`retry_after_ms` aware backoff)
- `MTLS_AUTH_FAILED` (for privileged operational lanes)
- `PAYLOAD_TOO_LARGE`
- `INTERNAL_ERROR`

For each code, the client must define:

- user-safe message,
- telemetry event,
- retry/no-retry decision,
- escalation path for operators.

## 4. Backoff and Rate-Limit Behavior

- Honor `retry_after_ms` when present.
- Use exponential backoff + jitter for transient failures.
- Avoid parallel retry storms by applying per-route retry budgets.

## 5. Request Construction Rules

- JSON POST requests must always set `Content-Type: application/json`.
- Mutation requests must enforce idempotency strategy where applicable.
- Payload shaping must respect upstream payload-size constraints.

## 6. Security and Secret Handling

- No secrets in source code or static client bundles.
- Secure local token storage policy and rotation strategy.
- Environment-specific endpoint and trust configuration separation.

## 7. Observability and Auditability

- Propagate and log request IDs and correlation IDs.
- Emit metrics for API latency, failure classes, and retry attempts.
- Include structured audit events for privileged actions.

## 8. Compatibility and Versioning

- Pin to a minimum supported API contract version.
- Enforce explicit compatibility checks during CI for breaking API changes.
- Maintain deprecation handling for removed/renamed endpoints.

## 9. CI Gates in the Client Repository

The client repository must fail CI if any gate fails:

- API contract conformance tests,
- error-mapping completeness checks,
- retry policy verification,
- integration smoke tests against staging API,
- security lint and secret scanning.

## 10. Handoff Acceptance Checklist

Client repository implementation is accepted only when:

- [ ] All consumed endpoints are typed and contract-tested.
- [ ] All upstream error codes are mapped and covered by tests.
- [ ] Retry and rate-limit logic is implemented and verified.
- [ ] Request ID propagation is observable in logs/metrics.
- [ ] Security review confirms no secret leakage and proper token handling.
- [ ] Staging integration tests pass with current API kernel baseline.
