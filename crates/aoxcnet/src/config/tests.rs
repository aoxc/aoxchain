use super::*;

#[test]
fn default_network_uses_canonical_primary_p2p_port() {
    let config = NetworkConfig::default();

    assert!(config.listen_addr.contains("2727"));
    assert!(config.public_advertise_addr.contains("2727"));
}

#[test]
fn serial_identity_default_uses_canonical_aoxc_values() {
    let serial_identity = SerialIdentityPolicy::default();

    assert_eq!(serial_identity.canonical_chain_name, "AOXC-MAINNET");
    assert_eq!(serial_identity.canonical_serial_label, "AOXC-000001");
    assert_eq!(serial_identity.genesis_origin_serial, "000000000001");
    assert_eq!(serial_identity.protocol_serial, 2626);
    assert_eq!(serial_identity.bip44_coin_type, 2626);
    assert_eq!(serial_identity.derivation_path_prefix(), "m/44'/2626'/0'/0");
}

#[test]
fn interop_exposes_canonical_chain_identity() {
    let interop = InteropPolicy::default();

    assert_eq!(interop.canonical_chain_id(), "AOXC-MAINNET");
    assert_eq!(interop.canonical_protocol_serial(), 2626);
    assert!(interop.is_domain_allowed(ExternalDomainKind::Native));
}

#[test]
fn serial_identity_validation_rejects_non_numeric_genesis_serial() {
    let serial_identity = SerialIdentityPolicy {
        genesis_origin_serial: "00000A000001".to_string(),
        ..SerialIdentityPolicy::default()
    };

    assert_eq!(
        serial_identity.validate(),
        Err("SERIAL_IDENTITY_GENESIS_SERIAL_NON_NUMERIC")
    );
}

#[test]
fn interop_validation_requires_native_domain() {
    let interop = InteropPolicy {
        allowed_domains: vec![ExternalDomainKind::Evm],
        ..InteropPolicy::default()
    };

    assert_eq!(
        interop.validate(),
        Err("INTEROP_POLICY_NATIVE_DOMAIN_REQUIRED")
    );
}

#[test]
fn audit_strict_validation_rejects_weak_ban_duration() {
    let mut config = NetworkConfig {
        security_mode: SecurityMode::AuditStrict,
        ..NetworkConfig::default()
    };

    assert!(config.validate().is_ok());

    config.peer_ban_secs = 60;
    assert_eq!(config.validate(), Err("NETWORK_CONFIG_BAN_DURATION_WEAK"));
}

#[test]
fn audit_strict_validation_rejects_excessive_handshake_timeout() {
    let config = NetworkConfig {
        security_mode: SecurityMode::AuditStrict,
        handshake_timeout_ms: 31_000,
        idle_timeout_ms: 30_000,
        ..NetworkConfig::default()
    };

    assert_eq!(
        config.validate(),
        Err("NETWORK_CONFIG_HANDSHAKE_TIMEOUT_EXCEEDS_IDLE_TIMEOUT")
    );
}

#[test]
fn secure_modes_reject_plain_tcp_transport_preference() {
    let config = NetworkConfig {
        transport_preference: TransportPreference::Tcp,
        ..NetworkConfig::default()
    };

    assert_eq!(
        config.validate(),
        Err("NETWORK_CONFIG_MTLS_TRANSPORT_REQUIRED")
    );
}

#[test]
fn secure_modes_require_domain_attestation() {
    let mut config = NetworkConfig::default();
    config.interop.require_domain_attestation = false;

    assert_eq!(
        config.validate(),
        Err("NETWORK_CONFIG_DOMAIN_ATTESTATION_REQUIRED")
    );
}

#[test]
fn validation_rejects_zero_inbound_queue_capacity() {
    let config = NetworkConfig {
        max_inbound_queue: 0,
        ..NetworkConfig::default()
    };

    assert_eq!(
        config.validate(),
        Err("NETWORK_CONFIG_INBOUND_QUEUE_INVALID")
    );
}

#[test]
fn serde_roundtrip_preserves_serial_identity_fields() {
    let config = NetworkConfig::default();

    let serialized =
        serde_json::to_string(&config).expect("network config serialization must succeed");
    let deserialized: NetworkConfig =
        serde_json::from_str(&serialized).expect("network config deserialization must succeed");

    assert_eq!(
        config.interop.serial_identity.canonical_chain_name,
        deserialized.interop.serial_identity.canonical_chain_name
    );
    assert_eq!(
        config.interop.serial_identity.protocol_serial,
        deserialized.interop.serial_identity.protocol_serial
    );
    assert_eq!(
        config.interop.serial_identity.bip44_coin_type,
        deserialized.interop.serial_identity.bip44_coin_type
    );
}
