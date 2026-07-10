//! svc::privacy_model — rung 3 of the I3 mask (canon §5.1, A5): the OpenAI privacy filter as an
//! in-process CPU classifier behind `mw::privacy`'s [`PiiClassifier`] seam. **Additive recall,
//! never the guarantee** — rungs 1–2 stay the deterministic oracle; this model can only ADD
//! masked spans on the egress path. Feature-gated (`privacy-model`): the default genome pulls
//! neither `ort` nor `tokenizers`.
//!
//! The pipeline: HF tokenizer (offsets on) → ONNX token-classification logits `[1, L, 33]` →
//! **KEEL-owned Viterbi** over the BIOES label graph with the repo's calibrated transition
//! biases (+ a recall bias on background→start — the keel.lock `operating_point: recall` knob,
//! a NON-model dial) → byte-offset [`PiiSpan`]s. The decode is deterministic given the logits.

use keel_contracts::{KeelError, Result};
use keel_middleware::{PiiClassifier, PiiSpan};
use std::path::Path;
use std::sync::Mutex;

/// A BIOES tag, parsed from the model's `id2label` (`O` / `B-class` / `I-class` / `E-class` /
/// `S-class`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tag {
    O,
    B,
    I,
    E,
    S,
}

/// One label = a tag + a class index (`O` carries class 0, unused).
#[derive(Clone, Debug)]
pub(crate) struct LabelSet {
    pub tags: Vec<(Tag, usize)>,
    pub classes: Vec<String>,
}

impl LabelSet {
    /// Parse `id2label` (index-ordered) into the tag/class table.
    pub fn parse(id2label: &[String]) -> Result<Self> {
        let mut classes: Vec<String> = Vec::new();
        let mut tags = Vec::with_capacity(id2label.len());
        for l in id2label {
            if l == "O" {
                tags.push((Tag::O, 0));
                continue;
            }
            let (t, c) = l
                .split_once('-')
                .ok_or_else(|| KeelError::Other(format!("privacy label without tag: {l}")))?;
            let tag = match t {
                "B" => Tag::B,
                "I" => Tag::I,
                "E" => Tag::E,
                "S" => Tag::S,
                _ => return Err(KeelError::Other(format!("unknown BIOES tag in {l}"))),
            };
            let cid = match classes.iter().position(|x| x == c) {
                Some(i) => i,
                None => {
                    classes.push(c.to_string());
                    classes.len() - 1
                }
            };
            tags.push((tag, cid));
        }
        Ok(Self { tags, classes })
    }
}

/// The calibrated transition biases (`viterbi_calibration.json` — the repo ships an all-zero
/// "default" operating point; the recall dial adds to background→start).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Biases {
    pub background_stay: f32,
    pub background_to_start: f32,
    pub end_to_background: f32,
    pub end_to_start: f32,
    pub inside_to_continue: f32,
    pub inside_to_end: f32,
}

const NEG_INF: f32 = f32::NEG_INFINITY;

/// The BIOES transition score `from → to` under the calibration (disallowed = `-inf`).
fn transition(labels: &LabelSet, b: &Biases, from: usize, to: usize) -> f32 {
    let (ft, fc) = labels.tags[from];
    let (tt, tc) = labels.tags[to];
    match (ft, tt) {
        (Tag::O, Tag::O) => b.background_stay,
        (Tag::O, Tag::B) | (Tag::O, Tag::S) => b.background_to_start,
        (Tag::B, Tag::I) | (Tag::B, Tag::E) if fc == tc => {
            if tt == Tag::I { b.inside_to_continue } else { b.inside_to_end }
        }
        (Tag::I, Tag::I) | (Tag::I, Tag::E) if fc == tc => {
            if tt == Tag::I { b.inside_to_continue } else { b.inside_to_end }
        }
        (Tag::E, Tag::O) | (Tag::S, Tag::O) => b.end_to_background,
        (Tag::E, Tag::B) | (Tag::E, Tag::S) | (Tag::S, Tag::B) | (Tag::S, Tag::S) => b.end_to_start,
        _ => NEG_INF,
    }
}

/// Viterbi over `logits` (row-major `[tokens × labels]`): the max-score label path under the
/// BIOES transition graph. Paths may start in {O, B, S} and must END in {O, E, S} (no dangling
/// span). Deterministic — the whole decode is a non-model computation over the logits.
pub(crate) fn viterbi(logits: &[f32], n_tokens: usize, labels: &LabelSet, b: &Biases) -> Vec<usize> {
    let n = labels.tags.len();
    if n_tokens == 0 || logits.len() < n_tokens * n {
        return Vec::new();
    }
    let start_ok = |l: usize| matches!(labels.tags[l].0, Tag::O | Tag::B | Tag::S);
    let end_ok = |l: usize| matches!(labels.tags[l].0, Tag::O | Tag::E | Tag::S);
    let mut score = vec![NEG_INF; n];
    let mut back: Vec<Vec<usize>> = vec![vec![0; n]; n_tokens];
    for (l, s) in score.iter_mut().enumerate() {
        if start_ok(l) {
            // sequence-start counts as background: starting a span pays/earns the same
            // background->start bias as mid-sequence, so the recall dial works on short texts too.
            let start_bias = if labels.tags[l].0 == Tag::O { 0.0 } else { b.background_to_start };
            *s = logits[l] + start_bias;
        }
    }
    for t in 1..n_tokens {
        let mut next = vec![NEG_INF; n];
        for to in 0..n {
            let emit = logits[t * n + to];
            let mut best = NEG_INF;
            let mut arg = 0;
            for (from, &fs) in score.iter().enumerate() {
                if fs == NEG_INF {
                    continue;
                }
                let tr = transition(labels, b, from, to);
                if tr == NEG_INF {
                    continue;
                }
                let s = fs + tr;
                if s > best {
                    best = s;
                    arg = from;
                }
            }
            if best > NEG_INF {
                next[to] = best + emit;
                back[t][to] = arg;
            }
        }
        score = next;
    }
    // best terminal state among the end-legal tags (fall back to global best if none survive).
    let mut end = 0;
    let mut best = NEG_INF;
    for (l, &s) in score.iter().enumerate() {
        if s > best && (end_ok(l) || n_tokens == 1) {
            best = s;
            end = l;
        }
    }
    if best == NEG_INF {
        for (l, s) in score.iter().enumerate() {
            if *s > best {
                best = *s;
                end = l;
            }
        }
    }
    let mut path = vec![0usize; n_tokens];
    path[n_tokens - 1] = end;
    for t in (1..n_tokens).rev() {
        path[t - 1] = back[t][path[t]];
    }
    path
}

/// Collapse a BIOES label path into `(token_start, token_end_inclusive, class_id)` spans.
pub(crate) fn path_to_spans(path: &[usize], labels: &LabelSet) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    let mut open: Option<(usize, usize)> = None; // (start_token, class)
    for (t, &l) in path.iter().enumerate() {
        let (tag, c) = labels.tags[l];
        match tag {
            Tag::S => {
                open = None;
                out.push((t, t, c));
            }
            Tag::B => open = Some((t, c)),
            Tag::I => {} // continuation; validity is the transition graph's job
            Tag::E => {
                if let Some((s, oc)) = open.take() {
                    if oc == c {
                        out.push((s, t, c));
                    }
                }
            }
            Tag::O => open = None,
        }
    }
    out
}

/// The ort-backed rung-3 classifier (`openai/privacy-filter`, quantized-CPU ONNX). Loads once;
/// `spans` runs tokenizer → session → Viterbi per call. The session is behind a `Mutex` (ort
/// `run` is `&mut`; rung 3 rides the egress path, which is not hot).
pub struct OnnxPiiClassifier {
    session: Mutex<ort::session::Session>,
    tokenizer: tokenizers::Tokenizer,
    labels: LabelSet,
    biases: Biases,
    input_ids_name: String,
    attention_name: String,
}

/// Defensive latency cap: egress prompts are far smaller, but never hand the model an unbounded
/// sequence (131k positions exist; we don't want the latency).
const MAX_TOKENS: usize = 8192;

impl OnnxPiiClassifier {
    /// Load from the model directory (`C:\models\privacy-filter` shape): the quantized ONNX graph
    /// (+ its external-data sibling, resolved by onnxruntime relative to the graph), the HF
    /// tokenizer, `config.json`'s `id2label`, and the calibrated Viterbi biases. `recall_bias`
    /// is added to background→start (the keel.lock `operating_point: recall` dial); 0.0 = the
    /// repo's shipped calibration.
    pub fn load(dir: &Path, recall_bias: f32) -> Result<Self> {
        let graph = dir.join("onnx").join("model_quantized.onnx");
        let session = ort::session::Session::builder()
            .and_then(|mut b| b.commit_from_file(&graph))
            .map_err(|e| KeelError::Other(format!("privacy-model onnx load {}: {e}", graph.display())))?;
        // inspect, don't assume: resolve the two tensor names from the graph itself.
        let names: Vec<String> = session.inputs().iter().map(|i| i.name().to_string()).collect();
        let pick = |needle: &str| -> Result<String> {
            names
                .iter()
                .find(|n| n.contains(needle))
                .cloned()
                .ok_or_else(|| KeelError::Other(format!("privacy-model graph has no '{needle}' input; inputs = {names:?}")))
        };
        let input_ids_name = pick("input_ids")?;
        let attention_name = pick("attention_mask")?;
        let tokenizer = tokenizers::Tokenizer::from_file(dir.join("tokenizer.json"))
            .map_err(|e| KeelError::Other(format!("privacy-model tokenizer: {e}")))?;
        let cfg: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("config.json"))
                .map_err(|e| KeelError::Other(format!("privacy-model config: {e}")))?,
        )
        .map_err(|e| KeelError::Other(format!("privacy-model config parse: {e}")))?;
        let id2label_map = cfg["id2label"]
            .as_object()
            .ok_or_else(|| KeelError::Other("privacy-model config: no id2label".into()))?;
        let mut id2label = vec![String::new(); id2label_map.len()];
        for (k, v) in id2label_map {
            let i: usize = k.parse().map_err(|_| KeelError::Other(format!("id2label key {k}")))?;
            id2label[i] = v.as_str().unwrap_or_default().to_string();
        }
        let labels = LabelSet::parse(&id2label)?;
        let mut biases = Biases::default();
        if let Ok(raw) = std::fs::read_to_string(dir.join("viterbi_calibration.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                let b = &v["operating_points"]["default"]["biases"];
                let f = |k: &str| b[k].as_f64().unwrap_or(0.0) as f32;
                biases = Biases {
                    background_stay: f("transition_bias_background_stay"),
                    background_to_start: f("transition_bias_background_to_start"),
                    end_to_background: f("transition_bias_end_to_background"),
                    end_to_start: f("transition_bias_end_to_start"),
                    inside_to_continue: f("transition_bias_inside_to_continue"),
                    inside_to_end: f("transition_bias_inside_to_end"),
                };
            }
        }
        biases.background_to_start += recall_bias;
        Ok(Self { session: Mutex::new(session), tokenizer, labels, biases, input_ids_name, attention_name })
    }

    fn classify(&self, text: &str) -> Result<Vec<PiiSpan>> {
        let enc = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| KeelError::Other(format!("privacy-model tokenize: {e}")))?;
        let ids: Vec<i64> = enc.get_ids().iter().take(MAX_TOKENS).map(|&u| u as i64).collect();
        let n = ids.len();
        if n == 0 {
            return Ok(Vec::new());
        }
        let offsets = enc.get_offsets();
        let mask: Vec<i64> = vec![1; n];
        let ids_v = ort::value::Value::from_array(([1usize, n], ids))
            .map_err(|e| KeelError::Other(format!("privacy-model input_ids: {e}")))?;
        let mask_v = ort::value::Value::from_array(([1usize, n], mask))
            .map_err(|e| KeelError::Other(format!("privacy-model attention: {e}")))?;
        let mut session = self.session.lock().unwrap_or_else(|p| p.into_inner());
        let outputs = session
            .run(ort::inputs![self.input_ids_name.as_str() => ids_v, self.attention_name.as_str() => mask_v])
            .map_err(|e| KeelError::Other(format!("privacy-model run: {e}")))?;
        let (shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| KeelError::Other(format!("privacy-model logits: {e}")))?;
        let n_labels = self.labels.tags.len();
        let dims: Vec<i64> = shape.iter().copied().collect();
        if *dims.last().unwrap_or(&0) as usize != n_labels {
            return Err(KeelError::Other(format!("privacy-model logits shape {dims:?} vs {n_labels} labels")));
        }
        let path = viterbi(data, n, &self.labels, &self.biases);
        let mut out = Vec::new();
        for (ts, te, c) in path_to_spans(&path, &self.labels) {
            // token range -> byte range; zero-width (special) tokens never anchor a span edge.
            let (mut s, mut e) = (usize::MAX, 0usize);
            for &(os, oe) in offsets.iter().take(te + 1).skip(ts) {
                if os == oe {
                    continue;
                }
                s = s.min(os);
                e = e.max(oe);
            }
            if s < e {
                out.push(PiiSpan { start: s, end: e, class: self.labels.classes[c].clone() });
            }
        }
        Ok(out)
    }
}

impl PiiClassifier for OnnxPiiClassifier {
    /// Best-effort by contract: a rung-3 failure returns NO spans (rungs 1–2 still ran; the
    /// additive pass must never take the turn down with it). Errors are stderr-visible.
    fn spans(&self, text: &str) -> Vec<PiiSpan> {
        match self.classify(text) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[keel] privacy rung-3 degraded for this call: {e}");
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> LabelSet {
        LabelSet::parse(&[
            "O".into(),
            "B-private_person".into(),
            "I-private_person".into(),
            "E-private_person".into(),
            "S-private_person".into(),
        ])
        .unwrap()
    }

    #[test]
    fn label_parse_and_transition_graph() {
        let l = labels();
        assert_eq!(l.classes, vec!["private_person".to_string()]);
        let b = Biases::default();
        assert_eq!(transition(&l, &b, 0, 0), 0.0); // O->O
        assert_eq!(transition(&l, &b, 0, 1), 0.0); // O->B
        assert_eq!(transition(&l, &b, 1, 2), 0.0); // B->I same class
        assert_eq!(transition(&l, &b, 2, 3), 0.0); // I->E
        assert_eq!(transition(&l, &b, 1, 0), NEG_INF); // B->O illegal (no dangling span)
        assert_eq!(transition(&l, &b, 0, 2), NEG_INF); // O->I illegal
    }

    #[test]
    fn viterbi_decodes_a_bie_span_and_recall_bias_tips_a_borderline_start() {
        let l = labels();
        // 3 tokens x 5 labels; strong B, I, E evidence on tokens 0..2.
        #[rustfmt::skip]
        let logits = vec![
            0.0, 5.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 5.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 5.0, 0.0,
        ];
        let path = viterbi(&logits, 3, &l, &Biases::default());
        assert_eq!(path, vec![1, 2, 3]);
        assert_eq!(path_to_spans(&path, &l), vec![(0, 2, 0)]);
        // borderline single token: O 1.0 vs S 0.9 -> O by default; a +0.2 recall bias tips it to S.
        let border = vec![1.0, 0.0, 0.0, 0.0, 0.9];
        assert_eq!(viterbi(&border, 1, &l, &Biases::default()), vec![0]);
        let recall = Biases { background_to_start: 0.2, ..Default::default() };
        assert_eq!(viterbi(&border, 1, &l, &recall), vec![4], "the operating point is a non-model dial");
    }

    #[test]
    fn path_to_spans_handles_s_runs_and_ignores_mismatched_e() {
        let l = labels();
        assert_eq!(path_to_spans(&[4, 0, 4], &l), vec![(0, 0, 0), (2, 2, 0)]);
        assert_eq!(path_to_spans(&[3], &l), vec![], "an E with no open B never emits");
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;
    use keel_middleware::Redactor;
    use std::sync::Arc;

    fn model_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(r"C:\models\privacy-filter")
    }

    /// GOLDEN_PRIVACY case 2, LIVED: "third-party name in context the model adds over
    /// deterministic" -> `model_adds_span: true`. Deterministic-only must miss the name; the
    /// model-assisted path must mask it and report rung 3.
    #[test]
    #[ignore = "requires the 1.6 GB privacy-filter model on disk"]
    fn live_golden_privacy_model_adds_span() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/golden.json");
        let golden: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let case = golden["privacy"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["expect"].get("model_adds_span").is_some())
            .expect("the rung-3 golden case");
        assert_eq!(case["expect"]["model_adds_span"].as_bool(), Some(true));
        let text = "yesterday I met Dana at the harbor office"; // the case''s "...met Dana at..." family
        let clf = OnnxPiiClassifier::load(&model_dir(), 0.0).expect("model loads");
        let r = Redactor::new(vec![]).with_classifier(Arc::new(clf));
        let (det, det_findings) = r.redact(text);
        assert_eq!(det, text);
        assert!(det_findings.is_empty(), "rungs 1-2 alone must miss an arbitrary name");
        let (masked, findings) = r.redact_with_model(text);
        assert!(!masked.contains("Dana"), "the model must add the span; got: {masked}");
        assert!(findings.iter().any(|f| f.rung == 3), "a rung-3 finding must report: {findings:?}");
        println!("GOLDEN_PRIVACY model_adds_span LIVED: '{masked}' findings={findings:?}");
    }

    /// The C3 decision bench (pre-registered thresholds ride the fixture): deterministic-only vs
    /// +model recall on positives, FP on clean negatives, per-call latency. Writes the artifact;
    /// asserts nothing - the DECISION is the session''s, recorded in WORKLOG/keel.lock.
    #[test]
    #[ignore = "requires the model; writes .keelstate/bench/privacy-c3.json"]
    fn live_c3_bench() {
        let fx_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/privacy/rung3-set.json");
        let fx: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(fx_path).unwrap()).unwrap();
        let items = fx["items"].as_array().unwrap();
        let t_load = std::time::Instant::now();
        let clf = OnnxPiiClassifier::load(&model_dir(), 0.0).expect("model loads");
        let load_ms = t_load.elapsed().as_millis() as u64;
        let r = Redactor::new(vec![]).with_classifier(Arc::new(clf));
        let (mut det_hits, mut model_hits, mut pos, mut fp, mut neg) = (0u32, 0u32, 0u32, 0u32, 0u32);
        let mut ms: Vec<u64> = Vec::new();
        let mut rows = Vec::new();
        for it in items {
            let text = it["text"].as_str().unwrap();
            let t0 = std::time::Instant::now();
            let (masked, _f) = r.redact_with_model(text);
            ms.push(t0.elapsed().as_millis() as u64);
            if it["clean"].as_bool() == Some(true) {
                neg += 1;
                let false_pos = masked != text;
                if false_pos {
                    fp += 1;
                }
                rows.push(serde_json::json!({"id": it["id"], "clean": true, "false_positive": false_pos, "masked": masked}));
            } else {
                pos += 1;
                let pii = it["pii"].as_str().unwrap();
                let (det, _)= r.redact(text);
                let d = !det.contains(pii);
                let m = !masked.contains(pii);
                det_hits += d as u32;
                model_hits += m as u32;
                rows.push(serde_json::json!({"id": it["id"], "pii_caught_deterministic": d, "pii_caught_with_model": m, "masked": masked}));
            }
        }
        ms.sort_unstable();
        let pct = |p: f64| ms[(((p / 100.0) * ms.len() as f64).ceil().max(1.0) as usize).min(ms.len()) - 1];
        let det_recall = det_hits as f64 / pos as f64;
        let model_recall = model_hits as f64 / pos as f64;
        let report = serde_json::json!({
            "set": fx["name"], "version": fx["version"], "thresholds": fx["thresholds"],
            "positives": pos, "negatives": neg,
            "deterministic_recall": det_recall, "with_model_recall": model_recall,
            "uplift": model_recall - det_recall,
            "fp_rate": fp as f64 / neg as f64,
            "load_ms": load_ms, "p50_ms": pct(50.0), "p95_ms": pct(95.0),
            "items": rows,
        });
        let out = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.keelstate/bench/privacy-c3.json");
        std::fs::create_dir_all(std::path::Path::new(out).parent().unwrap()).unwrap();
        std::fs::write(out, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        println!(
            "C3: det_recall={det_recall:.2} with_model={model_recall:.2} uplift={:+.2} fp={}/{neg} p50/p95={}/{} ms load={} ms -> {out}",
            model_recall - det_recall, fp, pct(50.0), pct(95.0), load_ms
        );
    }
}
