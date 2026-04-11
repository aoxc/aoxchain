#[cfg(test)]
mod tests {
    use super::{P2PNetwork, ProtocolEnvelope, SessionTicket, digest_payload};
    use crate::config::{ExternalDomainKind, NetworkConfig, SecurityMode};
    use crate::discovery::{PeerCandidate, RpcSurface};
    use crate::error::NetworkError;
    use crate::gossip::peer::{NodeCertificate, Peer, PeerRole};
    use aoxcunity::messages::ConsensusMessage;
    use aoxcunity::vote::{AuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

    fn test_certificate(subject: &str) -> NodeCertificate {
        NodeCertificate {
            subject: subject.to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: "serial-1".to_string(),
            domain_attestation_hash: "attestation-hash-1".to_string(),
        }
    }

    fn test_peer() -> Peer {
        test_peer_with_role(PeerRole::Validator, "node-1")
    }

    fn test_peer_with_role(role: PeerRole, node_id: &str) -> Peer {
        Peer::new(
            node_id,
            "10.0.0.1:2727",
            "AOXC-MAINNET",
            ExternalDomainKind::Native,
            role,
            3,
            true,
            test_certificate(node_id),
        )
    }

    fn test_vote() -> ConsensusMessage {
        ConsensusMessage::Vote(AuthenticatedVote {
            vote: Vote {
                voter: [1u8; 32],
                block_hash: [2u8; 32],
                height: 1,
                round: 0,
                kind: VoteKind::Prepare,
            },
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: 0,
                validator_set_root: [3u8; 32],
                pq_attestation_root: [5u8; 32],
                signature_scheme: 2,
            },
            signature: vec![4u8; 64],
            pq_public_key: Some(vec![7u8; 32]),
            pq_signature: Some(vec![8u8; 32]),
        })
    }

    fn classical_vote() -> ConsensusMessage {
        ConsensusMessage::Vote(AuthenticatedVote {
            vote: Vote {
                voter: [1u8; 32],
                block_hash: [6u8; 32],
                height: 2,
                round: 0,
                kind: VoteKind::Prepare,
            },
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: 0,
                validator_set_root: [3u8; 32],
                pq_attestation_root: [0u8; 32],
                signature_scheme: 1,
            },
            signature: vec![4u8; 64],
            pq_public_key: None,
            pq_signature: None,
        })
    }

    fn candidate(peer_id: &str, genesis: &str, quantum_ready: bool) -> PeerCandidate {
        PeerCandidate {
            peer_id: peer_id.to_string(),
            advertise_addr: "10.0.0.1:2727".to_string(),
            score: 10,
            source: "bootnodes".to_string(),
            last_seen_unix: 1_700_000_000,
            genesis_fingerprint: genesis.to_string(),
            rpc: RpcSurface {
                http_endpoint: format!("http://{peer_id}.mesh.local:28657"),
                jsonrpc_endpoint: format!("http://{peer_id}.mesh.local:28657/jsonrpc"),
                quantum_ready,
            },
        }
    }

    #[test]
    fn checked_constructor_rejects_invalid_config() {
        let config = NetworkConfig {
            max_outbound_peers: 0,
            ..NetworkConfig::default()
        };

        let result = P2PNetwork::new_checked(config);
        assert!(matches!(result, Err(NetworkError::InvalidConfig(_))));
    }

    #[test]
    fn discovery_ingestion_accepts_only_matching_genesis() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        let local = net.local_genesis_fingerprint().to_string();

        assert!(net.ingest_discovery_candidate(candidate("peer-1", &local, false)));
        assert!(!net.ingest_discovery_candidate(candidate("peer-2", "different-genesis", true)));
    }

    #[test]
    fn discovery_ingestion_honors_disable_switch() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        net.set_auto_discovery_enabled(false);
        let local = net.local_genesis_fingerprint().to_string();

        assert!(!net.ingest_discovery_candidate(candidate("peer-1", &local, true)));
        assert!(net.bootstrap_rpc_surfaces(8).is_empty());
    }

    #[test]
    fn secure_broadcast_requires_active_session() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        net.register_peer(test_peer())
            .expect("peer should register");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("broadcast without session must fail");

        assert!(matches!(err, NetworkError::UnknownPeer(_)));
    }

    #[test]
    fn session_based_broadcast_is_accepted() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let envelope = net
            .broadcast_secure("node-1", test_vote())
            .expect("broadcast should be accepted");

        assert_eq!(envelope.chain_id, "AOXC-MAINNET");
        assert_eq!(envelope.protocol_serial, 2626);
        assert!(!envelope.payload_hash_hex.is_empty());
        assert!(!envelope.frame_hash_hex.is_empty());
        assert!(net.receive().is_some());
    }

    #[test]
    fn secure_mode_rejects_duplicate_active_handshake_for_same_peer() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("first session should be established");

        let err = net
            .establish_session("node-1")
            .expect_err("duplicate secure handshake must be rejected");

        assert!(matches!(err, NetworkError::PeerAdmissionDenied(_)));
    }

    #[test]
    fn banned_peer_cannot_broadcast() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");
        net.ban_peer("node-1");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("banned peer must not broadcast");

        assert!(matches!(err, NetworkError::PeerBanned(_)));
    }

    #[test]
    fn banned_peer_cannot_register_again_during_ban_window() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer())
            .expect("peer should register");
        net.ban_peer("node-1");

        let err = net
            .register_peer(test_peer())
            .expect_err("banned peer must not re-register");

        assert!(matches!(err, NetworkError::PeerBanned(_)));
    }

    #[test]
    fn replay_cache_detects_duplicate_nonce_for_same_session() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer())
            .expect("peer should register");
        let ticket = net
            .establish_session("node-1")
            .expect("session should be established");

        let envelope = ProtocolEnvelope::new("AOXC-MAINNET", 2626, &ticket, test_vote(), 100)
            .expect("envelope creation must succeed");

        let replay_key = format!("{}:{}", envelope.session_id, envelope.nonce);
        net.replay_cache.insert(replay_key.clone());
        net.replay_order.push_back(replay_key);

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("duplicate session nonce must be rejected");

        assert!(matches!(err, NetworkError::ReplayDetected));
    }

    #[test]
    fn expired_session_is_rejected_in_secure_mode() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let session = net
            .sessions
            .get_mut("node-1")
            .expect("session should exist");
        session.expires_at_unix = 0;

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("expired session must be rejected");

        assert!(matches!(err, NetworkError::HandshakeTimeout));
    }

    #[test]
    fn insecure_mode_allows_expired_session_broadcast() {
        let config = NetworkConfig {
            security_mode: SecurityMode::Insecure,
            ..NetworkConfig::default()
        };

        let mut net = P2PNetwork::new(config);

        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let session = net
            .sessions
            .get_mut("node-1")
            .expect("session should exist");
        session.expires_at_unix = 0;

        assert!(net.broadcast_secure("node-1", test_vote()).is_ok());
    }

    #[test]
    fn mutual_auth_rejects_classical_vote_payload() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let err = net
            .broadcast_secure("node-1", classical_vote())
            .expect_err("classical vote must be rejected");
        assert!(matches!(err, NetworkError::QuantumPolicyViolation(_)));
    }

    #[test]
    fn audit_strict_rejects_observer_session_admission() {
        let config = NetworkConfig {
            security_mode: SecurityMode::AuditStrict,
            ..NetworkConfig::default()
        };
        let mut net = P2PNetwork::new(config);
        let observer_peer = test_peer_with_role(PeerRole::Observer, "observer-1");

        net.register_peer(observer_peer)
            .expect("observer should register");

        let err = net
            .establish_session("observer-1")
            .expect_err("audit strict must reject observer session admission");

        assert!(matches!(err, NetworkError::PeerAdmissionDenied(_)));
        assert!(
            err.to_string().contains("peer class forbidden"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn register_peer_rejects_capacity_overflow() {
        let config = NetworkConfig {
            max_inbound_peers: 1,
            max_outbound_peers: 1,
            ..NetworkConfig::default()
        };

        let mut net = P2PNetwork::new(config);

        net.register_peer(test_peer())
            .expect("first peer should register");

        let second_peer = Peer::new(
            "node-2",
            "10.0.0.2:2727",
            "AOXC-MAINNET",
            ExternalDomainKind::Native,
            PeerRole::Validator,
            3,
            true,
            NodeCertificate {
                subject: "node-2".to_string(),
                issuer: "AOXC-ROOT".to_string(),
                valid_from_unix: 1,
                valid_until_unix: u64::MAX,
                serial: "serial-2".to_string(),
                domain_attestation_hash: "attestation-hash-2".to_string(),
            },
        );

        net.register_peer(second_peer)
            .expect("second peer should register inside aggregate capacity");

        let third_peer = Peer::new(
            "node-3",
            "10.0.0.3:2727",
            "AOXC-MAINNET",
            ExternalDomainKind::Native,
            PeerRole::Validator,
            3,
            true,
            NodeCertificate {
                subject: "node-3".to_string(),
                issuer: "AOXC-ROOT".to_string(),
                valid_from_unix: 1,
                valid_until_unix: u64::MAX,
                serial: "serial-3".to_string(),
                domain_attestation_hash: "attestation-hash-3".to_string(),
            },
        );

        let err = net
            .register_peer(third_peer)
            .expect_err("aggregate capacity overflow must be rejected");

        assert!(matches!(err, NetworkError::PeerAdmissionDenied(_)));
    }

    #[test]
    fn oversize_frame_is_rejected() {
        let config = NetworkConfig {
            max_frame_bytes: 64,
            ..NetworkConfig::default()
        };

        let mut net = P2PNetwork::new(config);

        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("oversize frame must be rejected");

        assert!(matches!(err, NetworkError::FrameTooLarge));
    }

    #[test]
    fn payload_hash_is_deterministic() {
        let hash_a = digest_payload(&test_vote()).expect("hashing must succeed");
        let hash_b = digest_payload(&test_vote()).expect("hashing must succeed");
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn metrics_are_updated_on_successful_broadcast_and_receive() {
        let mut net = P2PNetwork::new(NetworkConfig::default());

        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        net.broadcast_secure("node-1", test_vote())
            .expect("broadcast should succeed");
        let _ = net.receive();

        let snapshot = net.metrics_snapshot();
        assert_eq!(snapshot.accepted_peers, 1);
        assert_eq!(snapshot.active_sessions, 1);
        assert_eq!(snapshot.frames_out, 1);
        assert_eq!(snapshot.frames_in, 1);
        assert_eq!(snapshot.gossip_messages, 1);
        assert!(snapshot.bytes_out > 0);
        assert!(snapshot.bytes_in > 0);
    }

    #[test]
    fn broadcast_rejects_when_inbound_queue_capacity_is_exhausted() {
        let config = NetworkConfig {
            max_sync_batch: 1,
            ..NetworkConfig::default()
        };
        let mut net = P2PNetwork::new(config);

        net.register_peer(test_peer())
            .expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        net.broadcast_secure("node-1", test_vote())
            .expect("first broadcast should fit inbound queue");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("second broadcast should exceed inbound queue limit");
        assert!(matches!(err, NetworkError::TransportUnavailable(_)));
    }

    #[test]
    fn protocol_envelope_rejects_empty_chain_id() {
        let ticket = SessionTicket {
            peer_id: "node-1".to_string(),
            cert_fingerprint: "fp".to_string(),
            established_at_unix: 1,
            replay_window_nonce: 7,
            session_id: "session-1".to_string(),
            expires_at_unix: u64::MAX,
        };

        let err = ProtocolEnvelope::new("", 2626, &ticket, test_vote(), 1)
            .expect_err("empty chain id must be rejected");

        assert!(matches!(err, NetworkError::ProtocolMismatch(_)));
    }

    #[test]
    fn protocol_envelope_detects_payload_tampering() {
        let ticket = SessionTicket {
            peer_id: "node-1".to_string(),
            cert_fingerprint: "fp".to_string(),
            established_at_unix: 1,
            replay_window_nonce: 7,
            session_id: "session-1".to_string(),
            expires_at_unix: u64::MAX,
        };

        let mut envelope = ProtocolEnvelope::new("AOXC-MAINNET", 2626, &ticket, test_vote(), 1)
            .expect("envelope should be created");

        envelope.payload_hash_hex = "deadbeef".to_string();

        let err = envelope
            .verify_against("AOXC-MAINNET", 2626)
            .expect_err("tampered envelope must be rejected");

        assert!(matches!(err, NetworkError::ProtocolMismatch(_)));
    }
}
