# AOXChain Release Criticality Tiers

Every major crate is assigned one primary release tier and one primary architecture domain.

## Tier definitions

- **Tier 0:** Consensus Critical
- **Tier 1:** Deterministic Runtime Critical
- **Tier 2:** Network / Availability / Persistence Critical
- **Tier 3:** Operator / Control Plane
- **Tier 4:** Application / Peripheral / Experimental

## Crate classification

| Crate | Primary domain | Primary tier | Secondary concern(s) | Rationale | Release-blocking implication |
|---|---|---:|---|---|---|
| `aoxcore` | Kernel | Tier 0 | runtime interfaces | Defines canonical chain primitives and state-transition data models. | Any correctness regression blocks release. |
| `aoxcunity` | Kernel | Tier 0 | persistence/recovery hooks | Implements quorum/safety/finality and consensus engine semantics. | Any safety/finality regression blocks release. |
| `aoxcexec` | Runtimes | Tier 1 | kernel coupling | Owns deterministic execution-lane policy and receipt/accounting boundaries. | Replay or policy drift blocks release. |
| `aoxcvm` | Runtimes | Tier 1 | service bridging | Multi-lane execution dispatcher that can affect deterministic execution outcomes. | Lane nondeterminism blocks release. |
| `aoxcenergy` | Runtimes | Tier 1 | economics policy | Economic arithmetic primitives influence deterministic runtime accounting decisions. | Arithmetic/policy determinism failure blocks release. |
| `aoxcnet` | System Services | Tier 2 | observability | Networking/discovery/gossip/sync are availability-critical but not protocol authority. | Liveness/regression can block network readiness. |
| `aoxcrpc` | System Services | Tier 2 | operator/client ingress | RPC is critical for operability and clients, outside consensus authority. | API integrity/availability regressions block service release. |
| `aoxcdata` | System Services | Tier 2 | kernel persistence | Persistence/index/integrity services are required for recoverability and operation. | Data integrity/recovery regressions block release. |
| `aoxconfig` | System Services | Tier 2 | operator controls | Typed config drives service/runtime behavior and must validate correctly. | Invalid config acceptance can block release. |
| `aoxclibs` | System Services | Tier 2 | deterministic utility base | Shared low-level types/encoding/time can cascade failures across critical crates. | Breaking shared primitives can block release. |
| `aoxchal` | System Services | Tier 2 | deployment optimization | HAL layer supports performance/deployment reliability, not direct consensus authority. | Critical runtime/platform breakage can block release. |
| `aoxcmd` | Operator Environment | Tier 3 | bootstrap/orchestration | Authoritative operational shell for bootstrap, db lifecycle, diagnostics. | Unsafe operator workflow regressions block ops release. |
| `aoxckit` | Operator Environment | Tier 3 | key lifecycle | Operator crypto/key lifecycle toolkit, control-plane critical not consensus authority. | Key custody workflow regression blocks release. |
| `aoxchub` | Operator Environment | Tier 3 | operator UX | Desktop control surface for orchestration and visibility only. | Hidden mutating path or unsafe UX can block release. |
| `aoxcsdk` | Applications/Peripheral | Tier 4 | integration contracts | Developer integration SDK for external adopters. | Breaks peripheral integrations; does not block consensus release by default. |
| `aoxcontract` | Applications/Peripheral | Tier 4 | runtime metadata model | Contract manifest/descriptor model used across ecosystem boundaries. | Schema breakage blocks ecosystem compatibility, not kernel safety by default. |
| `aoxcai` | Applications/Peripheral | Tier 4 | operator advisory extension | AI extension is explicitly non-authoritative and advisory. | Must not gate kernel release unless coupled to critical path. |
| `aoxcmob` | Applications/Peripheral | Tier 4 | mobile adapter | Mobile integration surface for external/operator scenarios. | Mobile regressions are peripheral unless declared mandatory. |

## Notes

- `tests` workspace member is a verification harness, not an architecture domain crate.
- If a crate spans domains, this table records one primary domain/tier and explicit secondary concerns.
