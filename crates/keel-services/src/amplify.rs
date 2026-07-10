//! keel-services::amplify — the §23 **amplification falsifier** bench (ISSUE-4) + its model-free
//! primitives. The amplify *loop step* itself lives in `kernel::engine` behind keel.lock
//! `router.amplify_n` (canon §8 `amplify?` — the loop is engine-internal; the layer rule keeps L4
//! out of the kernel), **shipping OFF**; this module carries the fixed benchmark that decides
//! whether it ever turns ON: *does verified best-of-N beat single-pass on a fixed set?* (canon
//! §23 — a trip ships it OFF and demotes the hypothesis, never the loop.)
//!
//! The estimator is the standard candidate-pool one: generate N candidates per task **once**;
//! `pass@1` = mean per-task fraction of passing candidates (the honest single-pass expectation
//! under the same sampling), `pass@N` = fraction of tasks where **any** candidate passes (what
//! verifier-select achieves, since the checks here are deterministic non-model assertions — I5).
//! Thresholds are **pre-registered in the set file** before any run (the C1/C2 discipline).

use keel_contracts::{Content, Context, Effort, GenerateRequest, KeelError, Message, ModelTier, Result, Role};
use std::collections::BTreeMap;

/// One deterministic check. `kind`: `equals` (whole normalized output) · `contains` (normalized
/// substring) · `contains_all` (every normalized substring; `value` is an array). Deterministic ⇒
/// a non-model assertion — the selection signal is I5-grounded by construction.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmplifyCheck {
    pub kind: String,
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmplifyTask {
    pub id: String,
    pub family: String,
    pub prompt: String,
    pub check: AmplifyCheck,
}

/// The fixed benchmark set (`tests/amplify/amplify-set.json`). Checks are deterministic, so unlike
/// the golden-recall set there is no human ground truth to ratify — the pre-registered `thresholds`
/// carry the decision policy instead.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmplifySet {
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub thresholds: serde_json::Value,
    pub tasks: Vec<AmplifyTask>,
}

pub const AMPLIFY_CHECK_KINDS: [&str; 3] = ["equals", "contains", "contains_all"];

impl AmplifySet {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| KeelError::Other(format!("amplify set read {}: {e}", path.display())))?;
        serde_json::from_str(&raw).map_err(|e| KeelError::Other(format!("amplify set parse {}: {e}", path.display())))
    }

    /// Structural coherence (ids unique, prompts non-empty, check kinds/values well-formed).
    pub fn lint(&self) -> Vec<String> {
        let mut errs = Vec::new();
        let mut ids = std::collections::BTreeSet::new();
        for t in &self.tasks {
            if !ids.insert(t.id.as_str()) {
                errs.push(format!("duplicate task id {}", t.id));
            }
            if t.prompt.trim().is_empty() {
                errs.push(format!("task {}: empty prompt", t.id));
            }
            match t.check.kind.as_str() {
                "equals" | "contains" => {
                    if t.check.value.as_str().map(str::trim).unwrap_or("").is_empty() {
                        errs.push(format!("task {}: {} needs a non-empty string value", t.id, t.check.kind));
                    }
                }
                "contains_all" => match t.check.value.as_array() {
                    Some(a) if !a.is_empty() && a.iter().all(|v| v.as_str().map(str::trim).map(|s| !s.is_empty()).unwrap_or(false)) => {}
                    _ => errs.push(format!("task {}: contains_all needs a non-empty array of strings", t.id)),
                },
                k => errs.push(format!("task {}: unknown check kind '{k}'", t.id)),
            }
        }
        if self.tasks.is_empty() {
            errs.push("no tasks".into());
        }
        errs
    }
}

/// Normalize a model output for checking: strip **closed** `<think>` blocks (the A7 lived lesson —
/// a lean local pass can still leak them; an *unclosed* block means the answer never arrived and
/// the text stays as-is, failing honestly), trim, collapse whitespace runs, lowercase.
pub fn normalize(raw: &str) -> String {
    let mut s = raw.to_string();
    while let (Some(a), Some(b)) = (s.find("<think>"), s.find("</think>")) {
        if b < a {
            break; // malformed ordering — leave as-is, fail honestly
        }
        s.replace_range(a..b + "</think>".len(), " ");
    }
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Apply one deterministic check to a raw model output.
pub fn check_output(check: &AmplifyCheck, raw: &str) -> bool {
    let out = normalize(raw);
    match check.kind.as_str() {
        "equals" => out == normalize(check.value.as_str().unwrap_or("")),
        "contains" => out.contains(&normalize(check.value.as_str().unwrap_or(""))),
        "contains_all" => check
            .value
            .as_array()
            .map(|a| a.iter().all(|v| out.contains(&normalize(v.as_str().unwrap_or("")))))
            .unwrap_or(false),
        _ => false, // unknown kind never passes (the lint catches it first)
    }
}

/// One task's candidate pool outcome.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmplifyOutcome {
    pub id: String,
    pub family: String,
    /// Per-candidate pass/fail, in generation order.
    pub passes: Vec<bool>,
    /// The pass@1 estimator contribution: passing candidates / N.
    pub pass_frac: f32,
    /// The pass@N (verifier-select) contribution: any candidate passed.
    pub any_pass: bool,
    /// Per-candidate generation wall time (ms).
    pub gen_ms: Vec<u64>,
    /// The first PASSING candidate's raw output (what verifier-select would return), if any.
    pub selected: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AmplifyAgg {
    pub tasks: usize,
    pub pass_at_1: f32,
    pub pass_at_n: f32,
}

/// The artifact `keel amplify-bench` writes (verify-by-artifact; the ISSUE-4 decision input).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmplifyReport {
    pub set: String,
    pub set_version: u32,
    pub n: u32,
    pub model: String,
    pub overall: AmplifyAgg,
    pub per_family: BTreeMap<String, AmplifyAgg>,
    pub gen_p50_ms: u64,
    pub gen_p95_ms: u64,
    pub outcomes: Vec<AmplifyOutcome>,
}

/// Run the falsifier: `n` candidates per task through the tier (sequential; server-side sampling
/// provides the variance), each checked deterministically. Pure over the [`ModelTier`] seam — a
/// scripted stub exercises the whole pipeline model-free; the live leg is `keel amplify-bench`'s.
pub async fn run_amplify_bench(
    tier: &dyn ModelTier,
    ctx: &Context,
    set: &AmplifySet,
    n: u32,
    model: &str,
) -> Result<AmplifyReport> {
    let errs = set.lint();
    if !errs.is_empty() {
        return Err(KeelError::Other(format!("amplify set incoherent: {}", errs.join(" | "))));
    }
    let n = n.max(1);
    let mut outcomes = Vec::with_capacity(set.tasks.len());
    let mut all_ms: Vec<u64> = Vec::new();
    for t in &set.tasks {
        let mut passes = Vec::with_capacity(n as usize);
        let mut gen_ms = Vec::with_capacity(n as usize);
        let mut selected = None;
        for _ in 0..n {
            let req = GenerateRequest {
                messages: vec![Message {
                    role: Role::User,
                    content: vec![Content::Text { text: t.prompt.clone() }],
                    tool_call_id: None,
                    name: None,
                    reasoning_content: None,
                }],
                model: model.to_string(),
                tools: vec![],
                grammar: None,
                // lean thinking — the engine's local policy (content-forward, the routed default).
                effort: Effort { n: 1, thinking: Some("low".into()) },
                cache_prefix_len: None,
            };
            let t0 = std::time::Instant::now();
            let r = tier.generate(req, ctx).await?;
            let ms = t0.elapsed().as_millis() as u64;
            gen_ms.push(ms);
            all_ms.push(ms);
            let ok = check_output(&t.check, &r.content);
            if ok && selected.is_none() {
                selected = Some(r.content.clone());
            }
            passes.push(ok);
        }
        let pass_count = passes.iter().filter(|p| **p).count();
        outcomes.push(AmplifyOutcome {
            id: t.id.clone(),
            family: t.family.clone(),
            pass_frac: pass_count as f32 / n as f32,
            any_pass: pass_count > 0,
            passes,
            gen_ms,
            selected,
        });
    }
    let agg = |of: &[&AmplifyOutcome]| -> AmplifyAgg {
        let tasks = of.len();
        if tasks == 0 {
            return AmplifyAgg::default();
        }
        AmplifyAgg {
            tasks,
            pass_at_1: of.iter().map(|o| o.pass_frac).sum::<f32>() / tasks as f32,
            pass_at_n: of.iter().filter(|o| o.any_pass).count() as f32 / tasks as f32,
        }
    };
    let all: Vec<&AmplifyOutcome> = outcomes.iter().collect();
    let mut per_family: BTreeMap<String, AmplifyAgg> = BTreeMap::new();
    let families: std::collections::BTreeSet<String> = outcomes.iter().map(|o| o.family.clone()).collect();
    for fam in families {
        let of: Vec<&AmplifyOutcome> = outcomes.iter().filter(|o| o.family == fam).collect();
        per_family.insert(fam, agg(&of));
    }
    all_ms.sort_unstable();
    let pct = |p: f64| -> u64 {
        if all_ms.is_empty() {
            return 0;
        }
        let rank = ((p / 100.0) * all_ms.len() as f64).ceil().max(1.0) as usize;
        all_ms[rank.min(all_ms.len()) - 1]
    };
    Ok(AmplifyReport {
        set: set.name.clone(),
        set_version: set.version,
        n,
        model: model.to_string(),
        overall: agg(&all),
        per_family,
        gen_p50_ms: pct(50.0),
        gen_p95_ms: pct(95.0),
        outcomes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use keel_contracts::{Capabilities, GenerateResult};
    use serde_json::json;
    use std::sync::Mutex;

    #[test]
    fn normalize_strips_think_blocks_and_flattens() {
        assert_eq!(normalize("  The\n Answer  IS   42 "), "the answer is 42");
        assert_eq!(normalize("<think>step by step...</think>\n3901"), "3901");
        // unclosed think = the answer never arrived — text stays, fails honestly downstream.
        assert_eq!(normalize("<think>still going"), "<think>still going");
    }

    #[test]
    fn checks_apply_normalized() {
        let eq = AmplifyCheck { kind: "equals".into(), value: json!("Leek") };
        assert!(check_output(&eq, "  leek \n"));
        assert!(!check_output(&eq, "the answer is leek"));
        let ct = AmplifyCheck { kind: "contains".into(), value: json!("apple fig kiwi mango") };
        assert!(check_output(&ct, "Sorted: apple  fig kiwi mango."));
        let ca = AmplifyCheck { kind: "contains_all".into(), value: json!(["\"port\"", "8080"]) };
        assert!(check_output(&ca, "{\"port\": 8080}"));
        assert!(!check_output(&ca, "{\"host\": 8080}"));
        let bad = AmplifyCheck { kind: "regex".into(), value: json!("x") };
        assert!(!check_output(&bad, "x"), "unknown kind never passes");
    }

    fn set(tasks: Vec<AmplifyTask>) -> AmplifySet {
        AmplifySet { name: "stub".into(), version: 1, thresholds: serde_json::Value::Null, tasks }
    }

    fn task(id: &str, family: &str, prompt: &str, kind: &str, value: serde_json::Value) -> AmplifyTask {
        AmplifyTask { id: id.into(), family: family.into(), prompt: prompt.into(), check: AmplifyCheck { kind: kind.into(), value } }
    }

    #[test]
    fn lint_catches_incoherence() {
        let s = set(vec![
            task("a", "f", " ", "equals", json!("x")),
            task("a", "f", "p", "sorcery", json!("x")),
            task("b", "f", "p", "contains_all", json!([])),
            task("c", "f", "p", "equals", json!("")),
        ]);
        let errs = s.lint();
        let has = |m: &str| errs.iter().any(|e| e.contains(m));
        assert!(has("duplicate task id a"));
        assert!(has("empty prompt"));
        assert!(has("unknown check kind 'sorcery'"));
        assert!(has("contains_all needs a non-empty array"));
        assert!(has("equals needs a non-empty string"));
        assert!(set(vec![]).lint().iter().any(|e| e.contains("no tasks")));
    }

    /// The shipped fixed set stays structurally coherent (thresholds are policy, content is checks —
    /// nothing here asserts content, so tuning tasks never breaks the gate).
    #[test]
    fn the_amplify_set_file_loads_and_lints_clean() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/amplify/amplify-set.json");
        let s = AmplifySet::load(std::path::Path::new(path)).expect("set loads");
        assert_eq!(s.name, "amplify-set");
        assert!(!s.tasks.is_empty());
        let errs = s.lint();
        assert!(errs.is_empty(), "structural lint must be clean: {errs:?}");
        assert!(
            s.thresholds.get("b1_pass_at_n_uplift_min").is_some(),
            "the pre-registered decision threshold rides the set file"
        );
    }

    /// Cycles scripted outputs per call — deterministic candidate pools without a model.
    struct CycleTier {
        outputs: Vec<&'static str>,
        calls: Mutex<usize>,
    }
    #[async_trait]
    impl ModelTier for CycleTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            let mut c = self.calls.lock().unwrap();
            let out = self.outputs[*c % self.outputs.len()];
            *c += 1;
            Ok(GenerateResult { content: out.into(), cost: 0.0, tier: "local".into(), model: req.model, ..Default::default() })
        }
    }

    #[tokio::test]
    async fn bench_estimates_pass_at_1_and_pass_at_n_from_one_pool() {
        // task t1 passes on every 3rd candidate (pool of 3: wrong, wrong, RIGHT);
        // task t2 never passes. n=3 → t1: pass_frac 1/3, any true; t2: 0, false.
        let s = set(vec![
            task("t1", "math", "2+2?", "equals", json!("4")),
            task("t2", "math", "3+3?", "equals", json!("6")),
        ]);
        let tier = CycleTier { outputs: vec!["5", "22", "4", "7", "9", "8"], calls: Mutex::new(0) };
        let ctx = Context { trace_id: "b".into(), ..Default::default() };
        let r = run_amplify_bench(&tier, &ctx, &s, 3, "stub-model").await.unwrap();
        assert_eq!(r.n, 3);
        let t1 = r.outcomes.iter().find(|o| o.id == "t1").unwrap();
        assert_eq!(t1.passes, vec![false, false, true]);
        assert!((t1.pass_frac - 1.0 / 3.0).abs() < 1e-6);
        assert!(t1.any_pass);
        assert_eq!(t1.selected.as_deref(), Some("4"), "verifier-select returns the passing candidate");
        let t2 = r.outcomes.iter().find(|o| o.id == "t2").unwrap();
        assert!(!t2.any_pass);
        assert!((r.overall.pass_at_1 - (1.0 / 3.0) / 2.0).abs() < 1e-6);
        assert!((r.overall.pass_at_n - 0.5).abs() < 1e-6);
        assert_eq!(r.per_family["math"].tasks, 2);
    }

    #[tokio::test]
    async fn bench_refuses_an_incoherent_set() {
        let s = set(vec![task("x", "f", "p", "wat", json!("v"))]);
        let tier = CycleTier { outputs: vec!["a"], calls: Mutex::new(0) };
        let ctx = Context { trace_id: "b".into(), ..Default::default() };
        assert!(run_amplify_bench(&tier, &ctx, &s, 2, "m").await.is_err());
    }
}
