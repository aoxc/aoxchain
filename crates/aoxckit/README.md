# READ.md

> Scope: `crates/aoxckit`

## Purpose
`aoxckit` provides the AOXC operator-facing cryptographic toolkit surface.

This crate hosts command-line workflows for:
- key generation and key custody preparation,
- actor identifier derivation,
- certificate issuance, verification, and inspection,
- passport inspection,
- encrypted keyfile workflows,
- registry mutation and listing,
- revocation operations,
- quorum evaluation,
- ZKP setup artifact initialization.

The crate is intended to act as a controlled operational boundary between
human operators, automation pipelines, and the lower-level AOXC identity
and cryptographic primitives implemented in other workspace crates.

## Design objectives
The primary design goals for this scope are:

- preserve a clear command-plane boundary for sensitive operator actions,
- keep public-safe output separate from dangerous plaintext secret flows,
- provide deterministic, machine-readable JSON output where appropriate,
- centralize filesystem persistence behavior through hardened shared helpers,
- keep command routing explicit and auditable,
- ensure each command surface remains independently testable,
- minimize accidental secret leakage to stdout, stderr, shell history, or logs.

## Security posture
`aoxckit` is part of the AOXC operational trust boundary.

The crate is responsible for invoking security-sensitive workflows, but it is
not itself the root source of all cryptographic logic. Cryptographic derivation,
identity primitives, certificate models, and key custody formats are delegated
to lower-level crates such as `aoxcore`.

Current security expectations for this scope include:

- public CLI output must not expose plaintext private key material,
- dangerous plaintext re-materialization flows must remain explicit,
- encrypted persistence is preferred over raw secret export,
- file writes should be atomic and permission-hardened where supported,
- command errors should be deterministic and safe for operator-facing stderr,
- changes to command contracts must be reviewed together with integration tests.

## Dependency and responsibility boundary
This crate depends on lower-level AOXC identity and cryptographic modules, but
it should remain focused on operational orchestration rather than re-implementing
cryptographic primitives.

In particular:

- `aoxcore` remains the canonical home for identity, key derivation, certificate,
  keyfile, and related cryptographic building blocks,
- `aoxckit` provides the operator and automation command surface over those primitives,
- shared command-plane persistence and serialization helpers are centralized under
  the local `keyforge::util` module,
- business logic should remain in command handlers, not in the entrypoint.

## Module layout
The `src/keyforge/` tree defines the command surface for this crate.

### Core module groups
- `cli`  
  Canonical command-line type definitions, argument contracts, and subcommand layout.

- `util`  
  Shared filesystem and serialization helpers used by command handlers. This module
  is security-relevant because it governs read, write, JSON persistence, and
  atomic output behavior.

### Command handlers
- `cmd_key`  
  Key generation, inspection, and key custody-oriented workflows.

- `cmd_keyfile`  
  Encrypted keyfile creation and controlled plaintext decryption workflows.

- `cmd_actor_id`  
  Actor identifier derivation workflows over canonical AOXC identity input.

- `cmd_cert`  
  Certificate issuance, verification, inspection, and mTLS template workflows.

- `cmd_passport`  
  Passport inspection and passport-level document validation workflows.

- `cmd_registry`  
  Registry load, normalization, upsert, and listing workflows.

- `cmd_revoke`  
  Registry-backed revocation operations.

- `cmd_quorum`  
  Deterministic quorum evaluation logic and operator-facing result emission.

- `cmd_zkp_setup`  
  Trusted-setup initialization artifact workflows.

## Entrypoint behavior
The crate binary entrypoint is intentionally narrow.

The `main.rs` entrypoint is expected to:
- parse the CLI contract,
- dispatch exactly one command,
- return a deterministic process exit code,
- emit failures through stderr only,
- preserve a stable and sanitized operator-facing error format.

Business logic should not accumulate in the entrypoint. Command-specific behavior
belongs in the relevant handler module.

## Output policy
Where feasible, command output should be:
- JSON-based,
- deterministic,
- stable enough for automation,
- explicit about whether it is public-safe or operationally sensitive.

Public inspection and summary commands should emit public-only data.
Dangerous flows that produce or re-materialize plaintext secret material must
remain clearly marked and operationally constrained.

## File persistence policy
This scope may read and write:
- encrypted keyfiles,
- registry documents,
- revocation-updated registry state,
- setup artifacts,
- certificate or passport inspection output,
- other command-generated JSON artifacts.

Persistence behavior should follow these principles:
- reject blank paths,
- normalize operator input before filesystem access,
- create parent directories safely when needed,
- prefer atomic file replacement over direct truncating writes,
- harden file permissions for sensitive artifacts where the platform supports it.

## Testing expectations
Any change within this scope should be evaluated across three layers:

### Unit tests
Unit tests should validate:
- normalization behavior,
- argument handling,
- deterministic command logic,
- serialization format,
- local helper functions,
- error mapping behavior.

### Integration tests
CLI integration tests should validate:
- command dispatch,
- success and failure exit behavior,
- stdout/stderr contracts,
- registry round trips,
- keyfile workflows,
- dangerous-flow acknowledgement rules.

### Compatibility review
Because `aoxckit` is a command-plane surface, command changes should also be
reviewed for:
- backward compatibility,
- operator workflow impact,
- automation breakage risk,
- output-contract drift,
- security regression risk.

## Change management guidance
Changes in this scope should be treated as operationally sensitive when they affect:
- key custody behavior,
- stdout or stderr content,
- file write semantics,
- path handling,
- password source handling,
- plaintext secret export or re-materialization,
- registry mutation behavior,
- certificate issuance or verification semantics.

Any such change should be evaluated together with:
- tests,
- compatibility implications,
- failure-mode behavior,
- operator misuse risk,
- documentation updates.

## Current-state honesty
This crate is part of an experimental pre-release codebase.

That means:
- interfaces may evolve,
- command contracts may be tightened,
- operational hardening may continue,
- some workflows may still represent initialization or metadata stages rather than
  full production ceremonies.

Production claims should therefore be based on validated behavior, tests, and
release evidence, not on the presence of command surfaces alone.

## Recommended review focus for contributors
When reviewing changes under `crates/aoxckit`, pay particular attention to:

- whether any new stdout output could leak sensitive material,
- whether dangerous operations require explicit acknowledgement,
- whether file writes remain atomic and permission-conscious,
- whether command outputs remain deterministic and parseable,
- whether handler logic is independently testable,
- whether error messages remain stable and safe for logs and CI,
- whether lower-level cryptographic invariants are being delegated correctly.

## Contents at a glance
- The code and files in this directory define the runtime behavior of this scope.
- The folder contains modules and supporting assets bounded by this responsibility domain.
- Any change should be evaluated together with its testing, compatibility, and security impact.
