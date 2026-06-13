//! KEEL L0 — the ten joints (canon §7). Frozen.
//!
//! Async traits use `#[async_trait]` so they stay **dyn-compatible** — the registry and the
//! middleware chain hold `dyn ModelTier` / `dyn Middleware`. `Router` is sync by design.

use crate::errors::Result;
use crate::types::*;
use async_trait::async_trait;
use core::pin::Pin;

/// Eyes/ears yield percepts as a stream (live capture is forward-only; archival sources
/// may multi-pass). Boxed for dyn use.
pub type PerceptStream = Pin<Box<dyn futures_core::stream::Stream<Item = Percept> + Send>>;

/// 1 — the uniform brain interface (cognition, incl. multimodal). Every adapter implements it.
#[async_trait]
pub trait ModelTier: Send + Sync {
    fn caps(&self) -> Capabilities;
    async fn generate(&self, req: GenerateRequest, ctx: &Context) -> Result<GenerateResult>;
}

/// 2 — tools/context/resources over MCP. KEEL is a client (and, in apps, a server).
#[async_trait]
pub trait ToolHost: Send + Sync {
    async fn list(&self) -> Result<Vec<ToolDef>>;
    async fn call(&self, name: &str, args: Json, ctx: &Context) -> Result<ToolResult>;
}

/// The rest-of-chain continuation handed to each middleware.
#[async_trait]
pub trait Next: Send + Sync {
    async fn run(&self, req: GenerateRequest, ctx: &Context) -> Result<GenerateResult>;
}

/// 3 — a cross-cutting concern on every call. I1 (audit), I3 (privacy), I4 (cost) live here,
/// which is what makes them structurally unbypassable.
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, req: GenerateRequest, ctx: &Context, next: &dyn Next) -> Result<GenerateResult>;
}

/// 4 — the fusion point: cheapest tier that clears the trust bar. Sync (a rules heuristic,
/// not a model call).
pub trait Router: Send + Sync {
    fn route(&self, step: &Step, ctx: &Context) -> Decision;
}

/// 5 — the externality surface (I5). A non-model assertion of correctness; flags JOINT_WRONG.
#[async_trait]
pub trait Oracle: Send + Sync {
    async fn verify(&self, output: &StepOutput, golden: &[GoldenCase], ctx: &Context) -> Result<Verdict>;
}

/// 6 — the self that persists: ringed/budgeted assembly, the lossless record, model-authored
/// consolidation (returns a maintenance `Step`). Stores plug in behind it.
#[async_trait]
pub trait Memory: Send + Sync {
    async fn assemble(&self, step: &Step, ctx: &Context) -> Result<AssembledContext>;
    async fn record(&self, trace: &Trace) -> Result<()>;
    async fn consolidate(&self) -> Result<Step>;
}

/// 7 — durable run-state (I2): resumable from checkpoint. The append-only ledger.
#[async_trait]
pub trait Spine: Send + Sync {
    async fn checkpoint(&self, run: &RunId, state: &State) -> Result<()>;
    async fn resume(&self, run: &RunId) -> Result<Option<State>>;
}

/// 8 — initiative: a source of work. The user-turn is one Driver; heartbeat/watch/outreach
/// are others. `None` = nothing to do now.
#[async_trait]
pub trait Driver: Send + Sync {
    async fn poll(&self, ctx: &Context) -> Result<Option<Step>>;
}

/// 9 — the flywheel feed: a verified trace becomes distillation feedstock.
#[async_trait]
pub trait TraceSink: Send + Sync {
    async fn emit(&self, trace: VerifiedTrace) -> Result<()>;
}

/// 10 — afferent senses (eyes + ears). Modality in → percepts out. Not a tier; the router
/// never routes to it.
pub trait PerceptionSource: Send + Sync {
    fn percepts(&self, spec: SampleSpec) -> PerceptStream;
}
