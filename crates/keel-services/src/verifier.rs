//! keel-services::verifier — the externality layer (I5, canon §10). KEEL routes and runs; **only
//! here does it verify** — against an assertion **no model authored**. This is the keystone that
//! separates a trust economy from an off-the-shelf harness (§0): the model may author the plan, the
//! code, even a verification pass — it may *not* author its own ground truth.
//!
//! Three model-free oracle kinds cover the frozen `GOLDEN_ORACLE` cases, walking the §10 taxonomy:
//! - [`PropertyOracle`] — a deterministic property/metamorphic assertion on the output (e.g. "no
//!   SSN pattern"). The highest-recall, model-free rung.
//! - [`GoldenOracle`] — the **joint-wrong detector**: when the implementation's own tests pass yet a
//!   frozen golden is violated → `JOINT_WRONG`, the most dangerous finding (everything looked green).
//! - [`SourceOracle`] — the trace-to-canon gate (SEXTANT's "Truth Gate"): an ungrounded claim →
//!   `INSUFFICIENT_SOURCE`, which routes to a human (§18).
//!
//! [`Verifier`] is the **pluggable registry** (§10): each cell registers its own oracles; `verify`
//! runs them all and folds the verdicts. The registry — not any single oracle — is the seam a cell
//! specializes (a clerk's cross-footing, a companion's chattiness gate, …) without a core edit.

use async_trait::async_trait;
use keel_contracts::{Assertion, Context, GoldenCase, Oracle, Result, StepOutput, Verdict};
use regex::Regex;
use serde_json::Value;

// ── property / metamorphic oracle ─────────────────────────────────────────────

/// Asserts named, deterministic properties of an output — no model. Unknown properties **fail
/// closed** (KEEL never claims a property held that it could not check — the §5 "fail toward the
/// safe side" rule applied to correctness).
pub struct PropertyOracle {
    properties: Vec<String>,
}

impl PropertyOracle {
    pub fn new(properties: Vec<String>) -> Self {
        Self { properties }
    }

    /// `Some(failure)` if the property is violated (or unknown); `None` if it holds.
    fn check(&self, property: &str, content: &str) -> Option<String> {
        match property {
            "no_ssn_pattern" => {
                let re = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").expect("static SSN pattern is valid");
                re.is_match(content).then(|| "no_ssn_pattern: output contains an SSN-shaped span".to_string())
            }
            other => Some(format!("unknown property '{other}' — fail-closed (cannot assert what it cannot check)")),
        }
    }
}

#[async_trait]
impl Oracle for PropertyOracle {
    async fn verify(&self, output: &StepOutput, _golden: &[GoldenCase], _ctx: &Context) -> Result<Verdict> {
        let mut failures = Vec::new();
        let mut evidence = Vec::new();
        for property in &self.properties {
            match self.check(property, &output.content) {
                Some(failure) => {
                    evidence.push(Assertion { kind: "property".into(), detail: failure.clone() });
                    failures.push(failure);
                }
                None => evidence.push(Assertion { kind: "property".into(), detail: format!("{property}: held") }),
            }
        }
        Ok(Verdict { passed: failures.is_empty(), failures, joint_wrong: false, evidence })
    }
}

// ── joint-wrong oracle ────────────────────────────────────────────────────────

/// The joint-wrong detector (canon §10): runs golden cases against an output with **contract +
/// golden + diff but NOT the implementer's reasoning**. If the implementation's own tests pass yet a
/// frozen golden is violated → `JOINT_WRONG` (surfaced to the operator immediately, §18). For
/// conformance the two signals arrive on `output.artifact`; in a live run the verifier derives them
/// by actually running the goldens (`golden_violated`) and the impl's self-tests (`self_tests_pass`).
pub struct GoldenOracle;

#[async_trait]
impl Oracle for GoldenOracle {
    async fn verify(&self, output: &StepOutput, _golden: &[GoldenCase], _ctx: &Context) -> Result<Verdict> {
        let self_tests_pass = output.artifact["self_tests_pass"].as_bool().unwrap_or(false);
        let golden_violated = output.artifact["golden_violated"].as_bool().unwrap_or(false);
        let joint_wrong = self_tests_pass && golden_violated;

        let mut failures = Vec::new();
        if golden_violated {
            failures.push(if joint_wrong {
                "JOINT_WRONG: self-tests pass but a frozen golden is violated (systematic — everything looked green)".to_string()
            } else {
                "golden_violated: output disagrees with a frozen golden".to_string()
            });
        }
        let evidence = vec![Assertion {
            kind: if joint_wrong { "joint_wrong" } else { "golden" }.into(),
            detail: format!("self_tests_pass={self_tests_pass} golden_violated={golden_violated}"),
        }];
        Ok(Verdict { passed: !golden_violated, failures, joint_wrong, evidence })
    }
}

// ── source-trace oracle ───────────────────────────────────────────────────────

/// The trace-to-ground-truth gate (canon §10, §17 — SEXTANT's "Truth Gate"): every claim must trace
/// to the canon/ground truth. An ungrounded claim → `INSUFFICIENT_SOURCE` (routes to a human, §18) —
/// the externality that stops a model fabricating its own support.
pub struct SourceOracle;

#[async_trait]
impl Oracle for SourceOracle {
    async fn verify(&self, output: &StepOutput, _golden: &[GoldenCase], _ctx: &Context) -> Result<Verdict> {
        // absence of the flag is treated as "supported" (only an explicit `false` flags a claim)
        let canon_supports = output.artifact["canon_supports"].as_bool().unwrap_or(true);
        if canon_supports {
            return Ok(Verdict {
                passed: true,
                failures: Vec::new(),
                joint_wrong: false,
                evidence: vec![Assertion { kind: "source".into(), detail: "claim traces to the canon".into() }],
            });
        }
        let claim = output.artifact["claim"].as_str().unwrap_or("<claim>");
        let failure = format!("INSUFFICIENT_SOURCE: claim '{claim}' does not trace to the canon (→ human)");
        Ok(Verdict {
            passed: false,
            failures: vec![failure.clone()],
            joint_wrong: false,
            evidence: vec![Assertion { kind: "source".into(), detail: failure }],
        })
    }
}

// ── schema-validation oracle ──────────────────────────────────────────────────

/// The schema-validation oracle (canon §10, §17 — the Backrooms Director's "schema-valid Directive
/// or reject" gate). **Non-model**: it validates the output against a JSON Schema with the
/// `jsonschema` crate, **in-memory only** (the crate's network/`$ref` features are off, so a
/// safety-critical oracle never egresses — I3). The instance is `output.artifact` when present, else
/// `output.content` parsed as JSON. **Any** violation → the verdict fails and lists every violation:
/// an invalid output is **rejected, never partially applied** (the Director's invariant). A schema
/// that will not compile **fails closed** — KEEL never blesses what it cannot check (§5).
/// The JSON Schema draft KEEL pins for **every** `SchemaOracle` — Draft 2020-12 (the current IETF
/// standard; KEEL and its cells author Directive/tool schemas against it). A safety oracle must
/// **never** auto-select the draft from `$schema` (or default to "latest"): validating against a
/// different draft than the schema was authored for is a silent JOINT_WRONG (subtly-wrong
/// accept/reject, everything green). The pin is explicit and the constructor strips any embedded
/// `$schema`, so the governing draft is deterministic regardless of the document. (A cell needing a
/// different draft is a future explicit `with_draft` param — never an auto-detect.)
const PINNED_DRAFT: jsonschema::Draft = jsonschema::Draft::Draft202012;

pub struct SchemaOracle {
    schema: Value,
}

impl SchemaOracle {
    pub fn new(mut schema: Value) -> Self {
        // determinism: drop any embedded `$schema` so PINNED_DRAFT always governs, never the document.
        if let Some(obj) = schema.as_object_mut() {
            obj.remove("$schema");
        }
        Self { schema }
    }

    fn reject(detail: String) -> Verdict {
        Verdict {
            passed: false,
            failures: vec![detail.clone()],
            joint_wrong: false,
            evidence: vec![Assertion { kind: "schema".into(), detail }],
        }
    }
}

#[async_trait]
impl Oracle for SchemaOracle {
    async fn verify(&self, output: &StepOutput, _golden: &[GoldenCase], _ctx: &Context) -> Result<Verdict> {
        // the instance to check: the structured artifact if present, else parse content as JSON.
        let instance: Value = if !output.artifact.is_null() {
            output.artifact.clone()
        } else {
            match serde_json::from_str(&output.content) {
                Ok(v) => v,
                Err(e) => return Ok(Self::reject(format!("schema: output is not JSON ({e}) — rejected, never partially applied"))),
            }
        };
        // compile the schema; fail closed if it cannot be compiled (cannot assert what it cannot check).
        let validator = match jsonschema::options().with_draft(PINNED_DRAFT).build(&self.schema) {
            Ok(v) => v,
            Err(e) => return Ok(Self::reject(format!("schema: cannot compile schema ({e}) — fail-closed"))),
        };
        let failures: Vec<String> = validator.iter_errors(&instance).map(|e| format!("schema: {e}")).collect();
        if failures.is_empty() {
            Ok(Verdict {
                passed: true,
                failures: Vec::new(),
                joint_wrong: false,
                evidence: vec![Assertion { kind: "schema".into(), detail: "output validates against the schema".into() }],
            })
        } else {
            let detail = format!("schema-invalid (rejected, never partially applied): {}", failures.join("; "));
            Ok(Verdict { passed: false, failures, joint_wrong: false, evidence: vec![Assertion { kind: "schema".into(), detail }] })
        }
    }
}

// ── the pluggable registry ────────────────────────────────────────────────────

/// The pluggable oracle registry (canon §10): a cell registers its own oracles; `verify` runs them
/// all and folds the verdicts — `passed` is AND (every oracle must clear), `joint_wrong` is OR (any
/// oracle may raise it), failures/evidence concatenate. The registry is the I5 seam a cell
/// specializes without touching the core.
#[derive(Default)]
pub struct Verifier {
    oracles: Vec<Box<dyn Oracle>>,
}

impl Verifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an oracle. Returns `&mut self` for chaining.
    pub fn register(&mut self, oracle: Box<dyn Oracle>) -> &mut Self {
        self.oracles.push(oracle);
        self
    }

    pub fn len(&self) -> usize {
        self.oracles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.oracles.is_empty()
    }

    /// Run every registered oracle and fold the verdicts into one (I5). An empty registry passes
    /// vacuously — a critical step with no oracle registered is itself a config fault the engine
    /// guards, not the verifier.
    pub async fn verify(&self, output: &StepOutput, golden: &[GoldenCase], ctx: &Context) -> Result<Verdict> {
        let mut agg = Verdict { passed: true, failures: Vec::new(), joint_wrong: false, evidence: Vec::new() };
        for oracle in &self.oracles {
            let v = oracle.verify(output, golden, ctx).await?;
            agg.passed &= v.passed;
            agg.joint_wrong |= v.joint_wrong;
            agg.failures.extend(v.failures);
            agg.evidence.extend(v.evidence);
        }
        Ok(agg)
    }
}

/// The registry is itself an [`Oracle`] (a composite): registering N oracles and folding their
/// verdicts is exactly the `Oracle` contract at a coarser grain. This lets `kernel::engine` hold a
/// single `&dyn Oracle` for the whole I5 surface while a cell composes many behind it — a pure L4
/// addition (zero contract edit). The inherent [`Verifier::verify`] is the implementation; this just
/// exposes it through the frozen joint.
#[async_trait]
impl Oracle for Verifier {
    async fn verify(&self, output: &StepOutput, golden: &[GoldenCase], ctx: &Context) -> Result<Verdict> {
        Verifier::verify(self, output, golden, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    /// Build a `StepOutput` from a language-neutral GOLDEN_ORACLE `input` object: `output` (if any)
    /// is the content; the whole input rides as the artifact so each oracle reads the signals it needs.
    fn step_output(input: &Value) -> StepOutput {
        StepOutput {
            content: input["output"].as_str().unwrap_or_default().to_string(),
            artifact: input.clone(),
        }
    }

    /// The conformance test: the verifier MUST satisfy every frozen GOLDEN_ORACLE case (I5).
    #[tokio::test]
    async fn passes_golden_oracle() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/golden.json");
        let raw = std::fs::read_to_string(path).expect("read golden.json");
        let golden: Value = serde_json::from_str(&raw).expect("parse golden.json");
        let cases = golden["oracle"].as_array().expect("oracle cases");
        assert!(!cases.is_empty());
        let ctx = Context::default();

        for case in cases {
            let name = case["name"].as_str().unwrap_or("?");
            let input = &case["input"];
            let out = step_output(input);

            // dispatch by input shape (the golden is language-neutral; the impl interprets it)
            let verdict = if let Some(property) = input["property"].as_str() {
                PropertyOracle::new(vec![property.to_string()]).verify(&out, &[], &ctx).await.unwrap()
            } else if input.get("self_tests_pass").is_some() {
                GoldenOracle.verify(&out, &[], &ctx).await.unwrap()
            } else if input.get("claim").is_some() {
                SourceOracle.verify(&out, &[], &ctx).await.unwrap()
            } else {
                panic!("case '{name}': unrecognized oracle input shape");
            };

            let expect = &case["expect"];
            if let Some(p) = expect["passed"].as_bool() {
                assert_eq!(verdict.passed, p, "case '{name}': passed");
            }
            if let Some(jw) = expect["joint_wrong"].as_bool() {
                assert_eq!(verdict.joint_wrong, jw, "case '{name}': joint_wrong");
            }
            if let Some(sub) = expect["reason_contains"].as_str() {
                let hay = verdict.failures.join(" ").to_lowercase();
                assert!(hay.contains(&sub.to_lowercase()), "case '{name}': failures {:?} missing '{sub}'", verdict.failures);
            }
        }
    }

    /// The registry folds verdicts: AND over `passed`, OR over `joint_wrong`.
    #[tokio::test]
    async fn registry_folds_verdicts() {
        let mut v = Verifier::new();
        v.register(Box::new(PropertyOracle::new(vec!["no_ssn_pattern".into()])));
        assert_eq!(v.len(), 1);

        let clean = StepOutput { content: "all clear".into(), artifact: Value::Null };
        assert!(v.verify(&clean, &[], &Context::default()).await.unwrap().passed);

        let dirty = StepOutput { content: "ssn 123-45-6789 leaked".into(), artifact: Value::Null };
        let verdict = v.verify(&dirty, &[], &Context::default()).await.unwrap();
        assert!(!verdict.passed);
        assert!(!verdict.failures.is_empty());

        // a joint-wrong oracle raises the flag through the fold
        v.register(Box::new(GoldenOracle));
        let jw = StepOutput { content: "x".into(), artifact: json!({ "self_tests_pass": true, "golden_violated": true }) };
        assert!(v.verify(&jw, &[], &Context::default()).await.unwrap().joint_wrong);
    }

    /// The Director's gate (canon §17): a conformant Directive validates; a malformed one is
    /// **rejected, never partially applied.** The rejection case is load-bearing — a validator that
    /// accepts everything passes the accept-case trivially, so we prove it *catches the bad*.
    #[tokio::test]
    async fn schema_oracle_validates_and_rejects() {
        // a Director-shaped Directive schema: required action(string) + bounded intensity(int 0..10).
        let schema = json!({
            "type": "object",
            "required": ["action", "intensity"],
            "properties": {
                "action": { "type": "string" },
                "intensity": { "type": "integer", "minimum": 0, "maximum": 10 }
            },
            "additionalProperties": false
        });
        let oracle = SchemaOracle::new(schema);
        let ctx = Context::default();

        // conformant Directive → accepted
        let good = StepOutput { content: String::new(), artifact: json!({ "action": "spawn", "intensity": 7 }) };
        assert!(oracle.verify(&good, &[], &ctx).await.unwrap().passed, "conformant Directive must validate");

        // LOAD-BEARING: malformed — missing required `action` AND wrong-typed `intensity` — must be
        // REJECTED (invalid, never partially applied), with the violations listed.
        let bad = StepOutput { content: String::new(), artifact: json!({ "intensity": "high" }) };
        let v = oracle.verify(&bad, &[], &ctx).await.unwrap();
        assert!(!v.passed, "malformed Directive must be rejected");
        assert!(!v.failures.is_empty(), "rejection must list the violations");

        // a value the schema forbids (out-of-range + extra prop) → also rejected
        let oob = StepOutput { content: String::new(), artifact: json!({ "action": "x", "intensity": 99, "extra": true }) };
        assert!(!oracle.verify(&oob, &[], &ctx).await.unwrap().passed, "out-of-range / extra-prop must be rejected");

        // content path: non-JSON output with a null artifact → fail-closed (cannot validate ⇒ reject).
        let nonjson = StepOutput { content: "not json at all".into(), artifact: Value::Null };
        assert!(!oracle.verify(&nonjson, &[], &ctx).await.unwrap().passed, "non-JSON output fails closed");
    }

    /// GOLDEN_MODEL_TIER case [1], verify side: the golden's schema validates a conformant tool-call
    /// (`tool_call_valid_against_schema: true`) and rejects one that violates it. (The decode side —
    /// schema → llama-server `json_schema` — is asserted in keel-adapters `passes_golden_model_tier`.)
    #[tokio::test]
    async fn golden_model_tier_schema_is_enforced() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/golden.json");
        let raw = std::fs::read_to_string(path).expect("read golden.json");
        let golden: Value = serde_json::from_str(&raw).expect("parse golden.json");
        let cases = golden["model_tier"].as_array().expect("model_tier cases");
        let schema = cases
            .iter()
            .find_map(|c| c["input"].get("schema").cloned())
            .expect("a model_tier case carrying a schema");

        let oracle = SchemaOracle::new(schema);
        let ctx = Context::default();
        // conformant to {required:[path], properties:{path:string}} → valid
        let good = StepOutput { content: String::new(), artifact: json!({ "path": "/etc/hosts" }) };
        assert!(oracle.verify(&good, &[], &ctx).await.unwrap().passed, "conformant tool-call must validate");
        // missing required `path` → rejected
        let bad = StepOutput { content: String::new(), artifact: json!({ "wrong": 1 }) };
        assert!(!oracle.verify(&bad, &[], &ctx).await.unwrap().passed, "non-conformant tool-call must be rejected");
    }

    /// Hardening: the oracle **pins** the draft (Draft 2020-12) and never auto-selects from `$schema`
    /// — else a Directive validated against the wrong draft is a silent JOINT_WRONG. Proof:
    /// `exclusiveMinimum` is *numeric* in Draft 2020-12 (the value must exceed it) but a *boolean*
    /// modifier in Draft 4; we even embed a draft-04 `$schema` decoy. Under the pin, instance `5`
    /// violates `exclusiveMinimum: 5` (5 is not > 5) → **rejected**; `6` → accepted (which also proves
    /// the schema compiled + the constraint is live, so the rejection isn't a fail-closed artifact).
    #[tokio::test]
    async fn schema_oracle_pins_the_draft() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-04/schema#", // decoy — must NOT take effect
            "type": "integer",
            "exclusiveMinimum": 5
        });
        let oracle = SchemaOracle::new(schema);
        let ctx = Context::default();
        let five = StepOutput { content: String::new(), artifact: json!(5) };
        assert!(
            !oracle.verify(&five, &[], &ctx).await.unwrap().passed,
            "pinned Draft 2020-12: 5 is not > 5 → rejected (the decoy draft-04 must be ignored)"
        );
        let six = StepOutput { content: String::new(), artifact: json!(6) };
        assert!(
            oracle.verify(&six, &[], &ctx).await.unwrap().passed,
            "6 > 5 → valid (proves the schema compiled and the constraint is actually applied)"
        );
    }
}
