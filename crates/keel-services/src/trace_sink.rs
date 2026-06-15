//! keel-services::trace_sink — the flywheel feed (canon §8 step 9 / §5 reversibility gate).
//!
//! A passed verdict is the only thing that becomes distillation feedstock (the local model learning
//! from its own verified wins — the §0 flywheel). [`FileTraceSink`] appends each [`VerifiedTrace`] to
//! an append-only JSONL **distill corpus** — but **scrubs secrets/PII first**. That scrub is the
//! reversibility gate (canon §5): *a secret baked into a distilled LoRA is irreversible* (undo =
//! retrain), so the trace→distill path MUST scrub before the trace is ever feedstock. It reuses the
//! same [`Redactor`] (rungs 1–2) the I3 egress mask uses — **one source of truth** for what counts as
//! a secret, so the corpus can never carry what egress would have masked.
//!
//! KEEL only **emits + stores** the scrubbed corpus; the LoRA training is out-of-band (Unsloth Studio,
//! an operator step the core §16-refuses) — this is the hand-off feedstock, distill-ready.

use async_trait::async_trait;
use keel_contracts::{Content, KeelError, Result, TraceSink, VerifiedTrace};
use keel_middleware::Redactor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A file-backed [`TraceSink`]: passed verdicts → an append-only JSONL distill corpus, **secrets/PII
/// scrubbed before write** (the reversibility gate). Crash-safe per line; serializes appends within a
/// process. The redactor is shared with the I3 privacy rung (one definition of "secret").
pub struct FileTraceSink {
    /// The append-only distill corpus (JSONL of scrubbed `VerifiedTrace`s).
    path: PathBuf,
    /// The shared deterministic redactor (rungs 1–2) — the same one the egress mask uses.
    redactor: Arc<Redactor>,
    /// Serializes appends within a process (the corpus is also crash-safe per JSONL line).
    lock: Mutex<()>,
}

impl FileTraceSink {
    /// Build a corpus sink. `redactor` should be the same one wired into the I3 privacy rung so the
    /// distill scrub and the egress mask agree on what a secret is.
    pub fn new(path: impl Into<PathBuf>, redactor: Arc<Redactor>) -> Self {
        Self { path: path.into(), redactor, lock: Mutex::new(()) }
    }

    /// Scrub every text-carrying field of a trace — the prompt (`step.content`) and the completion
    /// (`result.content` + `reasoning_content`), i.e. exactly the (prompt, completion) pair a distill
    /// run would train on — through the redactor, so no secret/PII becomes feedstock (reversibility gate).
    fn scrub(&self, mut vt: VerifiedTrace) -> VerifiedTrace {
        for c in &mut vt.trace.step.content {
            if let Content::Text { text } = c {
                *text = self.redactor.redact(text).0;
            }
        }
        vt.trace.result.content = self.redactor.redact(&vt.trace.result.content).0;
        if let Some(rc) = vt.trace.result.reasoning_content.take() {
            vt.trace.result.reasoning_content = Some(self.redactor.redact(&rc).0);
        }
        vt
    }
}

#[async_trait]
impl TraceSink for FileTraceSink {
    async fn emit(&self, trace: VerifiedTrace) -> Result<()> {
        use std::io::Write;
        let scrubbed = self.scrub(trace); // reversibility gate: scrub BEFORE it is ever feedstock
        let line = serde_json::to_string(&scrubbed).map_err(|e| KeelError::Other(format!("trace encode: {e}")))?;
        // poison-tolerant: a panicked holder must not wedge the corpus.
        let _g = self.lock.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(dir) = self.path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir).map_err(|e| KeelError::Other(format!("traces dir: {e}")))?;
            }
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| KeelError::Other(format!("traces open: {e}")))?;
        writeln!(f, "{line}").map_err(|e| KeelError::Other(format!("traces write: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::{
        Content, DataClass, Decision, GenerateResult, Kind, Step, Trace, Trust, Verdict,
    };

    fn step_with(text: &str) -> Step {
        Step {
            kind: Kind::Scaffolding,
            ty: "t".into(),
            trust_required: Trust::Normal,
            data_class: DataClass::Normal,
            tier_history: vec![],
            oracle_failures: 0,
            projected_cost: None,
            critical: false,
            source: None,
            content: vec![Content::Text { text: text.into() }],
            golden_refs: vec![],
        }
    }

    fn verified(prompt: &str, answer: &str, reasoning: Option<&str>) -> VerifiedTrace {
        VerifiedTrace {
            trace: Trace {
                step: step_with(prompt),
                decision: Decision { tier: "local".into(), effort: Default::default(), reason: "t".into() },
                result: GenerateResult {
                    content: answer.into(),
                    reasoning_content: reasoning.map(Into::into),
                    tier: "local".into(),
                    ..Default::default()
                },
                verdict: Verdict { passed: true, ..Default::default() },
            },
        }
    }

    fn temp_corpus(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("keel-traces-{}-{}.jsonl", tag, std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[tokio::test]
    async fn emit_scrubs_secrets_and_pii_before_writing() {
        let path = temp_corpus("scrub");
        let sink = FileTraceSink::new(&path, Arc::new(Redactor::new(vec![])));
        // a key in the completion, an email in the reasoning, an ssn in the prompt — none may survive.
        sink.emit(verified(
            "my ssn is 123-45-6789",
            "your key is sk-abcd1234 ok",
            Some("contact a@b.com"),
        ))
        .await
        .unwrap();

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(!body.contains("sk-abcd1234"), "secret key scrubbed from the completion");
        assert!(!body.contains("123-45-6789"), "ssn scrubbed from the prompt");
        assert!(!body.contains("a@b.com"), "email scrubbed from the reasoning");
        assert!(body.contains("[REDACTED]"), "redaction placeholder present");
        // the trace structure survives the scrub (it's still a usable training record).
        let vt: VerifiedTrace = serde_json::from_str(body.trim()).unwrap();
        assert!(vt.trace.verdict.passed);
        assert_eq!(vt.trace.result.tier, "local");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn emit_appends_one_line_per_trace_and_preserves_clean_text() {
        let path = temp_corpus("append");
        let sink = FileTraceSink::new(&path, Arc::new(Redactor::new(vec![])));
        sink.emit(verified("capital of France?", "Paris", None)).await.unwrap();
        sink.emit(verified("2+2?", "4", None)).await.unwrap();

        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body.lines().count(), 2, "one JSONL line per emitted trace");
        // clean text (no PII) is written verbatim — the corpus is faithful when there's nothing to scrub.
        assert!(body.contains("Paris") && body.contains("capital of France"));
        let _ = std::fs::remove_file(&path);
    }
}
