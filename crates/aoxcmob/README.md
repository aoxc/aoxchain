# README.md

> Scope: `crates/aoxcmob`

## Purpose
Contains mobile/adapter access layers and session-security helpers.

## Contents at a glance
- The code and files in this directory define the runtime behavior of this scope.
- The folder contains modules and supporting assets bounded by this responsibility domain.
- Any change should be evaluated together with its testing and compatibility impact.

## Integration Surfaces
- `NativeGateway` provides deterministic device provisioning, short-lived session opening, and signed task receipt submission.
- `HttpRelayTransport` provides a production-oriented HTTPS JSON transport implementation for relay integration.
- `SecureStore` remains the platform boundary for Android Keystore / iOS Keychain style credential custody.
- `MobileConfig.relay_verifying_key_hex` enables optional relay challenge + permit signature verification at the gateway boundary.
