//! kernel::lifecycle — the substrate resolver (canon §13). Dependency-free (std only), so the
//! kernel stays tiny.
//!
//! - **(c1) use-whatever-exists** — `probe`/`resolve_endpoint`: a TCP-connect liveness check that
//!   routes to the first server already serving.
//! - **(c2) launch + supervise** — `launch`: spawn llama-server, poll `/health` until the model is
//!   loaded (a real HTTP GET distinguishes *ready* 200 from *loading* 503), and hand back a
//!   `LlamaServer` handle. Dropping the handle does **not** kill the process — a one-shot CLI
//!   leaves it running for reuse; a long-lived host calls `kill`/`is_alive` to supervise it.

use keel_contracts::{KeelError, Result};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

// ── (c1) probe / resolve ─────────────────────────────────────────────────────

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

// ── (c2) launch / supervise ──────────────────────────────────────────────────

/// How to launch llama-server. The caller (app layer) fills the paths from `keel.lock`.
#[derive(Clone, Debug)]
pub struct LlamaServerConfig {
    pub exe: String,
    pub model: String,
    pub mmproj: Option<String>,
    pub ctx_size: u32,
    pub n_gpu_layers: u32,
    pub host: String,
    pub port: u16,
    /// Where to send the server's stdout/stderr (None ⇒ discard).
    pub log_path: Option<String>,
    pub startup_timeout_secs: u64,
}

impl LlamaServerConfig {
    pub fn new(exe: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            exe: exe.into(),
            model: model.into(),
            mmproj: None,
            ctx_size: 16384,
            n_gpu_layers: 99,
            host: "127.0.0.1".to_string(),
            port: 8080,
            log_path: None,
            startup_timeout_secs: 90,
        }
    }
}

/// A launched llama-server subprocess. **Dropping the handle does not kill the process** (so a
/// one-shot CLI can leave it running for reuse); call `kill` to stop it, `is_alive` to supervise.
pub struct LlamaServer {
    child: Child,
    endpoint: String,
}

impl LlamaServer {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
    /// Is the subprocess still running?
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
    /// Stop the subprocess.
    pub fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }
}

/// Spawn llama-server and block until `/health` reports ready (the model is loaded), or fail with
/// `SUBSTRATE_UNRESOLVED` if it dies on startup or doesn't come up within the timeout.
pub fn launch(cfg: &LlamaServerConfig) -> Result<LlamaServer> {
    let mut cmd = Command::new(&cfg.exe);
    cmd.arg("--model").arg(&cfg.model)
        .arg("--ctx-size").arg(cfg.ctx_size.to_string())
        .arg("--n-gpu-layers").arg(cfg.n_gpu_layers.to_string())
        .arg("--host").arg(&cfg.host)
        .arg("--port").arg(cfg.port.to_string());
    if let Some(mmproj) = &cfg.mmproj {
        cmd.arg("--mmproj").arg(mmproj);
    }
    match &cfg.log_path {
        Some(p) => {
            if let Some(dir) = std::path::Path::new(p).parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let out = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)
                .map_err(|e| KeelError::SubstrateUnresolved(format!("server log {p}: {e}")))?;
            let err = out
                .try_clone()
                .map_err(|e| KeelError::SubstrateUnresolved(format!("server log clone: {e}")))?;
            cmd.stdout(out).stderr(err);
        }
        None => {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| KeelError::SubstrateUnresolved(format!("launch {}: {e}", cfg.exe)))?;
    let endpoint = format!("http://{}:{}", cfg.host, cfg.port);

    let start = Instant::now();
    let timeout = Duration::from_secs(cfg.startup_timeout_secs);
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(KeelError::SubstrateUnresolved(format!(
                "llama-server exited during startup ({status}); see {:?}",
                cfg.log_path
            )));
        }
        if http_health_ok(&cfg.host, cfg.port, Duration::from_millis(800)) {
            return Ok(LlamaServer { child, endpoint });
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err(KeelError::SubstrateUnresolved(format!(
                "llama-server not ready within {}s",
                cfg.startup_timeout_secs
            )));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// A minimal HTTP `GET /health` (dependency-free) — true only when the status line is `200`, so it
/// distinguishes a *ready* llama-server from one still loading the model (which returns `503`).
fn http_health_ok(host: &str, port: u16, timeout: Duration) -> bool {
    let Some(addr) = format!("{host}:{port}").to_socket_addrs().ok().and_then(|mut a| a.next()) else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let req = format!("GET /health HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0u8; 256];
    match stream.read(&mut buf) {
        Ok(n) if n > 0 => String::from_utf8_lossy(&buf[..n])
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .map(|code| code == "200")
            .unwrap_or(false),
        _ => false,
    }
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
        assert_eq!(resolve_endpoint(&cands).unwrap_err().code(), "SUBSTRATE_UNRESOLVED");
    }

    /// `/health` readiness parsing: 200 ⇒ ready, 503 (loading) ⇒ not ready. The test is its own
    /// one-shot HTTP server.
    #[test]
    fn health_ok_parses_200_and_rejects_503() {
        fn serve_once(status_line: &'static str) -> u16 {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = l.local_addr().unwrap().port();
            std::thread::spawn(move || {
                if let Ok((mut s, _)) = l.accept() {
                    let mut buf = [0u8; 256];
                    let _ = s.read(&mut buf);
                    let _ = s.write_all(format!("{status_line}\r\nContent-Length: 0\r\n\r\n").as_bytes());
                }
            });
            port
        }
        let ready = serve_once("HTTP/1.1 200 OK");
        assert!(http_health_ok("127.0.0.1", ready, Duration::from_millis(500)));
        let loading = serve_once("HTTP/1.1 503 Service Unavailable");
        assert!(!http_health_ok("127.0.0.1", loading, Duration::from_millis(500)));
    }
}
