# AOXHub Desktop

AOXHub Desktop is the Tauri/React operator control surface for AOXChain.

## Mission

Provide a clear, auditable desktop experience for node operators without weakening the underlying CLI/runtime security model.

## Scope

- launch readiness and blocker visibility,
- cluster and node status monitoring,
- wallet/treasury/recovery visibility,
- report export and operational evidence presentation,
- command queue surfaces mapped to backend controls.

## Design boundary with AOXCVM

- `aoxcvm` is protocol-adjacent execution logic.
- `aoxchub` is operator UX.
- Desktop must orchestrate and display; it should not become a hidden consensus path.

## Operator safety expectations

1. Destructive actions should be explicit and reviewable.
2. Read-only vs mutating actions must be clearly separated.
3. Command mapping to CLI/runtime should be transparent.
4. Error feedback should be actionable for incident response.

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```
