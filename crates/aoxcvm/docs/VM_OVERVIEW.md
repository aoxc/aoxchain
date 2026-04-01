# VM OVERVIEW

AOXCVM, zincir için tek kanonik yürütme makinesi olarak tasarlanır.

## Core model

AOXCVM = deterministic VM core + verified bytecode + typed object state + capability-native auth + governed upgrade.

## Layering

1. Identity and Authorization
2. Transaction Admission
3. Bytecode Verification
4. VM Execution Core
5. Object State
6. Capability Enforcement
7. Host/Syscall Boundary
8. Gas + Authority Metering
9. Governance/Upgrade Control

Bu katmanlama, execution’ı salt kod çalıştırma olmaktan çıkarıp authority doğrulamalı state transition protokolüne dönüştürür.
