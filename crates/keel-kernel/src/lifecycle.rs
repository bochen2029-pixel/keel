//! kernel::lifecycle — the substrate resolver (canon §13).
//!
//! Stage-0 rung **(c1): use-whatever-exists** — probe the candidate endpoints and route to the
//! first inference server already serving. The probe is a dependency-free TCP-connect liveness
//! check (a full `/v1/models` verification is a refinement; the launch path will do a proper
//! `/health` poll). **(c2) launch + supervise** llama-server (subprocess, restart) lands next.

use keel_contracts::{KeelError, Result};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// The canonical local probe order (canon §13 / `keel.lock` resolver_order): LM Studio · Ollama ·
/// llama-server. Base URLs (no path) — the adapter appends `/v1/chat/completions`.
pub fn default_local_candidates() -> Vec<String> {
    vec![
        "http://127.0.0.1:1234".to_string(),  // LM Studio
        "http://127.0.0.1:11434".to_string(), // Ollama
        "http://127.0.0.1:8080".to_string(),  // llama-server
    ]
}

/// Is something listening at this endpoint? A TCP connect to its host:port.
pub fn probe(endpoint: &str, timeout: Duration) -> bool {
    match socket_addr(endpoint) {
        Some(addr) => TcpStream::connect_timeout(&addr, timeout).is_ok(),
        None => false,
    }
}

/// Resolve the first live endpoint from `candidates` (base URL, no trailing slash). Returns
/// `SUBSTRATE_UNRESOLVED` if none respond.
pub fn resolve_endpoint(candidates: &[String]) -> Result<String> {
    let timeout = Duration::from_millis(300);
    for c in candidates {
        if probe(c, timeout) {
            return Ok(c.trim_end_matches('/').to_string());
        }
    }
    Err(KeelError::SubstrateUnresolved(format!("no inference server reachable among {candidates:?}")))
}

/// Parse `http://host:port[/path]` → the first resolved socket address.
fn socket_addr(endpoint: &str) -> Option<std::net::SocketAddr> {
    let after_scheme = endpoint.split("://").nth(1).unwrap_or(endpoint);
    let host_port = after_scheme.split('/').next().unwrap_or(after_scheme);
    host_port.to_socket_addrs().ok()?.next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// A port nothing is listening on (bind to get one, then drop the listener).
    fn free_port() -> u16 {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let p = l.local_addr().unwrap().port();
        drop(l);
        p
    }

    #[test]
    fn probe_detects_a_listener() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(probe(&format!("http://127.0.0.1:{port}/v1"), Duration::from_millis(300)));
    }

    #[test]
    fn probe_misses_a_dead_port() {
        assert!(!probe(&format!("http://127.0.0.1:{}", free_port()), Duration::from_millis(200)));
    }

    #[test]
    fn resolve_picks_the_first_live_candidate() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let live = format!("http://127.0.0.1:{port}");
        let cands = vec![format!("http://127.0.0.1:{}", free_port()), live.clone()];
        assert_eq!(resolve_endpoint(&cands).unwrap(), live);
    }

    #[test]
    fn resolve_errors_when_nothing_is_up() {
        let cands = vec![format!("http://127.0.0.1:{}", free_port())];
        let err = resolve_endpoint(&cands).unwrap_err();
        assert_eq!(err.code(), "SUBSTRATE_UNRESOLVED");
    }
}
