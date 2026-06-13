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

use async_trait::async_trait;
use keel_contracts::{Content, Context, GenerateRequest, GenerateResult, Middleware, Next, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A detected PII span's classification (not its value — the secret isn't retained here).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub kind: String,
    /// 1 = operator marker, 2 = structured regex/checksum.
    pub rung: u8,
}

/// The deterministic redactor (rungs 1–2). Compiles its patterns once.
pub struct Redactor {
    markers: Vec<String>,
    patterns: Vec<(&'static str, Regex)>,
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
        Self { markers, patterns }
    }

    /// Redact every marker/PII span (the union) with `[REDACTED]`; return the scrubbed text and
    /// the findings. Empty findings ⇒ the text is returned unchanged.
    pub fn redact(&self, text: &str) -> (String, Vec<Finding>) {
        // (start, end, kind, rung)
        let mut spans: Vec<(usize, usize, &str, u8)> = Vec::new();

        // rung 1 — operator markers (exact, case-sensitive)
        for m in self.markers.iter().filter(|m| !m.is_empty()) {
            let mut from = 0;
            while let Some(rel) = text[from..].find(m.as_str()) {
                let s = from + rel;
                let e = s + m.len();
                spans.push((s, e, "operator_marker", 1));
                from = e;
            }
        }

        // rung 2 — structured patterns (credit cards must pass Luhn)
        for (kind, re) in &self.patterns {
            for mat in re.find_iter(text) {
                if *kind == "credit_card" && !luhn_ok(mat.as_str()) {
                    continue;
                }
                spans.push((mat.start(), mat.end(), kind, 2));
            }
        }

        if spans.is_empty() {
            return (text.to_string(), Vec::new());
        }

        let findings = spans
            .iter()
            .map(|(_, _, k, r)| Finding { kind: (*k).to_string(), rung: *r })
            .collect();

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

/// I3 mask middleware. On an **egress** call (a non-local tier) it scrubs the request's text
/// before it leaves the box; on a local call it passes through (local is sovereign-safe). The
/// engine constructs it with `egress = true` only when the terminal tier leaves the box.
pub struct PrivacyMiddleware {
    redactor: Arc<Redactor>,
    egress: bool,
}

impl PrivacyMiddleware {
    pub fn new(redactor: Arc<Redactor>, egress: bool) -> Self {
        Self { redactor, egress }
    }
}

#[async_trait]
impl Middleware for PrivacyMiddleware {
    async fn handle(&self, mut req: GenerateRequest, ctx: &Context, next: &dyn Next) -> Result<GenerateResult> {
        if self.egress {
            for msg in &mut req.messages {
                for content in &mut msg.content {
                    if let Content::Text { text } = content {
                        let (scrubbed, _findings) = self.redactor.redact(text);
                        *text = scrubbed;
                    }
                }
            }
        }
        next.run(req, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::{PrivacyMiddleware, Redactor};
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
        let mw = PrivacyMiddleware::new(Arc::new(Redactor::new(vec![])), egress);
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
}
