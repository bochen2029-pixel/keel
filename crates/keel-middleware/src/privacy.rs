//! keel-middleware::privacy — I3, the deterministic mask (canon §5.1, rungs 1–2).
//!
//! The non-model privacy **oracle**: operator sovereign markers (rung 1, exact strings) + a
//! structured regex/checksum sweep (rung 2: email, SSN, `sk-`/`AKIA` keys, Luhn-valid cards).
//! `redact` returns the scrubbed text plus the `Finding`s — a non-model assertion that PII is
//! present (the highest-recall, model-free rungs). The probabilistic rung 3 (the OpenAI Privacy
//! Filter) is a *verification pass* that lands in Stage 2 behind `GOLDEN_PRIVACY`; it is never
//! the guarantee.
//!
//! **The operator's sovereign markers are operator-authored, agent-frozen ground truth** (like
//! the goldens): this module supplies the *mechanism*; the marker list is configured/frozen by
//! the operator, never invented by the agent.

use crate::audit::{AuditEvent, AuditSink};
use async_trait::async_trait;
use keel_contracts::{Content, Context, GenerateRequest, GenerateResult, Middleware, Next, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A detected PII span's classification (not its value — the secret isn't retained here).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub kind: String,
    /// 1 = operator marker, 2 = structured regex/checksum, 3 = model classifier (additive recall).
    pub rung: u8,
}

/// A model-detected PII span (rung 3): **byte** offsets into the original text + the class label
/// (e.g. `private_person`). The value itself is never retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiiSpan {
    pub start: usize,
    pub end: usize,
    pub class: String,
}

/// The rung-3 seam (canon §5.1): a probabilistic classifier that ADDS recall over the
/// deterministic rungs — **never the guarantee, structurally unable to unmask** (its spans join
/// the rung-1/2 union; a union only grows). Sync by design: rung 3 is in-process CPU inference
/// (the OpenAI privacy filter over `ort`), not a server call. Implementations live at L4+
/// (feature-gated, heavy deps stay out of the default genome); this module owns only the seam.
pub trait PiiClassifier: Send + Sync {
    fn spans(&self, text: &str) -> Vec<PiiSpan>;
}

/// The redactor: deterministic rungs 1–2 always; an optional rung-3 classifier joins on the
/// model-assisted path ([`Redactor::redact_with_model`]) only. Compiles its patterns once.
pub struct Redactor {
    markers: Vec<String>,
    patterns: Vec<(&'static str, Regex)>,
    classifier: Option<Arc<dyn PiiClassifier>>,
}

impl Redactor {
    /// Build with the operator's sovereign markers (rung 1). Pass an empty list to run rung 2 only.
    pub fn new(markers: Vec<String>) -> Self {
        let raw: &[(&'static str, &str)] = &[
            ("email", r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}"),
            ("ssn", r"\b\d{3}-\d{2}-\d{4}\b"),
            ("api_key", r"\bsk-[A-Za-z0-9]{4,}\b"),
            ("aws_key", r"\bAKIA[0-9A-Z]{16}\b"),
            ("credit_card", r"\b\d(?:[ -]?\d){12,18}\b"),
        ];
        let patterns = raw
            .iter()
            .map(|(k, p)| (*k, Regex::new(p).expect("static pattern is valid")))
            .collect();
        Self { markers, patterns, classifier: None }
    }

    /// Attach the rung-3 classifier (the L5 wiring does this when the `privacy-model` substrate
    /// resolves). Only [`Redactor::redact_with_model`] consults it; [`Redactor::redact`] stays
    /// deterministic-only forever.
    pub fn with_classifier(mut self, classifier: Arc<dyn PiiClassifier>) -> Self {
        self.classifier = Some(classifier);
        self
    }

    /// The deterministic rung-1/2 span sweep: (start, end, kind, rung).
    fn deterministic_spans(&self, text: &str) -> Vec<(usize, usize, String, u8)> {
        let mut spans: Vec<(usize, usize, String, u8)> = Vec::new();

        // rung 1 — operator markers (exact, case-sensitive)
        for m in self.markers.iter().filter(|m| !m.is_empty()) {
            let mut from = 0;
            while let Some(rel) = text[from..].find(m.as_str()) {
                let s = from + rel;
                let e = s + m.len();
                spans.push((s, e, "operator_marker".to_string(), 1));
                from = e;
            }
        }

        // rung 2 — structured patterns (credit cards must pass Luhn)
        for (kind, re) in &self.patterns {
            for mat in re.find_iter(text) {
                if *kind == "credit_card" && !luhn_ok(mat.as_str()) {
                    continue;
                }
                spans.push((mat.start(), mat.end(), (*kind).to_string(), 2));
            }
        }
        spans
    }

    /// Mask the span union with `[REDACTED]` and derive the findings. Empty ⇒ unchanged text.
    fn mask(text: &str, mut spans: Vec<(usize, usize, String, u8)>) -> (String, Vec<Finding>) {
        if spans.is_empty() {
            return (text.to_string(), Vec::new());
        }
        let findings = spans.iter().map(|(_, _, k, r)| Finding { kind: k.clone(), rung: *r }).collect();

        // merge overlapping spans, then rebuild with the placeholder
        spans.sort_by_key(|s| s.0);
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (s, e, _, _) in spans {
            match merged.last_mut() {
                Some(last) if s <= last.1 => last.1 = last.1.max(e),
                _ => merged.push((s, e)),
            }
        }

        let mut out = String::new();
        let mut cur = 0;
        for (s, e) in merged {
            out.push_str(&text[cur..s]);
            out.push_str("[REDACTED]");
            cur = e;
        }
        out.push_str(&text[cur..]);
        (out, findings)
    }

    /// Redact every marker/PII span (the rung-1/2 union) with `[REDACTED]`; return the scrubbed
    /// text and the findings. **Deterministic-only — the guarantee path** (canon §5.1); empty
    /// findings ⇒ the text is returned unchanged.
    pub fn redact(&self, text: &str) -> (String, Vec<Finding>) {
        Self::mask(text, self.deterministic_spans(text))
    }

    /// The model-assisted path: rungs 1–2 **plus** the rung-3 classifier's spans (additive-only —
    /// the model joins the union, it can never shrink it). No classifier wired ⇒ identical to
    /// [`Redactor::redact`]. Model spans with invalid offsets (out of range / not on a char
    /// boundary) are dropped defensively — a bad span must never panic the mask.
    pub fn redact_with_model(&self, text: &str) -> (String, Vec<Finding>) {
        let mut spans = self.deterministic_spans(text);
        if let Some(c) = &self.classifier {
            for s in c.spans(text) {
                let valid = s.start < s.end
                    && s.end <= text.len()
                    && text.is_char_boundary(s.start)
                    && text.is_char_boundary(s.end);
                if valid {
                    spans.push((s.start, s.end, s.class, 3));
                }
            }
        }
        Self::mask(text, spans)
    }
}

/// Luhn check over the digits of `s` (ignores spaces/dashes). 13–19 digits.
fn luhn_ok(s: &str) -> bool {
    let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&digits.len()) {
        return false;
    }
    let mut sum = 0u32;
    for (i, &d) in digits.iter().rev().enumerate() {
        let v = if i % 2 == 1 {
            let dd = d * 2;
            if dd > 9 { dd - 9 } else { dd }
        } else {
            d
        };
        sum += v;
    }
    sum.is_multiple_of(10)
}

/// I3 mask middleware. Two rungs: the **request** rung scrubs outbound text on an **egress** call (a
/// non-local tier) and passes through locally (the input is sovereign-safe on the box); the **output**
/// rung scrubs the response on **every** tier, so the model's own PII never lands in KEEL's persistent
/// state (Tape/ledger), egress, or display. Both are audited (I1). The engine sets `egress = true` only
/// when the terminal tier leaves the box.
pub struct PrivacyMiddleware {
    redactor: Arc<Redactor>,
    egress: bool,
    /// I1 sink for redaction events (canon §5.1). `None` ⇒ mask without auditing (tests / an
    /// embedder that owns its own audit); the app always wires it, so in production every
    /// redaction is recorded.
    audit_sink: Option<Arc<dyn AuditSink>>,
}

impl PrivacyMiddleware {
    pub fn new(redactor: Arc<Redactor>, egress: bool, audit_sink: Option<Arc<dyn AuditSink>>) -> Self {
        Self { redactor, egress, audit_sink }
    }
}

#[async_trait]
impl Middleware for PrivacyMiddleware {
    async fn handle(&self, mut req: GenerateRequest, ctx: &Context, next: &dyn Next) -> Result<GenerateResult> {
        if self.egress {
            let mut labels: Vec<String> = Vec::new();
            for msg in &mut req.messages {
                for content in &mut msg.content {
                    if let Content::Text { text } = content {
                        // EGRESS = the model-assisted path (rung 3 rides here only, canon §5.1 —
                        // the sovereignty-critical direction; the $0 local path never pays it).
                        let (scrubbed, findings) = self.redactor.redact_with_model(text);
                        for f in &findings {
                            labels.push(format!("rung{}:{}", f.rung, f.kind));
                        }
                        *text = scrubbed;
                    }
                }
            }
            // I3 → I1: every redaction decision is an audited event (canon §5.1) — labels, never
            // values — so a leak is forensically traceable. No findings ⇒ no event (no noise).
            if !labels.is_empty() {
                if let Some(sink) = &self.audit_sink {
                    sink.emit(&AuditEvent::redaction(ctx.trace_id.clone(), req.model.clone(), labels));
                }
            }
        }
        let mut result = next.run(req, ctx).await?;

        // I3 OUTPUT rung (canon §5.1): scrub PII from the response on EVERY tier — so the model's own
        // output never lands in KEEL's persistent state (the Tape/ledger), egress, or display. This is
        // the proper home of what was an I5 no-SSN "baseline" stopgap in the engine. Always-on
        // (state-hygiene; a cell that needs a sovereign-local answer intact can swap a no-op redactor).
        let mut out_labels: Vec<String> = Vec::new();
        let (scrubbed, findings) = self.redactor.redact(&result.content);
        if !findings.is_empty() {
            result.content = scrubbed;
            for f in &findings {
                out_labels.push(format!("rung{}:{}", f.rung, f.kind));
            }
        }
        if let Some(rc) = result.reasoning_content.take() {
            let (s, rf) = self.redactor.redact(&rc);
            for f in &rf {
                out_labels.push(format!("rung{}:{}", f.rung, f.kind));
            }
            result.reasoning_content = Some(s);
        }
        if !out_labels.is_empty() {
            if let Some(sink) = &self.audit_sink {
                sink.emit(&AuditEvent::redaction(ctx.trace_id.clone(), result.model.clone(), out_labels));
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{PrivacyMiddleware, Redactor};
    use crate::audit::{AuditEvent, AuditSink};
    use async_trait::async_trait;
    use keel_contracts::{Capabilities, Content, Context, GenerateRequest, GenerateResult, Message, ModelTier, Result, Role};
    use keel_kernel::Chain;
    use std::sync::{Arc, Mutex};

    #[test]
    fn rung2_catches_operator_key_deterministically() {
        // mirrors GOLDEN_PRIVACY: "...sk-abc123..." → redacted by a rung-2 regex (no model)
        let r = Redactor::new(vec![]);
        let (out, findings) = r.redact("token is sk-abc123 ok");
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("sk-abc123"));
        assert!(findings.iter().any(|f| f.kind == "api_key" && f.rung == 2));
    }

    #[test]
    fn catches_email_and_ssn() {
        let r = Redactor::new(vec![]);
        let (out, f) = r.redact("mail a@b.com ssn 123-45-6789");
        assert!(!out.contains("a@b.com"));
        assert!(!out.contains("123-45-6789"));
        assert!(f.iter().any(|x| x.kind == "email"));
        assert!(f.iter().any(|x| x.kind == "ssn"));
    }

    #[test]
    fn credit_card_only_when_luhn_valid() {
        let r = Redactor::new(vec![]);
        let (valid, vf) = r.redact("card 4111111111111111 here"); // valid Luhn
        assert!(!valid.contains("4111111111111111"));
        assert!(vf.iter().any(|x| x.kind == "credit_card"));

        let (invalid, inf) = r.redact("card 4111111111111112 here"); // bad Luhn
        assert!(invalid.contains("4111111111111112"));
        assert!(!inf.iter().any(|x| x.kind == "credit_card"));
    }

    #[test]
    fn operator_marker_is_rung1() {
        let r = Redactor::new(vec!["ACME-INTERNAL".to_string()]);
        let (out, f) = r.redact("project ACME-INTERNAL ships friday");
        assert!(!out.contains("ACME-INTERNAL"));
        assert_eq!(f.iter().filter(|x| x.rung == 1).count(), 1);
    }

    #[test]
    fn clean_text_is_unchanged() {
        let r = Redactor::new(vec!["ACME-INTERNAL".to_string()]);
        let (out, f) = r.redact("the quick brown fox");
        assert_eq!(out, "the quick brown fox");
        assert!(f.is_empty());
    }

    // ── the middleware: redact on egress, pass through locally ──

    struct CaptureTier {
        seen: Arc<Mutex<String>>,
    }
    #[async_trait]
    impl ModelTier for CaptureTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            let text = req
                .messages
                .first()
                .and_then(|m| m.content.first())
                .map(|c| match c {
                    Content::Text { text } => text.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default();
            *self.seen.lock().unwrap() = text;
            Ok(GenerateResult::default())
        }
    }

    fn req_with(text: &str) -> GenerateRequest {
        GenerateRequest {
            messages: vec![Message {
                role: Role::User,
                content: vec![Content::Text { text: text.to_string() }],
                name: None,
                reasoning_content: None,
                tool_call_id: None,
            }],
            model: "m".into(),
            tools: vec![],
            grammar: None,
            effort: Default::default(),
            cache_prefix_len: None,
        }
    }

    async fn seen_text(egress: bool) -> String {
        let seen = Arc::new(Mutex::new(String::new()));
        let mw = PrivacyMiddleware::new(Arc::new(Redactor::new(vec![])), egress, None);
        let chain = Chain::new(vec![Arc::new(mw)]);
        chain
            .run(req_with("reach me at a@b.com"), &Context::default(), Arc::new(CaptureTier { seen: seen.clone() }))
            .await
            .unwrap();
        let s = seen.lock().unwrap().clone();
        s
    }

    #[tokio::test]
    async fn egress_scrubs_before_the_tier_sees_it() {
        let s = seen_text(true).await;
        assert!(!s.contains("a@b.com"));
        assert!(s.contains("[REDACTED]"));
    }

    #[tokio::test]
    async fn local_passes_through_unredacted() {
        let s = seen_text(false).await;
        assert_eq!(s, "reach me at a@b.com");
    }

    // ── I1 audit of redactions (canon §5.1: every redaction is an audited event) ──

    #[derive(Default)]
    struct VecAuditSink {
        events: Mutex<Vec<AuditEvent>>,
    }
    impl AuditSink for VecAuditSink {
        fn emit(&self, e: &AuditEvent) {
            self.events.lock().unwrap().push(e.clone());
        }
    }

    async fn run_with_sink(egress: bool, sink: Arc<VecAuditSink>) {
        let mw = PrivacyMiddleware::new(Arc::new(Redactor::new(vec![])), egress, Some(sink));
        let chain = Chain::new(vec![Arc::new(mw)]);
        chain
            .run(
                req_with("reach me at a@b.com"),
                &Context::default(),
                Arc::new(CaptureTier { seen: Arc::new(Mutex::new(String::new())) }),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn egress_redaction_emits_an_i1_event() {
        let sink = Arc::new(VecAuditSink::default());
        run_with_sink(true, sink.clone()).await;
        let events = sink.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, "REDACTION");
        assert!(events[0].redactions.iter().any(|r| r.contains("email")));
        // the value itself is never recorded, only the label
        assert!(!events[0].redactions.iter().any(|r| r.contains("a@b.com")));
    }

    #[tokio::test]
    async fn local_call_audits_no_redaction() {
        // non-egress + empty output (CaptureTier): no request mask, nothing to scrub on output → no event.
        let sink = Arc::new(VecAuditSink::default());
        run_with_sink(false, sink.clone()).await;
        assert!(sink.events.lock().unwrap().is_empty());
    }

    // ── the I3 OUTPUT rung (canon §5.1): scrub the model's response on every tier ──

    struct PiiTier;
    #[async_trait]
    impl ModelTier for PiiTier {
        fn caps(&self) -> Capabilities {
            Capabilities::default()
        }
        async fn generate(&self, _req: GenerateRequest, _ctx: &Context) -> Result<GenerateResult> {
            Ok(GenerateResult { content: "your key is sk-deadbeef99 ok".into(), model: "m".into(), ..Default::default() })
        }
    }

    #[tokio::test]
    async fn output_pii_is_masked_on_every_tier() {
        // a tier that emits a secret in its response — masked + audited on BOTH local and egress.
        for egress in [false, true] {
            let sink = Arc::new(VecAuditSink::default());
            let mw = PrivacyMiddleware::new(Arc::new(Redactor::new(vec![])), egress, Some(sink.clone()));
            let chain = Chain::new(vec![Arc::new(mw)]);
            let res = chain.run(req_with("hi"), &Context::default(), Arc::new(PiiTier)).await.unwrap();
            assert!(!res.content.contains("sk-deadbeef99"), "output secret masked (egress={egress})");
            assert!(res.content.contains("[REDACTED]"));
            assert!(
                sink.events.lock().unwrap().iter().any(|e| e.code == "REDACTION" && e.redactions.iter().any(|r| r.contains("api_key"))),
                "output redaction is I1-audited (egress={egress})"
            );
        }
    }
}

#[cfg(test)]
mod rung3_tests {
    use super::{PiiClassifier, PiiSpan, Redactor};
    use std::sync::Arc;

    /// A stub rung-3: flags every occurrence of "Dana" as a private_person span.
    struct DanaStub;
    impl PiiClassifier for DanaStub {
        fn spans(&self, text: &str) -> Vec<PiiSpan> {
            text.match_indices("Dana")
                .map(|(s, m)| PiiSpan { start: s, end: s + m.len(), class: "private_person".into() })
                .collect()
        }
    }

    /// A stub that returns garbage offsets - the mask must drop them, never panic.
    struct BadOffsets;
    impl PiiClassifier for BadOffsets {
        fn spans(&self, text: &str) -> Vec<PiiSpan> {
            vec![
                PiiSpan { start: 5, end: 3, class: "x".into() },
                PiiSpan { start: 0, end: text.len() + 40, class: "x".into() },
                PiiSpan { start: 1, end: 2, class: "mid-char".into() },
            ]
        }
    }

    #[test]
    fn rung3_adds_a_span_rungs_1_2_miss_and_is_additive_only() {
        // the GOLDEN_PRIVACY mechanic: "...met Dana at..." carries no marker/regex hit -
        // deterministic-only passes it through; the model-assisted path masks it (rung 3).
        let text = "yesterday I met Dana at the harbor office";
        let plain = Redactor::new(vec![]);
        let (kept, none) = plain.redact(text);
        assert_eq!(kept, text);
        assert!(none.is_empty(), "rungs 1-2 cannot catch an arbitrary name");
        let r = Redactor::new(vec![]).with_classifier(Arc::new(DanaStub));
        // the deterministic path NEVER consults the model (the guarantee stays deterministic).
        let (still, f0) = r.redact(text);
        assert_eq!(still, text);
        assert!(f0.is_empty());
        // the model-assisted path masks the model span and reports it as rung 3.
        let (scrubbed, findings) = r.redact_with_model(text);
        assert!(!scrubbed.contains("Dana"), "the model span is masked: {scrubbed}");
        assert!(scrubbed.contains("[REDACTED]"));
        assert!(findings.iter().any(|f| f.rung == 3 && f.kind == "private_person"));
    }

    #[test]
    fn rung3_overlap_with_rung2_merges_and_both_findings_report() {
        // an email (rung 2) sitting where the model also flags a person - one mask, two findings.
        let text = "write to Dana dana@example.test today";
        let r = Redactor::new(vec![]).with_classifier(Arc::new(DanaStub));
        let (scrubbed, findings) = r.redact_with_model(text);
        assert!(!scrubbed.contains("dana@example.test"));
        assert!(!scrubbed.contains("Dana"));
        assert!(findings.iter().any(|f| f.rung == 2 && f.kind == "email"));
        assert!(findings.iter().any(|f| f.rung == 3));
    }

    #[test]
    fn invalid_model_offsets_are_dropped_never_panicking() {
        let text = "\u{00e9}clair time"; // multi-byte first char: offset 1 is mid-char
        let r = Redactor::new(vec![]).with_classifier(Arc::new(BadOffsets));
        let (scrubbed, findings) = r.redact_with_model(text);
        assert_eq!(scrubbed, text, "all garbage spans dropped -> unchanged");
        assert!(findings.is_empty());
    }
}
