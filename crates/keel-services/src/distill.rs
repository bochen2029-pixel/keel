//! keel-services::distill — the out-of-band distillation hand-off (canon §16 / Stage 3).
//!
//! KEEL only **emits + stores + exports** the verified corpus; the LoRA training itself is **external**
//! (Unsloth Studio) — the core §16-*refuses* the trainer (it is a vertical's heavy apparatus, not the
//! genome). [`FileTraceSink`](crate::FileTraceSink) writes the **secret-scrubbed** corpus (the
//! reversibility gate, §5); this module turns that corpus into clean **chat-format training pairs**
//! (`{messages:[user, assistant]}` JSONL) that an out-of-band trainer consumes directly. Because the
//! corpus is already scrubbed at write time, the exported pairs are distill-ready — no secret reaches
//! the trainer.
//!
//! The training signal is exactly the verified win: the user prompt (`step.content`) → the assistant
//! completion (`result.content`) the oracle passed. The system/memory preamble is assembled at runtime
//! (not stored in the trace), so a pair carries only what the local model should learn to produce.

use keel_contracts::{Content, VerifiedTrace};
use serde_json::{json, Value};

/// A chat-format training pair from one verified trace: the user prompt → the assistant completion.
/// `None` when either side is empty (nothing to learn from). The corpus is already secret-scrubbed by
/// the `TraceSink`, so the pair is distill-ready as-is.
pub fn training_pair(vt: &VerifiedTrace) -> Option<Value> {
    let prompt = vt.trace.step.content.iter().find_map(|c| match c {
        Content::Text { text } if !text.trim().is_empty() => Some(text.trim().to_string()),
        _ => None,
    })?;
    let completion = vt.trace.result.content.trim();
    if completion.is_empty() {
        return None;
    }
    Some(json!({
        "messages": [
            { "role": "user", "content": prompt },
            { "role": "assistant", "content": completion },
        ]
    }))
}

/// Read a scrubbed corpus (JSONL of `VerifiedTrace`) and emit **chat-format training-pair JSONL** — one
/// `{messages:[…]}` line per usable trace. Unparseable lines and empty pairs are skipped (best-effort;
/// the corpus outlives schema). The output is what an out-of-band trainer (Unsloth) consumes directly.
pub fn export_training_jsonl(corpus: &str) -> String {
    corpus
        .lines()
        .filter_map(|l| serde_json::from_str::<VerifiedTrace>(l).ok())
        .filter_map(|vt| training_pair(&vt))
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::{DataClass, Decision, GenerateResult, Kind, Step, Trace, Trust, Verdict};

    fn vt(prompt: &str, completion: &str) -> VerifiedTrace {
        VerifiedTrace {
            trace: Trace {
                step: Step {
                    kind: Kind::Scaffolding,
                    ty: "t".into(),
                    trust_required: Trust::Normal,
                    data_class: DataClass::Normal,
                    tier_history: vec![],
                    oracle_failures: 0,
                    projected_cost: None,
                    critical: false,
                    source: None,
                    content: if prompt.is_empty() { vec![] } else { vec![Content::Text { text: prompt.into() }] },
                    golden_refs: vec![],
                },
                decision: Decision { tier: "local".into(), effort: Default::default(), reason: "t".into() },
                result: GenerateResult { content: completion.into(), tier: "local".into(), ..Default::default() },
                verdict: Verdict { passed: true, ..Default::default() },
            },
        }
    }

    #[test]
    fn training_pair_is_chat_format_user_then_assistant() {
        let p = training_pair(&vt("capital of France?", "Paris")).expect("a usable pair");
        let msgs = p["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "capital of France?");
        assert_eq!(msgs[1]["role"], "assistant");
        assert_eq!(msgs[1]["content"], "Paris");
    }

    #[test]
    fn empty_prompt_or_completion_yields_no_pair() {
        assert!(training_pair(&vt("", "Paris")).is_none(), "no prompt -> nothing to learn");
        assert!(training_pair(&vt("q", "   ")).is_none(), "empty completion -> nothing to learn");
    }

    #[test]
    fn export_skips_bad_lines_and_empty_pairs() {
        let corpus = [
            serde_json::to_string(&vt("q1", "a1")).unwrap(),
            "{ not json".to_string(),               // unparseable -> skipped
            serde_json::to_string(&vt("", "orphan")).unwrap(), // empty prompt -> skipped
            serde_json::to_string(&vt("q2", "a2")).unwrap(),
        ]
        .join("\n");
        let out = export_training_jsonl(&corpus);
        assert_eq!(out.lines().count(), 2, "only the two usable pairs are exported");
        assert!(out.contains("\"a1\"") && out.contains("\"a2\""));
        assert!(!out.contains("orphan"));
    }
}
