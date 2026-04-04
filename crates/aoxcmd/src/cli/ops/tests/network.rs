use super::*;

#[test]
fn shifted_ports_require_same_delta_across_profiles() {
    let mainnet_profile = super::super::types::NetworkProfileConfig {
        chain_id: "aox-mainnet-1".to_string(),
        listen_addr: "0.0.0.0:26656".to_string(),
        rpc_addr: "0.0.0.0:8545".to_string(),
        peers: vec!["seed-1".to_string(), "seed-2".to_string()],
        security_mode: "audit_strict".to_string(),
    };
    let testnet_profile = super::super::types::NetworkProfileConfig {
        chain_id: "aox-testnet-1".to_string(),
        listen_addr: "0.0.0.0:36656".to_string(),
        rpc_addr: "0.0.0.0:18545".to_string(),
        peers: vec!["seed-1".to_string(), "seed-2".to_string()],
        security_mode: "audit_strict".to_string(),
    };

    assert!(ports_are_shifted_consistently(
        &mainnet_profile,
        &testnet_profile
    ));
}

#[test]
fn rpc_http_get_probe_reports_success_for_200_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let port = listener
        .local_addr()
        .expect("listener should expose local addr")
        .port();
    let server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
            );
        }
    });

    assert!(rpc_http_get_probe("127.0.0.1", port, "/health"));
    let _ = server.join();
}

#[test]
fn rpc_jsonrpc_status_probe_reports_success_for_200_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let port = listener
        .local_addr()
        .expect("listener should expose local addr")
        .port();
    let server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 36\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}",
                );
        }
    });

    assert!(rpc_jsonrpc_status_probe("127.0.0.1", port));
    let _ = server.join();
}
