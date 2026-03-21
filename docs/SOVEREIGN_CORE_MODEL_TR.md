# AOXChain Yerel Egemen Kök Modeli

Bu modelde:

- **yerel zincir** = sovereign constitutional core
- **uzak zincirler** = execution domains

Yerel zincir her şeyi yapan yer değil, **her şeye son sözü söyleyen yer** olmalıdır.

## Yerelde tutulması gereken 7 anayasal kök

1. **Identity**
   - root account registry
   - chain mappings
   - signer bindings
   - recovery authority
   - key rotation rules
   - delegate / permission registry

2. **Supply**
   - total canonical supply
   - mint authority root
   - burn settlement root
   - treasury root
   - emission policy
   - global supply accounting

3. **Governance**
   - protocol upgrades
   - module approvals
   - remote domain authorization
   - risk parameters
   - bridge mint ceilings
   - validator policy

4. **Relay**
   - outbound message commitments
   - inbound settlement acceptance rules
   - nonce root
   - replay protection root
   - approved remote domains
   - message policy classes

5. **Security**
   - validator set
   - attester set
   - quorum thresholds
   - slashing / penalty logic
   - signature policy
   - emergency security overrides

6. **Settlement**
   - final settlement records
   - remote execution receipts hash
   - dispute intake
   - final confirmation state
   - accounting closure
   - cross-domain settlement journal

7. **Treasury**
   - protocol treasury
   - reserve balances
   - insurance / emergency reserve
   - strategic liquidity authority
   - module funding authority

## Yerelde olmaması gerekenler

- ağır uygulama mantığı
- zincire özel dApp logic
- uzak ağ entegrasyonunun implementation detayı
- büyük veri yükü
- AI karar motoru
- deneysel app execution

## Kısa hüküm

Yerel zincir:

- kimliği tanır,
- arzı yönetir,
- yönetişimi taşır,
- mesajları meşrulaştırır,
- güvenliği tanımlar,
- settlement'ı kapatır,
- rezervi korur.

Uzak zincirler ise execution, entegrasyon ve likidite alanıdır.
