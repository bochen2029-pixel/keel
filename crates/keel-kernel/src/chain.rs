//! keel-kernel::chain — the middleware executor (canon §6, §14).
//!
//! The onion that threads every call through the registered `Middleware`s, in order, down to a
//! terminal `ModelTier::generate`. Because the chain wraps **every** call, the cross-cutting
//! invariants that live in middleware — I1 audit, I3 privacy, I4 cost — become structurally
//! unbypassable: there is no path to a tier that does not pass through here.
//!
//! `Middleware::handle` receives `&Context` (it observes, gates, and transforms the *request*);
//! it cannot mutate the context. Accumulation that must persist across a run (cost totals,
//! redaction state) is therefore the engine's job — the engine owns the `Context` and folds in
//! `result.cost` after the chain returns. This keeps the frozen contract honest.

use keel_contracts::{Context, GenerateRequest, GenerateResult, Middleware, ModelTier, Next, Result};
use std::sync::Arc;

/// An ordered stack of middleware, run around a terminal tier. Cheap to clone (shared `Arc`).
#[derive(Clone, Default)]
pub struct Chain {
    middleware: Arc<Vec<Arc<dyn Middleware>>>,
}

impl Chain {
    /// Build a chain from middleware in **outermost-first** order: index 0 sees the call first
    /// (and the response last).
    pub fn new(middleware: Vec<Arc<dyn Middleware>>) -> Self {
        Self { middleware: Arc::new(middleware) }
    }

    /// Number of middleware layers.
    pub fn len(&self) -> usize {
        self.middleware.len()
    }

    pub fn is_empty(&self) -> bool {
        self.middleware.is_empty()
    }

    /// Run a request through the full stack, terminating at `terminal.generate`.
    pub async fn run(
        &self,
        req: GenerateRequest,
        ctx: &Context,
        terminal: Arc<dyn ModelTier>,
    ) -> Result<GenerateResult> {
        let link = Link { chain: self.middleware.clone(), idx: 0, terminal };
        link.run(req, ctx).await
    }
}

/// One position in the onion: middleware `idx`, or the terminal once `idx` runs off the end.
/// Fully owned (`Arc` clones), so the recursive `&dyn Next` borrow has no lifetime tangles.
struct Link {
    chain: Arc<Vec<Arc<dyn Middleware>>>,
    idx: usize,
    terminal: Arc<dyn ModelTier>,
}

#[async_trait::async_trait]
impl Next for Link {
    async fn run(&self, req: GenerateRequest, ctx: &Context) -> Result<GenerateResult> {
        match self.chain.get(self.idx) {
            Some(mw) => {
                let next = Link {
                    chain: self.chain.clone(),
                    idx: self.idx + 1,
                    terminal: self.terminal.clone(),
                };
                mw.handle(req, ctx, &next).await
            }
            None => self.terminal.generate(req, ctx).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use keel_contracts::{Capabilities, KeelError};
    use std::sync::Mutex;

    fn req() -> GenerateRequest {
        GenerateRequest {
            messages: vec![],
            model: "m".into(),
            tools: vec![],
            grammar: None,
            effort: Default::default(),
            cache_prefix_len: None,
        }
    }

    /// Terminal that records the model it was asked for and echoes it back.
    struct EchoTier {
        seen: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl ModelTier for EchoTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            self.seen.lock().unwrap().push(format!("terminal:{}", req.model));
            Ok(GenerateResult { content: req.model.clone(), tier: "test".into(), model: req.model, ..Default::default() })
        }
    }

    /// Middleware that logs around the inner call — proves onion order.
    struct Recorder {
        label: String,
        log: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl Middleware for Recorder {
        async fn handle(&self, req: GenerateRequest, ctx: &Context, next: &dyn Next) -> Result<GenerateResult> {
            self.log.lock().unwrap().push(format!("{}:pre", self.label));
            let res = next.run(req, ctx).await;
            self.log.lock().unwrap().push(format!("{}:post", self.label));
            res
        }
    }

    #[tokio::test]
    async fn onion_order_and_terminal() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let chain = Chain::new(vec![
            Arc::new(Recorder { label: "A".into(), log: log.clone() }),
            Arc::new(Recorder { label: "B".into(), log: log.clone() }),
        ]);
        let res = chain.run(req(), &Context::default(), Arc::new(EchoTier { seen: seen.clone() })).await.unwrap();
        assert_eq!(res.model, "m");
        assert_eq!(*log.lock().unwrap(), vec!["A:pre", "B:pre", "B:post", "A:post"]);
        assert_eq!(*seen.lock().unwrap(), vec!["terminal:m"]);
    }

    /// A middleware that returns without calling `next` — the terminal must never run.
    struct Block;
    #[async_trait]
    impl Middleware for Block {
        async fn handle(&self, _req: GenerateRequest, _ctx: &Context, _next: &dyn Next) -> Result<GenerateResult> {
            Err(KeelError::BudgetExceeded("blocked".into()))
        }
    }

    #[tokio::test]
    async fn middleware_can_short_circuit() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let chain = Chain::new(vec![Arc::new(Block)]);
        let res = chain.run(req(), &Context::default(), Arc::new(EchoTier { seen: seen.clone() })).await;
        assert!(matches!(res, Err(KeelError::BudgetExceeded(_))));
        assert!(seen.lock().unwrap().is_empty()); // terminal unreachable past the gate
    }

    #[tokio::test]
    async fn empty_chain_hits_terminal() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let chain = Chain::new(vec![]);
        assert!(chain.is_empty());
        let res = chain.run(req(), &Context::default(), Arc::new(EchoTier { seen: seen.clone() })).await.unwrap();
        assert_eq!(res.content, "m");
        assert_eq!(*seen.lock().unwrap(), vec!["terminal:m"]);
    }

    /// A middleware that rewrites the request — the transform must reach the terminal.
    struct Retarget;
    #[async_trait]
    impl Middleware for Retarget {
        async fn handle(&self, mut req: GenerateRequest, ctx: &Context, next: &dyn Next) -> Result<GenerateResult> {
            req.model = "rewritten".into();
            next.run(req, ctx).await
        }
    }

    #[tokio::test]
    async fn request_transform_reaches_terminal() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let chain = Chain::new(vec![Arc::new(Retarget)]);
        let res = chain.run(req(), &Context::default(), Arc::new(EchoTier { seen: seen.clone() })).await.unwrap();
        assert_eq!(res.model, "rewritten");
        assert_eq!(*seen.lock().unwrap(), vec!["terminal:rewritten"]);
    }
}
