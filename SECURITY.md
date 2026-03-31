# Security Policy

## Purpose

This policy defines the private reporting process, triage expectations, and disclosure handling model for security issues affecting the AOXChain repository and its associated operational components.

## Private Reporting Requirement

Suspected vulnerabilities must not be disclosed through public issue trackers, pull requests, discussions, or other public communication channels prior to coordinated review and containment.

All security findings must be reported privately to:

**admin@aoxcore.com**

## Required Report Contents

To support efficient triage and reproducibility, security reports should include, at minimum:

- the affected component, subsystem, interface, or operational boundary;
- a clear reproduction procedure, proof-of-concept, or exploit narrative;
- the expected behavior versus the observed behavior;
- an impact statement covering, where applicable, safety, funds, availability, integrity, confidentiality, or consensus risk;
- environmental assumptions, version details, commit references, configuration dependencies, or network conditions relevant to reproduction;
- suggested mitigations or containment options, if known.

Incomplete reports may still be reviewed, but report quality directly affects triage speed and remediation efficiency.

## Response Objectives

AOXChain will make a good-faith effort to:

- acknowledge receipt of a security report within 24 hours;
- perform initial technical triage and severity assessment as promptly as practical;
- validate impact, affected scope, and exploitability before public disclosure;
- coordinate remediation, operational containment, and release strategy according to impact class;
- publish advisory or disclosure notes after reasonable containment has been achieved.

Response timing may vary depending on report complexity, reproducibility, operational exposure, and maintainer availability.

## Priority Security Areas

The following classes of issues are treated as high-priority review areas:

- consensus safety or liveness violations;
- deterministic execution or state transition failures;
- signature, key management, or authorization boundary vulnerabilities;
- RPC, peer-to-peer, or network abuse pathways;
- privilege escalation, persistence boundary bypass, or integrity compromise;
- denial-of-service vectors with material operational impact;
- vulnerabilities that can affect funds, governance, validator trust, or chain continuity.

## Disclosure and Coordination Model

AOXChain follows an evidence-driven and coordinated disclosure approach. Public disclosure may be delayed until one or more of the following conditions are satisfied:

- the issue has been reproduced and validated;
- practical mitigation or containment guidance is available;
- affected operators or stakeholders have had a reasonable opportunity to respond where applicable;
- public disclosure is judged not to create disproportionate additional risk.

## Security Assurance Statement

Security assurance in AOXChain is iterative, evidence-driven, and continuously improved over time. No system, repository, or release should be interpreted as providing an absolute security guarantee.

## Scope and Repository Context

This policy applies to security-relevant issues affecting this repository, including source code, operational workflows, release procedures, and trust-sensitive execution paths directly maintained within the AOXChain project scope.

Third-party dependencies, external infrastructure, and upstream components may introduce inherited risk outside the direct remediation authority of this repository, although such issues may still be triaged for impact and containment relevance.

## License and Liability Context

This repository is provided under the MIT License on an **"as is"** basis, without warranties, guarantees, or liability assumptions by maintainers or contributors, except where required by applicable law.
