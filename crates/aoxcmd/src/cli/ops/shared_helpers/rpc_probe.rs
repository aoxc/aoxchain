use super::*;

pub(in crate::cli::ops) fn uptime_secs_from_rfc3339(value: &str) -> u64 {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .and_then(|time| {
            let elapsed = Utc::now().signed_duration_since(time.with_timezone(&Utc));
            (elapsed.num_seconds() >= 0).then_some(elapsed.num_seconds() as u64)
        })
        .unwrap_or(0)
}

pub(in crate::cli::ops) fn rpc_listener_active(probe_target: &str) -> bool {
    match probe_target.parse() {
        Ok(addr) => TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok(),
        Err(_) => false,
    }
}

pub(in crate::cli::ops) fn rpc_http_get_probe(host: &str, port: u16, path: &str) -> bool {
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    rpc_http_status_code(host, port, &request)
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false)
}

pub(in crate::cli::ops) fn rpc_jsonrpc_status_probe(host: &str, port: u16) -> bool {
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"status","params":[]}"#;
    let request = format!(
        "POST / HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    rpc_http_status_code(host, port, &request)
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false)
}

pub(in crate::cli::ops) fn rpc_http_status_code(
    host: &str,
    port: u16,
    request: &str,
) -> Option<u16> {
    let target = format!("{host}:{port}");
    let addr = target.to_socket_addrs().ok()?.next()?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(350)).ok()?;
    if stream
        .set_read_timeout(Some(Duration::from_millis(350)))
        .is_err()
    {
        return None;
    }
    if stream
        .set_write_timeout(Some(Duration::from_millis(350)))
        .is_err()
    {
        return None;
    }
    if stream.write_all(request.as_bytes()).is_err() {
        return None;
    }
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    if reader.read_line(&mut status_line).ok()? == 0 {
        return None;
    }
    let mut parts = status_line.split_whitespace();
    let _http_version = parts.next()?;
    parts.next()?.parse::<u16>().ok()
}
