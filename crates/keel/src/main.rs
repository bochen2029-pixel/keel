//! keel — the daily-driver CLI (L5). By default it **self-drives**: it builds the multi-tier
//! [`Engine`](keel::Engine), lets the router pick the tier for the turn (scaffolding → local ·
//! core-wire → cheap-API · escalate → frontier), runs it through the invariant chain, and prints
//! the answer + the routing reason. `--tier` pins a tier manually (the single-tier path).
//!
//!   keel "read me the config"                  # → routed (scaffolding → local)
//!   keel --kind core-wire "weigh the tradeoffs"# → routed (core-wire → cheap-API)
//!   keel --sovereign "summarize my journal"    # → forced local (I3)
//!   keel --tier frontier "hard question"       # → manual override (no router)
//!   keel --think "…"      keel --manifest C:\KEEL\keel.lock "hello"

use keel_contracts::{
    Content, Context, DataClass, Driver, Effort, GenerateRequest, GenerateResult, Kind, Message, Role, Step, Trust,
};
use keel_kernel::{new_context, Manifest};
use keel_services::{HeartbeatDriver, WatchDriver};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("keel: {e}");
        std::process::exit(1);
    }
}

async fn run() -> keel_contracts::Result<()> {
    // `keel metrics` — an off-loop, read-only rollup over the I2 index (no manifest, no tier, no loop).
    if std::env::args().nth(1).as_deref() == Some("metrics") {
        let store = keel_store::SqliteStore::open(keel::INDEX_DB)?;
        print_metrics(&store.metrics()?);
        return Ok(());
    }
    // `keel daemon [...]` — the self-driving select-loop (canon §8): wire the heartbeat (+ optional
    // --watch file probe) drivers, then poll -> run -> idle, sleeping --interval between idle polls.
    // Bounded by default (--max-ticks, default 1); --max-ticks 0 (or --watch w/o a bound) runs perpetual.
    if std::env::args().nth(1).as_deref() == Some("daemon") {
        return run_daemon().await;
    }
    let mut manifest_path = "keel.lock".to_string();
    let mut tier_override: Option<String> = None;
    let mut think = false;
    let mut core_wire = false;
    let mut sovereign = false;
    let mut critical = false;
    let mut golden_refs: Vec<String> = Vec::new();
    let mut prompt = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--manifest" => manifest_path = args.next().unwrap_or_default(),
            "--tier" => tier_override = args.next(),
            "--kind" => core_wire = matches!(args.next().as_deref(), Some("core-wire") | Some("core_wire")),
            "--sovereign" => sovereign = true,
            "--critical" => critical = true,
            "--golden-ref" => {
                if let Some(n) = args.next() {
                    golden_refs.push(n);
                }
            }
            "--think" => think = true,
            _ => prompt.push(a),
        }
    }
    let prompt = prompt.join(" ");
    if prompt.trim().is_empty() {
        eprintln!("usage: keel [--manifest PATH] [--tier NAME] [--kind core-wire] [--sovereign] [--critical] [--golden-ref NAME] [--think] <prompt>");
        std::process::exit(2);
    }

    let manifest = Manifest::load(&manifest_path)?;
    let mut ctx = new_context(&manifest);
    let user = Message {
        role: Role::User,
        content: vec![Content::Text { text: prompt.clone() }],
        name: None,
        reasoning_content: None,
        tool_call_id: None,
    };

    if let Some(t) = tier_override {
        // ── manual override: pin one tier, skip the router (no needless local cold-start) ──
        let asm = keel::assemble(&manifest, Some(&t))?;
        let req = GenerateRequest {
            messages: vec![user],
            model: asm.model.clone(),
            tools: vec![],
            grammar: None,
            effort: Effort { n: 1, thinking: Some(if think { "high" } else { "low" }.into()) },
            cache_prefix_len: None,
        };
        let res = asm.chain.run(req, &ctx, asm.tier.clone()).await?;
        report(&res, &ctx, &format!("manual override → {}", asm.tier_name), think);
    } else {
        // ── self-driving: the router picks the tier for this turn ──
        let engine = keel::Engine::assemble(&manifest)?;
        let mut step = Step {
            kind: if core_wire { Kind::CoreWire } else { Kind::Scaffolding },
            ty: "user_turn".into(),
            trust_required: Trust::Normal,
            data_class: if sovereign { DataClass::Sovereign } else { DataClass::Normal },
            tier_history: vec![],
            oracle_failures: 0,
            projected_cost: None,
            critical,
            source: Some("cli".into()),
            content: vec![Content::Text { text: prompt.clone() }],
            golden_refs,
        };
        let req = GenerateRequest {
            messages: vec![user],
            model: String::new(), // the engine sets the routed tier's model
            tools: vec![],
            grammar: None,
            // let the router pick thinking per tier; `--think` forces it on regardless of tier.
            effort: Effort { n: 1, thinking: if think { Some("high".into()) } else { None } },
            cache_prefix_len: None,
        };
        let outcome = engine.run(&mut step, &mut ctx, req).await?;
        let mut note = outcome.decision.reason.clone();
        if outcome.substituted {
            note = format!("{note} - chosen tier '{}' unavailable, fell back to '{}'", outcome.decision.tier, outcome.tier_used);
        }
        report(&outcome.result, &ctx, &note, think);
        // I5 surfaced: quiet on a passed (incl. vacuous) verdict; loud on a real oracle failure.
        if outcome.verdict.joint_wrong {
            eprintln!("[keel] !! JOINT_WRONG - {}", outcome.verdict.failures.join("; "));
        } else if !outcome.verdict.passed {
            eprintln!("[keel] !! verify FAILED - {}", outcome.verdict.failures.join("; "));
        }
    }
    Ok(())
}

/// `keel daemon` — the self-driving select-loop (canon §8): the thin **L5 wrapper** the kernel
/// docstring defers here (the bounded loop logic + tests live in `kernel::engine`). Wires a
/// `HeartbeatDriver` (+ an optional `WatchDriver` over `--watch PATH`) and runs the §8 loop:
/// `tick` (select → run → verify → checkpoint → Tape) → on idle, sleep `--interval` and re-poll.
/// **Bounded by default** (`--max-ticks 1`, terminates); `--max-ticks 0` or a `--watch` without an
/// explicit bound runs **perpetual** (until interrupted). Each turn gets a distinct `trace_id`.
async fn run_daemon() -> keel_contracts::Result<()> {
    let mut manifest_path = "keel.lock".to_string();
    let mut max_ticks: usize = 1;
    let mut max_ticks_set = false;
    let mut interval_ms: u64 = 1000;
    let mut watch_path: Option<String> = None;
    let mut prompt = "daemon heartbeat tick: briefly note KEEL is alive.".to_string();
    let mut core_wire = false;
    let mut sovereign = false;
    let mut args = std::env::args().skip(2); // skip "keel" + "daemon"
    while let Some(a) = args.next() {
        match a.as_str() {
            "--manifest" => manifest_path = args.next().unwrap_or_default(),
            "--max-ticks" => {
                max_ticks = args.next().and_then(|n| n.parse().ok()).unwrap_or(1);
                max_ticks_set = true;
            }
            "--interval" => interval_ms = args.next().and_then(|n| n.parse().ok()).unwrap_or(1000),
            "--watch" => watch_path = args.next(),
            "--prompt" => prompt = args.next().unwrap_or(prompt),
            "--kind" => core_wire = matches!(args.next().as_deref(), Some("core-wire") | Some("core_wire")),
            "--sovereign" => sovereign = true,
            _ => {}
        }
    }

    let manifest = Manifest::load(&manifest_path)?;
    let mut ctx = new_context(&manifest);
    let engine = keel::Engine::assemble(&manifest)?;

    // the template Step each self-initiated tick carries; the Driver stamps `Step.source` (canon §7).
    let template = Step {
        kind: if core_wire { Kind::CoreWire } else { Kind::Scaffolding },
        ty: "daemon_tick".into(),
        trust_required: Trust::Normal,
        data_class: if sovereign { DataClass::Sovereign } else { DataClass::Normal },
        tier_history: vec![],
        oracle_failures: 0,
        projected_cost: None,
        critical: false,
        source: None,
        content: vec![Content::Text { text: prompt }],
        golden_refs: vec![],
    };
    let interval = Duration::from_millis(interval_ms);

    // priority order (canon §8): heartbeat, then watch. Both share the one `poll()` joint.
    let mut drivers: Vec<Arc<dyn Driver>> = vec![Arc::new(HeartbeatDriver::new(interval, template.clone()))];
    if let Some(p) = &watch_path {
        let path = std::path::PathBuf::from(p);
        drivers.push(Arc::new(WatchDriver::new(template.clone(), move |_ctx| keel::watch_token(&path))));
    }

    let perpetual = keel::daemon_perpetual(max_ticks, max_ticks_set, watch_path.is_some());
    eprintln!(
        "[keel] daemon: tiers {:?}, interval {interval_ms}ms, {}{}",
        engine.available(),
        if perpetual { "perpetual (until interrupted)".to_string() } else { format!("max-ticks {max_ticks}") },
        watch_path.as_deref().map(|p| format!(", watching {p}")).unwrap_or_default(),
    );

    // the §8 loop with idle = sleep (the L5 form). `tick` returns None when every driver is idle.
    let base = ctx.trace_id.clone();
    let (mut ran, mut attempt) = (0usize, 0usize);
    loop {
        ctx.trace_id = format!("{base}-{attempt}");
        attempt += 1;
        match engine.tick(&drivers, &mut ctx).await {
            Ok(Some(outcome)) => {
                ran += 1;
                report_daemon(ran, &outcome, &ctx);
                if !perpetual && ran >= max_ticks {
                    break;
                }
            }
            Ok(None) => {
                if perpetual || ran < max_ticks {
                    tokio::time::sleep(interval).await; // idle: wait for the next due tick, then re-poll
                } else {
                    break;
                }
            }
            Err(e) => {
                eprintln!("[keel] daemon turn error: {e}");
                break;
            }
        }
    }
    eprintln!("[keel] daemon stopped after {ran} turn(s); total cost ${:.4}", ctx.cost.total);
    Ok(())
}

/// One concise per-turn line for the daemon (operational, on stderr): tier, this turn's + the run's
/// cost, the I5 verdict, and the answer's first line (truncated). The answer is a self-initiated turn.
fn report_daemon(n: usize, o: &keel::Outcome, ctx: &Context) {
    let answer: String = o.result.content.trim().lines().next().unwrap_or("").chars().take(120).collect();
    let verdict = if o.verdict.joint_wrong {
        "JOINT_WRONG"
    } else if o.verdict.passed {
        "pass"
    } else {
        "FAIL"
    };
    eprintln!(
        "[keel] daemon turn {n}: tier={} cost=${:.4} (run ${:.4}) verdict={verdict} | {answer}",
        o.tier_used, o.result.cost, ctx.cost.total,
    );
}

/// Print the answer (+ thinking on `--think`) and the per-call footer: route reason, tier/model/cost.
fn report(res: &GenerateResult, ctx: &Context, route: &str, think: bool) {
    println!("{}", res.content.trim());
    if think {
        if let Some(rc) = res.reasoning_content.as_deref() {
            if !rc.trim().is_empty() {
                eprintln!("\n[thinking] {}", rc.trim());
            }
        }
    }
    eprintln!("\n[keel] route: {route}");
    eprintln!(
        "- tier={} model={} cost=${:.4} tokens={}+{} | trace {} | audit->{}",
        res.tier,
        res.model,
        res.cost,
        res.usage.input_tokens,
        res.usage.output_tokens,
        ctx.trace_id,
        keel::AUDIT_LEDGER
    );
}

/// Print the off-loop metric rollup (canon 19) — ASCII, console-safe on any codepage.
fn print_metrics(m: &keel_store::MetricsSummary) {
    println!("KEEL metrics (reader over {}; off-loop, read-only)", keel::INDEX_DB);
    println!("  turns            {}", m.turns);
    println!("  escalation_rate  {:.3}   (turns above the kind's base tier; flywheel target: down)", m.escalation_rate);
    println!("  rework_rate      {:.3}   (model/content verify-fails, excl. wiring; proxy - precise canon metric deferred)", m.rework_rate);
    println!("  total_cost       ${:.4}", m.total_cost);
    let by_tier = if m.by_tier.is_empty() {
        " (none)".to_string()
    } else {
        m.by_tier.iter().map(|(t, n)| format!(" {t}={n}")).collect::<String>()
    };
    println!("  by_tier         {by_tier}");
}
