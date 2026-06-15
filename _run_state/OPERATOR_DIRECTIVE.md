# OPERATOR DIRECTIVE — standing, overrides default cadence (READ ON EVERY RECONSTITUTION)

> Given by Bo **2026-06-15**, then he **went to bed — the autonomous run continues overnight.** This is
> the operating mode UNTIL the bulk of what can be done is done and all the low-hanging fruit is gone
> (per ROADMAP). **It survives compaction:** a post-compaction instance reads this and keeps going in
> this mode without asking. Do NOT stop, do NOT ask, do NOT wait for him.

## Verbatim (the core)
"there is too much meta work going on here, the overhead is more than the actual work, just make real
progress on a bunch of stuff, hit hard, find work to do, go all out... stop wasting time on this meta
work that is taking you down a rabbit hole; set kill timers for everything, the moment you find you are
getting stuck, pivot to something else, in a round robin way until everything gets first pass if it
comes to that, so your efforts are not wasted and so that bulk of it actually goes into first order
'real work' as opposed to scaffolding on scaffolding... that is the directive from now until you have
done the bulk of what can be done and all the low hanging fruit is gone etc, per roadmap. after you
have done all the low hanging fruit you can without using the model then proceed with it. USE YOUR
COMMON SENSE... do NOT prompt me asking for question... if i am not at console this means you get stuck
overnight, never trigger this ever again."

## The rules (follow these — they are the loop, with ROADMAP §0)
1. **REAL WORK over meta.** Bias hard to building/shipping real features. Minimize deliberation, status
   essays, doc churn (terse WORKLOG/STATE lines only). No scaffolding-on-scaffolding. If the overhead
   exceeds the actual work, you're doing it wrong — cut it and go build.
2. **NEVER ask the operator a question** (no `AskUserQuestion`, EVER) until there is genuinely no work
   left at all. He may be asleep → asking = stuck overnight. **Decide with common sense + document
   tersely**; he overrides later if wrong. (He explicitly authorizes deciding on his behalf.)
3. **TTL everything.** Bound every shell / live-model run with an explicit kill timer. Proven pattern:
   `$p = Start-Process keel.exe -ArgumentList '…' -RedirectStandardOutput out.txt -PassThru -NoNewWindow;
   if ($p.WaitForExit(ms)) {…} else { $p.Kill($true) }` — **file redirect, NOT `| Out-String`** (the pipe
   hangs on the detached llama-server's inherited handle; ISSUE-8). Verify by **artifact**, never a
   hanging capture. Never start anything unbounded.
4. **Stuck ~twice → PIVOT.** Diagnose once; if it doesn't yield, record what you have, file a ROADMAP §5
   ISSUE, and move to the next actionable item. **Round-robin the ROADMAP; first-pass everything before
   perfecting anything.** One blocker NEVER stalls the run.
5. **Model-free fruit first, THEN use the model.** (As of 2026-06-15 the model-free fruit is DONE — so
   **proceed WITH the model**, kill-timered: C1/C2 recall-uplift benchmarks · cold-eyes validation · B3
   daemon trend data · the D1 cell. The substrate: pre-start or let keel cold-start llama-server; reuse
   via probe to avoid the spawn-hang.)
6. **Use common sense.** Don't seek hand-holding on obvious calls. Decide and move.

**Inviolable still holds** (even fully autonomous): never edit a frozen contract / golden / re-stamp the
seal / mutate the toolchain (→ ISSUES, route around); never commit a secret; reversibility gate. Keep
going until **DONE / STALLED / HALT / tokens / power**.
