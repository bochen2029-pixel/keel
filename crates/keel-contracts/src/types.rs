//! KEEL L0 — core types. Frozen with the traits; the genome's data surface.
//!
//! Contracts stay clock-free and side-effect-free: time is an `i64` the kernel stamps,
//! opaque state is JSON. Multimodal `Content` carries the eyes/ears (canon §10, §12).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Arbitrary JSON (tool args, opaque artifacts, golden input/expect, run-state).
pub type Json = serde_json::Value;
/// Unix epoch milliseconds. The kernel stamps time; contracts never read a clock.
pub type Time = i64;
/// Resumable-run identifier (I2).
pub type RunId = String;
/// Opaque, serializable per-step state for the run-state spine (I2).
pub type State = Json;

// ── messages & multimodal content ──────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A message part. Vision rides the cognition protocol (canon §3, §12); raw `Image`/`Clip`/
/// `Audio` are sovereign-by-default (I3) and force the local tier.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    Text { text: String },
    Image { source: String },                 // path or data-uri
    Clip { video: String, frames: String },    // video ref + frame spec
    Audio { source: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<Content>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// DeepSeek: MUST be replayed across tool turns or the provider 400s.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

// ── routing primitives ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Scaffolding,
    CoreWire,
}

/// Trust as a first-class routable quantity (canon §4). Ordered Low < Normal < High < Critical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trust {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClass {
    Normal,
    Sovereign,
    Phi,
}

/// The amplification dial (canon §4): best-of-N width × thinking depth.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effort {
    pub n: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>, // "low" | "high" | "max"
}
impl Default for Effort {
    fn default() -> Self {
        Self { n: 1, thinking: None }
    }
}

// ── usage, price, cost ───────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// DeepSeek prompt_cache_hit_tokens — the 100× lever.
    pub cache_hit_tokens: u64,
}

/// USD per 1,000,000 tokens.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Price {
    pub input_miss: f64,
    pub input_hit: f64,
    pub output: f64,
}

/// Cost from usage + tier price. Cache-miss input = total input minus cache hits.
/// Pinned by GOLDEN_MODEL_TIER (the language-neutral conformance layer).
pub fn compute_cost(u: &Usage, p: &Price) -> f64 {
    let miss = u.input_tokens.saturating_sub(u.cache_hit_tokens) as f64;
    (miss * p.input_miss + u.cache_hit_tokens as f64 * p.input_hit + u.output_tokens as f64 * p.output)
        / 1_000_000.0
}

/// What a tier can do; the router and egress filter read this.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Capabilities {
    pub vision: bool,
    pub video: bool,
    pub thinking: bool,
}

// ── generate request/result ──────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String, // "function"
    pub name: String,
    pub arguments: String, // JSON string, as the provider returns it
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub schema: Json,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub messages: Vec<Message>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDef>,
    /// GBNF string or JSON-schema; the local tier enforces at decode time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grammar: Option<Json>,
    #[serde(default)]
    pub effort: Effort,
    /// Bytes of stable prefix to mark cacheable (cache-prefix discipline).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_prefix_len: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GenerateResult {
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub usage: Usage,
    pub cost: f64,
    pub tier: String,
    pub model: String,
}

// ── steps & decisions ────────────────────────────────────────────────────────

fn default_trust() -> Trust {
    Trust::Normal
}
fn default_data_class() -> DataClass {
    DataClass::Normal
}

/// The atomic unit of routed work.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub kind: Kind,
    /// e.g. "tool_call:read_file", "reason:multi_constraint", "see", "hear".
    pub ty: String,
    #[serde(default = "default_trust")]
    pub trust_required: Trust,
    #[serde(default = "default_data_class")]
    pub data_class: DataClass,
    #[serde(default)]
    pub tier_history: Vec<String>,
    #[serde(default)]
    pub oracle_failures: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projected_cost: Option<f64>,
    #[serde(default)]
    pub critical: bool,
    /// Which Driver emitted it (user turn? heartbeat? watch?).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default)]
    pub content: Vec<Content>,
    #[serde(default)]
    pub golden_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Decision {
    /// A tier name, or "BLOCK".
    pub tier: String,
    #[serde(default)]
    pub effort: Effort,
    pub reason: String,
}

// ── oracle / externality (I5) ────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StepOutput {
    pub content: String,
    /// e.g. {"self_tests_pass": true, "golden_violated": true, "property": "no_ssn_pattern"}.
    #[serde(default)]
    pub artifact: Json,
}

/// A non-model assertion carried on a verdict (the externality ledger).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assertion {
    pub kind: String,
    pub detail: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Verdict {
    pub passed: bool,
    #[serde(default)]
    pub failures: Vec<String>,
    #[serde(default)]
    pub joint_wrong: bool,
    #[serde(default)]
    pub evidence: Vec<Assertion>,
}

/// Operator-authored, agent-frozen ground truth. Stored language-neutrally in
/// `tests/golden/golden.json`; this is its in-code shape.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoldenCase {
    pub name: String,
    pub input: Json,
    pub expect: Json,
}

// ── perception (eyes & ears) ─────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Image,
    Audio,
}

/// A timestamped, modality-tagged, source-attributed unit of perception.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Percept {
    pub content: Json,
    pub t_utc: Time,
    pub modality: Modality,
    /// Capture topology, not a model: e.g. "mic" / "loopback" / "screen".
    pub source: String,
    #[serde(default)]
    pub confidence: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SampleSpec {
    /// Pre-resize cap for any frame handed onward (guards token-capped consumers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_px: Option<u32>,
    /// Live capture cadence; archival sources may multi-pass instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_ms: Option<u64>,
}

// ── memory (the ringed, budgeted self) ───────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenBudget {
    pub total: u32,
    pub ring0: u32,
    pub ring1: u32,
    pub ring2: u32,
    pub ring2_max: u32,
    pub ring3: u32,
    pub ring4: u32,
    pub generation_reserve: u32,
}

/// Ring 0 soul · Ring 1 exemplars · Ring 2 working · Ring 3 compressed history · Ring 4 retrieved.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AssembledContext {
    pub system: String,
    #[serde(default)]
    pub exemplars: Vec<Message>,
    #[serde(default)]
    pub history: String,
    #[serde(default)]
    pub retrieved: Vec<String>,
    #[serde(default)]
    pub conversation: Vec<Message>,
    #[serde(default)]
    pub budget: TokenBudget,
}

// ── traces (the flywheel feed) ───────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trace {
    pub step: Step,
    pub decision: Decision,
    pub result: GenerateResult,
    pub verdict: Verdict,
}

/// A `Trace` whose `verdict.passed == true`. The feedstock for distillation; secrets
/// are scrubbed before a trace becomes feedstock (reversibility gate, canon §5).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedTrace {
    pub trace: Trace,
}

// ── the universal context ────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CostAcc {
    pub total: f64,
    #[serde(default)]
    pub by_tier: HashMap<String, f64>,
}
impl CostAcc {
    pub fn add(&mut self, tier: &str, amount: f64) {
        self.total += amount;
        *self.by_tier.entry(tier.to_string()).or_default() += amount;
    }
}

/// Flows through every call (I1/I3/I4 ride here). Defined at L0 because every contract
/// method receives it; the kernel populates it.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Context {
    pub trace_id: String,
    #[serde(default)]
    pub cost: CostAcc,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_budget: Option<f64>,
    /// I3 marker: "clean" | "sovereign" | "phi".
    #[serde(default)]
    pub redaction_state: String,
    #[serde(default)]
    pub golden_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<f64>,
}
impl Context {
    pub fn budget_remaining(&self) -> Option<f64> {
        self.task_budget.map(|b| b - self.cost.total)
    }
}
