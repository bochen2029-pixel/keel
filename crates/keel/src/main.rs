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

use keel_contracts::{Content, Context, DataClass, Effort, GenerateRequest, GenerateResult, Kind, Message, Role, Step, Trust};
use keel_kernel::{new_context, Manifest};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("keel: {e}");
        std::process::exit(1);
    }
}

async fn run() -> keel_contracts::Result<()> {
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
            note = format!("{note} — chosen tier '{}' unavailable, fell back to '{}'", outcome.decision.tier, outcome.tier_used);
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
