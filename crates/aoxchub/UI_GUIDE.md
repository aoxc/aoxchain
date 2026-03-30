# AOXCHub UI Guide

## UX Objectives
- Maintain premium, readable, dark-theme operator ergonomics.
- Keep environment identity visually persistent and impossible to ignore.
- Present command truth with clear preview and explicit confirmation.

## Primary Surfaces
- Top header with localhost safety badge.
- Large environment selector for MAINNET and TESTNET.
- Environment banner across primary dashboard area.
- Binary source status card with trust and path metadata.
- Command catalog cards with risk and policy notes.
- Read-only command preview panel.
- Terminal activity panel with live SSE updates.

## Theme Model
- MAINNET: restrained navy/graphite profile.
- TESTNET: experimental indigo/cyan profile.
- CSS variables switch by `data-env` attribute and update all key surfaces.

## Operator Flow
1. Choose environment.
2. Choose policy-compliant binary source.
3. Select action card.
4. Validate read-only command preview.
5. Confirm execution.
6. Observe streamed terminal output.
