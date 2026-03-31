# Coverage Status

## Purpose

This document defines how AOXCHAIN coverage is measured, reported, and reviewed for release readiness.

## Current Reporting Contract

- Coverage reports are generated with `cargo llvm-cov`.
- The canonical inventory input for denominator sanity is `artifacts/testing/test_inventory.json`.
- Coverage findings must be attached to the same commit/branch under review.

## Required Views

At minimum, each coverage run must include:

- workspace-wide line coverage,
- workspace-wide function coverage,
- crate-level line coverage table,
- explicit excluded-path rationale (generated code, vendored artifacts, intentionally unreachable mocks).

## Release Gate Expectations

A change is not release-ready when one of the following occurs:

- coverage execution is skipped without documented risk acceptance,
- critical-path crates lose coverage with no compensating targeted tests,
- exclusions are added without an explicit engineering reason.

## Operational Notes

Coverage percentage alone is not sufficient. Reviewers must evaluate whether critical invariants in `docs/testing/CRITICAL_INVARIANTS.md` are actively exercised by tests.
