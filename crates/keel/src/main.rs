//! keel — the daily-driver CLI (L5). Assembles the spine from `keel.lock` (via `keel::assemble`),
//! runs one prompt through the invariant chain to the resolved tier, prints the answer, and logs
//! every call to the file ledger.
//!
//!   keel "read me the config"              keel --tier cheap-API "weigh the tradeoffs"
//!   keel --think "…"                       keel --manifest C:\KEEL\keel.lock "hello"

use keel_contracts::{Content, Effort, GenerateRequest, Message, Role};
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
    let mut prompt = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--manifest" => manifest_path = args.next().unwrap_or_default(),
            "--tier" => tier_override = args.next(),
            "--think" => think = true,
            _ => prompt.push(a),
        }
    }
    let prompt = prompt.join(" ");
    if prompt.trim().is_empty() {
        eprintln!("usage: keel [--manifest PATH] [--tier NAME] [--think] <prompt>");
        std::process::exit(2);
    }

    let manifest = Manifest::load(&manifest_path)?;
    let asm = keel::assemble(&manifest, tier_override.as_deref())?;
    let ctx = new_context(&manifest);
    let req = GenerateRequest {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text { text: prompt }],
            name: None,
            reasoning_content: None,
            tool_call_id: None,
        }],
        model: asm.model.clone(),
        tools: vec![],
        grammar: None,
        effort: Effort { n: 1, thinking: Some((if think { "high" } else { "low" }).into()) },
        cache_prefix_len: None,
    };
    let res = asm.chain.run(req, &ctx, asm.tier.clone()).await?;

    println!("{}", res.content.trim());
    if think {
        if let Some(rc) = res.reasoning_content.as_deref() {
            if !rc.trim().is_empty() {
                eprintln!("\n[thinking] {}", rc.trim());
            }
        }
    }
    eprintln!(
        "\n— tier={} model={} cost=${:.4} tokens={}+{} · trace {} · audit→{}",
        res.tier,
        res.model,
        res.cost,
        res.usage.input_tokens,
        res.usage.output_tokens,
        ctx.trace_id,
        keel::AUDIT_LEDGER
    );
    Ok(())
}
