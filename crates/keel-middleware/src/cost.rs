//! keel-middleware::cost — I4, the budget hard-stop gate (canon §5, §9).
//!
//! Reads the run's remaining budget from the `Context` and **blocks the call before it reaches a
//! tier** (returns `BudgetExceeded`) when remaining sits under the hard-stop floor. It is a
//! *gate*, not an accumulator: the frozen `Middleware::handle` sees `&Context`, so totalling
//! spent cost stays the engine's job (it owns the `Context` and folds in `result.cost` after the
//! chain returns). Pre-call gate + post-call accumulate is the I4 split the chain's `&Context`
//! contract implies.

use async_trait::async_trait;
use keel_contracts::{Context, GenerateRequest, GenerateResult, KeelError, Middleware, Next, Result};

/// The I4 budget gate. Construct with the hard-stop floor (canonically `CostCfg.hard_stop_at`).
pub struct CostMiddleware {
    hard_stop_at: f64,
}

impl CostMiddleware {
    /// Block a call when the run's remaining budget would fall below `hard_stop_at` dollars.
    pub fn new(hard_stop_at: f64) -> Self {
        Self { hard_stop_at }
    }
}

#[async_trait]
impl Middleware for CostMiddleware {
    async fn handle(&self, req: GenerateRequest, ctx: &Context, next: &dyn Next) -> Result<GenerateResult> {
        // Unbudgeted runs (no task_budget) are not gated; budgeted runs hard-stop at the floor.
        if let Some(remaining) = ctx.budget_remaining() {
            if remaining < self.hard_stop_at {
                return Err(KeelError::BudgetExceeded(format!(
                    "remaining ${:.4} below hard-stop ${:.4}",
                    remaining, self.hard_stop_at
                )));
            }
        }
        next.run(req, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::CostMiddleware;
    use async_trait::async_trait;
    use keel_contracts::{
        Capabilities, Context, GenerateRequest, GenerateResult, KeelError, ModelTier, Result,
    };
    use keel_kernel::Chain;
    use std::sync::{Arc, Mutex};

    /// Terminal that counts how many times it was actually reached.
    struct CountTier {
        calls: Arc<Mutex<u32>>,
    }
    #[async_trait]
    impl ModelTier for CountTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, _req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            *self.calls.lock().unwrap() += 1;
            Ok(GenerateResult { content: "ok".into(), ..Default::default() })
        }
    }

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

    fn ctx_with(budget: f64, spent: f64) -> Context {
        let mut c = Context { task_budget: Some(budget), ..Default::default() };
        c.cost.add("local", spent);
        c
    }

    async fn run(ctx: &Context, hard_stop: f64) -> (Result<GenerateResult>, u32) {
        let calls = Arc::new(Mutex::new(0));
        let chain = Chain::new(vec![Arc::new(CostMiddleware::new(hard_stop))]);
        let res = chain.run(req(), ctx, Arc::new(CountTier { calls: calls.clone() })).await;
        let n = *calls.lock().unwrap();
        (res, n)
    }

    #[tokio::test]
    async fn passes_when_budget_healthy() {
        let (res, calls) = run(&ctx_with(5.0, 0.5), 1.0).await; // remaining 4.5 ≥ 1.0
        assert!(res.is_ok());
        assert_eq!(calls, 1); // terminal reached
    }

    #[tokio::test]
    async fn blocks_at_the_floor_before_calling_the_tier() {
        let (res, calls) = run(&ctx_with(5.0, 4.5), 1.0).await; // remaining 0.5 < 1.0
        assert!(matches!(res, Err(KeelError::BudgetExceeded(_))));
        assert_eq!(calls, 0); // the tier is never reached — I4 is unbypassable
    }

    #[tokio::test]
    async fn unbudgeted_run_is_not_gated() {
        let (res, calls) = run(&Context::default(), 1.0).await; // task_budget None
        assert!(res.is_ok());
        assert_eq!(calls, 1);
    }
}
