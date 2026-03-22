# Security Policy

## 1. Supported Versions

We prioritize the security of the AOXChain ecosystem. Security updates, including critical patches for consensus integrity and cryptographic vulnerabilities, are provided for the following active release branches:

| Version | Status | Security Support |
| :--- | :--- | :--- |
| **v5.1.x** | Mainline | :white_check_mark: Full Support |
| **v5.0.x** | Deprecated | :x: End of Life (EOL) |
| **v4.0.x** | LTS (Long-Term Support) | :white_check_mark: Critical Fixes Only |
| **< v4.0** | Legacy | :x: No Support |

## 2. Reporting a Vulnerability

If you discover a security vulnerability within the AOXChain core, its associated crates, or the Unity Consensus engine, please follow our **Responsible Disclosure** protocol. 

**Do not open a public GitHub issue for security-related findings.**

### 2.1 Submission Process
Please send a detailed technical report to **security@aoxchain.io**. For sensitive information, we recommend encrypting your communication using our PGP public key.

### 2.2 Reporting Requirements
To assist our engineers in the triage process, please include:
* **Vulnerability Class:** (e.g., ZKP Logic Error, Double Spend Vector, Consensus Bypass, or Remote Code Execution).
* **Proof of Concept (PoC):** Functional scripts, logs, or a step-by-step guide to reproduce the state.
* **Impact Assessment:** Potential threat to network finality, validator stability, or user funds.

## 3. Incident Response & Remediation

Upon receiving a valid report, the AOXChain Security Team will:
1.  **Acknowledge:** Receipt of report within 24 hours.
2.  **Triage:** Technical validation of the exploit vector within 3-5 business days.
3.  **Silent Patching:** Fixes are developed in isolated private repositories to prevent "front-running" of the vulnerability.
4.  **Coordinated Rollout:** For consensus-critical bugs, we coordinate with the validator set for a synchronized hard-fork or patch application.
5.  **Public Advisory:** A full post-mortem and CVE (if applicable) will be released after the network is secured.

## 4. Scope of Audit

We are particularly interested in high-severity reports concerning:
* **Zero-Knowledge Proofs:** Soundness or completeness errors in our ZKP circuits.
* **Unity Consensus Engine:** Faults in the BFT logic or safety/liveness violations.
* **Multi-Lane VM Isolation:** Cross-lane state pollution or gas-metering bypasses in EVM/WASM/Move environments.
* **P2P Hardening:** Eclipse attacks, sybil vectors, or high-amplification DoS.

---
*Qualified researchers may be eligible for rewards under the AOXChain Bug Bounty Program based on the CVSS v3.1 score of the finding.*
