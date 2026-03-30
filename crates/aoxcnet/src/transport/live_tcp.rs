// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LiveTcpSmokeReport {
    pub listener_addr: SocketAddr,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub payload_echoed: bool,
    pub round_trip_ms: u128,
}

pub fn run_live_tcp_smoke(payload: &[u8], timeout: Duration) -> Result<LiveTcpSmokeReport, String> {
    run_live_tcp_smoke_on("127.0.0.1:0", payload, timeout)
}

pub fn run_live_tcp_smoke_on(
    bind_addr: &str,
    payload: &[u8],
    timeout: Duration,
) -> Result<LiveTcpSmokeReport, String> {
    if payload.is_empty() {
        return Err("payload must not be empty".to_string());
    }

    let listener =
        TcpListener::bind(bind_addr).map_err(|error| format!("tcp bind failed: {error}"))?;
    let listener_addr = listener
        .local_addr()
        .map_err(|error| format!("failed to read listener address: {error}"))?;
    listener
        .set_nonblocking(false)
        .map_err(|error| format!("failed to configure listener mode: {error}"))?;

    let expected = payload.to_vec();
    let server = thread::spawn(move || -> Result<usize, String> {
        let (mut stream, _) = listener
            .accept()
            .map_err(|error| format!("tcp accept failed: {error}"))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|error| format!("server read timeout failed: {error}"))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|error| format!("server write timeout failed: {error}"))?;

        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .map_err(|error| format!("server read frame length failed: {error}"))?;
        let frame_len = u32::from_be_bytes(len_buf) as usize;
        let mut recv = vec![0u8; frame_len];
        stream
            .read_exact(&mut recv)
            .map_err(|error| format!("server read payload failed: {error}"))?;
        if recv != expected {
            return Err("server payload mismatch".to_string());
        }
        stream
            .write_all(&(frame_len as u32).to_be_bytes())
            .map_err(|error| format!("server write frame length failed: {error}"))?;
        stream
            .write_all(&recv)
            .map_err(|error| format!("server write echo payload failed: {error}"))?;
        Ok(frame_len)
    });

    let started = Instant::now();
    let mut client = TcpStream::connect(listener_addr)
        .map_err(|error| format!("tcp connect failed: {error}"))?;
    client
        .set_read_timeout(Some(timeout))
        .map_err(|error| format!("client read timeout failed: {error}"))?;
    client
        .set_write_timeout(Some(timeout))
        .map_err(|error| format!("client write timeout failed: {error}"))?;
    client
        .write_all(&(payload.len() as u32).to_be_bytes())
        .map_err(|error| format!("client write frame length failed: {error}"))?;
    client
        .write_all(payload)
        .map_err(|error| format!("client write payload failed: {error}"))?;

    let mut echoed_len = [0u8; 4];
    client
        .read_exact(&mut echoed_len)
        .map_err(|error| format!("client read echo length failed: {error}"))?;
    let echoed_size = u32::from_be_bytes(echoed_len) as usize;
    let mut echoed_payload = vec![0u8; echoed_size];
    client
        .read_exact(&mut echoed_payload)
        .map_err(|error| format!("client read echo payload failed: {error}"))?;

    let bytes_received = server
        .join()
        .map_err(|_| "server thread panicked".to_string())??;
    Ok(LiveTcpSmokeReport {
        listener_addr,
        bytes_sent: payload.len(),
        bytes_received,
        payload_echoed: echoed_payload == payload,
        round_trip_ms: started.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::{run_live_tcp_smoke, run_live_tcp_smoke_on};
    use std::time::Duration;

    #[test]
    fn live_tcp_smoke_echoes_payload() {
        let report = run_live_tcp_smoke(b"aoxc-live-network", Duration::from_secs(2))
            .expect("live tcp smoke should succeed");
        assert!(report.payload_echoed);
        assert_eq!(report.bytes_sent, report.bytes_received);
    }

    #[test]
    fn live_tcp_smoke_on_explicit_port() {
        let report =
            run_live_tcp_smoke_on("127.0.0.1:3636", b"aoxc-port-3636", Duration::from_secs(2))
                .expect("live tcp smoke should succeed on explicit port");
        assert_eq!(report.listener_addr.port(), 3636);
        assert!(report.payload_echoed);
    }
}
