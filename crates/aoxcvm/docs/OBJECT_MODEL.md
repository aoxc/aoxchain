# OBJECT MODEL

AOXCVM state modeli typed object sınıflarına dayanır.

## Canonical classes

- IdentityObject
- AssetObject
- CapabilityObject
- ContractObject
- PackageObject
- VaultObject
- GovernanceObject

Her object için `header`, `owner`, `policy`, `version` ve lifecycle meta-bilgisi taşınır.

Bu yaklaşım auditability, parallelization ve explicit authorization için temel sağlar.
