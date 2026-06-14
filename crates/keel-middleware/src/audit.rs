//! keel-middleware::audit — I1, observability (canon §5).
//!
//! Emits one structured `AuditEvent` for **every** call through the chain — success or failure —
//! behind a pluggable `AuditSink` (an in-memory `Vec` for tests; the file ledger, I2, in an app).
//! Because it rides the chain like any middleware, no call to a tier escapes it.

use async_trait::async_trait;
use keel_contracts::{Context, GenerateRequest, GenerateResult, Middleware, Next, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// One audited call. Serializable so a file sink can append it as JSONL (the ledger).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub trace_id: String,
    pub t_utc: i64,
    pub model: String,
    pub tier: String,
    pub cost: f64,
    pub ok: bool,
    /// `"OK"` on success, else the `KeelError` code (canon §18).
    pub code: String,
}

/// Where audit events go. Append-only; the file ledger is the system of record (canon §11, §13).
pub trait AuditSink: Send + Sync {
    fn emit(&self, event: &AuditEvent);
}

/// I1 middleware: records a structured event per call, around the rest of the chain.
pub struct AuditMiddleware {
    sink: Arc<dyn AuditSink>,
}

impl AuditMiddleware {
    pub fn new(sink: Arc<dyn AuditSink>) -> Self {
        Self { sink }
    }
}

#[async_trait]
impl Middleware for AuditMiddleware {
    async fn handle(&self, req: GenerateRequest, ctx: &Context, next: &dyn Next) -> Result<GenerateResult> {
        let trace_id = ctx.trace_id.clone();
        let model = req.model.clone(); // req is moved into next; capture what we report on error
        let res = next.run(req, ctx).await;
        let event = match &res {
            Ok(r) => AuditEvent {
                trace_id,
                t_utc: keel_kernel::now_millis(),
                model,
                tier: r.tier.clone(),
                cost: r.cost,
                ok: true,
                code: "OK".to_string(),
            },
            Err(e) => AuditEvent {
                trace_id,
                t_utc: keel_kernel::now_millis(),
                model,
                tier: String::new(),
                cost: 0.0,
                ok: false,
                code: e.code().to_string(),
            },
        };
        self.sink.emit(&event);
        res
    }
}

/// An append-only JSONL `AuditSink` — the file ledger (canon §11, §13). Best-effort: a write
/// failure goes to stderr and never crashes the call (a hash-chained durable sink swaps in later).
pub struct FileAuditSink {
    file: std::sync::Mutex<std::fs::File>,
}

impl FileAuditSink {
    /// Open the audit ledger at `path` in append mode, creating parent dirs + the file.
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir)?;
            }
        }
        let file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file: std::sync::Mutex::new(file) })
    }
}

impl AuditSink for FileAuditSink {
    fn emit(&self, event: &AuditEvent) {
        use std::io::Write;
        let Ok(line) = serde_json::to_string(event) else {
            eprintln!("[keel] audit serialize failed");
            return;
        };
        if let Ok(mut f) = self.file.lock() {
            if let Err(e) = writeln!(f, "{line}").and_then(|()| f.flush()) {
                eprintln!("[keel] audit write failed: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuditEvent, AuditMiddleware, AuditSink};
    use crate::cost::CostMiddleware;
    use async_trait::async_trait;
    use keel_contracts::{Capabilities, Context, GenerateRequest, GenerateResult, ModelTier, Result};
    use keel_kernel::Chain;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct VecSink {
        events: Mutex<Vec<AuditEvent>>,
    }
    impl AuditSink for VecSink {
        fn emit(&self, event: &AuditEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    /// A metered terminal — returns a tier/cost the audit event should capture.
    struct PaidTier;
    #[async_trait]
    impl ModelTier for PaidTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, _req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            Ok(GenerateResult { content: "ok".into(), tier: "cheap-API".into(), cost: 0.0123, ..Default::default() })
        }
    }

    fn req() -> GenerateRequest {
        GenerateRequest {
            messages: vec![],
            model: "deepseek-v4-pro".into(),
            tools: vec![],
            grammar: None,
            effort: Default::default(),
            cache_prefix_len: None,
        }
    }

    #[tokio::test]
    async fn emits_one_event_on_success() {
        let sink = Arc::new(VecSink::default());
        let chain = Chain::new(vec![Arc::new(AuditMiddleware::new(sink.clone()))]);
        let res = chain.run(req(), &Context::default(), Arc::new(PaidTier)).await;
        assert!(res.is_ok());

        let events = sink.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        let e = &events[0];
        assert!(e.ok);
        assert_eq!(e.code, "OK");
        assert_eq!(e.tier, "cheap-API");
        assert_eq!(e.model, "deepseek-v4-pro");
        assert_eq!(e.cost, 0.0123);
        assert!(e.t_utc > 0);
    }

    #[tokio::test]
    async fn emits_event_even_when_a_downstream_gate_blocks() {
        // audit (outer) wraps a cost gate that blocks — I1 must still record the blocked call.
        let sink = Arc::new(VecSink::default());
        let chain = Chain::new(vec![
            Arc::new(AuditMiddleware::new(sink.clone())),
            Arc::new(CostMiddleware::new(1.0)),
        ]);
        let mut ctx = Context { task_budget: Some(5.0), ..Default::default() };
        ctx.cost.add("cheap-API", 4.5); // remaining 0.5 < 1.0 → the gate blocks

        let res = chain.run(req(), &ctx, Arc::new(PaidTier)).await;
        assert!(res.is_err());

        let events = sink.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(!events[0].ok);
        assert_eq!(events[0].code, "BUDGET_EXCEEDED");
    }
}
