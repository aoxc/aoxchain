# Testing

## Test Types
Unit tests, integration tests, and regression tests focused on this crate's critical behavior.

## Critical Flows
Validate core success paths, failure paths, and deterministic behavior guarantees that other modules rely on.

## Commands
- `cargo test -p aoxcore`
- `cargo test -p aoxcore --all-features`

## Notes
Behavior changes should include targeted regression coverage and compatibility checks for dependent crates.
