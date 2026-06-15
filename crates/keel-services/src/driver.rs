//! keel-services::driver — initiative: the sources of work (canon §7 joint #8; §8 loop step 1).
//!
//! A `Driver` is what makes KEEL **a self that acts**, not merely one that responds. The engine's
//! loop opens with `select(drivers).poll()` (canon §8), so a turn can originate from the operator OR
//! from KEEL itself. Three of the operator's four real systems act without a user turn (DAVE's
//! outreach, TARS's PULSE, The Box's SILENCE) — which is why initiative is a day-one genome member.
//!
//! This module ships the three default drivers, all behind the one frozen `poll() -> Option<Step>`
//! joint (`None` = nothing to do now). That is exactly the §23 falsifier for this seam: *one* Driver
//! abstraction must express idle/user work **and** a perpetual heartbeat **and** watch-on-change. If
//! it couldn't, the initiative abstraction would be at the wrong altitude.
//!
//! - [`UserTurnDriver`] — a FIFO queue of operator-submitted Steps (the user turn).
//! - [`HeartbeatDriver`] — a perpetual tick: emit every `interval` (consolidation, idle outreach).
//! - [`WatchDriver`] — poll-based watch-on-change: emit when a probed state token changes.
//!
//! Each driver stamps `Step.source` with its id (canon §7 — "which Driver emitted it"), and is pure,
//! model-free, and unit-testable. **Scope (this slice):** the drivers themselves. The daemon
//! select-loop — poll the drivers in priority order and run each emitted `Step` through
//! `kernel::engine` — is a deliberate follow-on (it owns the run-execution, not the initiative seam).

use async_trait::async_trait;
use keel_contracts::{Context, Driver, Result, Step};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// `Step.source` ids — the *initiative* topology (which Driver emitted the Step, canon §7), the
/// efferent twin of perception's capture topology ("mic"/"loopback"/"screen").
pub const SRC_USER: &str = "user";
pub const SRC_HEARTBEAT: &str = "heartbeat";
pub const SRC_WATCH: &str = "watch";

// ── user-turn driver ───────────────────────────────────────────────────────────

/// The user-turn driver: a FIFO queue of operator-submitted Steps. `poll` dequeues one and stamps
/// `source = "user"`; an empty queue is `None` (nothing to do now). Deliberately thin — Step
/// construction (kind, `critical`, `golden_refs`, the CLI flags) stays at the caller (L5, where that
/// context lives). This is just the seam that turns "the operator submitted a turn" into the loop's
/// first move, *uniform* with the self-initiated drivers below — so the engine never special-cases who
/// started a turn.
#[derive(Default)]
pub struct UserTurnDriver {
    pending: Mutex<VecDeque<Step>>,
}

impl UserTurnDriver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-seed the queue (e.g. a batch of turns).
    pub fn with_steps(steps: impl IntoIterator<Item = Step>) -> Self {
        Self { pending: Mutex::new(steps.into_iter().collect()) }
    }

    /// Enqueue a user turn (L5 calls this when the operator submits input).
    pub fn push(&self, step: Step) {
        self.pending.lock().expect("user-turn queue poisoned").push_back(step);
    }

    pub fn len(&self) -> usize {
        self.pending.lock().expect("user-turn queue poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl Driver for UserTurnDriver {
    async fn poll(&self, _ctx: &Context) -> Result<Option<Step>> {
        // lock scope holds no await: the guard is dropped before we return (no await-holding-lock).
        let next = self.pending.lock().expect("user-turn queue poisoned").pop_front();
        Ok(next.map(|mut s| {
            s.source = Some(SRC_USER.to_string());
            s
        }))
    }
}

// ── heartbeat driver ─────────────────────────────────────────────────────────

/// A perpetual heartbeat: emits its template Step once per elapsed `interval` — the §8 self-initiated
/// tick (consolidation-as-a-Step, idle outreach). The pure cadence logic is [`HeartbeatDriver::tick`]
/// (clock injected) so it is deterministically testable; `poll` is exactly `self.tick(Instant::now())`.
/// Reading a clock is legitimate at L4 — the L0 *contracts* stay clock-free (canon §7; the kernel
/// stamps `Time`), but a heartbeat service necessarily observes elapsed wall-time. The first poll
/// fires (it starts the cadence).
pub struct HeartbeatDriver {
    interval: Duration,
    step: Step,
    last: Mutex<Option<Instant>>,
}

impl HeartbeatDriver {
    pub fn new(interval: Duration, step: Step) -> Self {
        Self { interval, step, last: Mutex::new(None) }
    }

    /// The pure cadence decision at an injected `now`: `Some(step)` if an `interval` has elapsed since
    /// the last fire (or this is the first poll), else `None`. Advances the last-fire instant only when
    /// it fires, so the cadence never drifts faster than `interval`.
    pub fn tick(&self, now: Instant) -> Option<Step> {
        let mut last = self.last.lock().expect("heartbeat clock poisoned");
        let due = match *last {
            None => true,
            Some(prev) => now.duration_since(prev) >= self.interval,
        };
        if !due {
            return None;
        }
        *last = Some(now);
        let mut s = self.step.clone();
        s.source = Some(SRC_HEARTBEAT.to_string());
        Some(s)
    }
}

#[async_trait]
impl Driver for HeartbeatDriver {
    async fn poll(&self, _ctx: &Context) -> Result<Option<Step>> {
        Ok(self.tick(Instant::now()))
    }
}

// ── watch driver ─────────────────────────────────────────────────────────────

/// A watch probe: the current state of the watched thing as a comparable token, or `None` when there
/// is nothing to observe (e.g. the watched path is absent). The caller owns *what* is watched (a file
/// mtime, a queue depth, a content hash), so the genome takes no dependency on any particular source —
/// the watched edge stays at the edge.
type Probe = Box<dyn Fn(&Context) -> Option<u64> + Send + Sync>;

/// Poll-based watch-on-change: emits the template Step when the probed token **differs** from the last
/// observed one, else `None`. The first observation (`None -> Some`) counts as a change and fires
/// (unknown → known); a probe returning `None` never fires. Model-free — the change comparison is a
/// plain token equality, never a model judgement (the same discipline as perception's dHash gate).
pub struct WatchDriver {
    probe: Probe,
    step: Step,
    last: Mutex<Option<u64>>,
}

impl WatchDriver {
    pub fn new(step: Step, probe: impl Fn(&Context) -> Option<u64> + Send + Sync + 'static) -> Self {
        Self { probe: Box::new(probe), step, last: Mutex::new(None) }
    }
}

#[async_trait]
impl Driver for WatchDriver {
    async fn poll(&self, ctx: &Context) -> Result<Option<Step>> {
        let Some(token) = (self.probe)(ctx) else {
            return Ok(None); // nothing to observe
        };
        let mut last = self.last.lock().expect("watch state poisoned");
        if *last == Some(token) {
            return Ok(None); // unchanged → free
        }
        *last = Some(token);
        let mut s = self.step.clone();
        s.source = Some(SRC_WATCH.to_string());
        Ok(Some(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keel_contracts::{DataClass, Kind, Trust};
    use std::sync::Arc;

    fn step() -> Step {
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
            content: vec![],
            golden_refs: vec![],
        }
    }
    fn ctx() -> Context {
        Context { trace_id: "t1".into(), ..Default::default() }
    }

    #[tokio::test]
    async fn user_turn_dequeues_fifo_then_none() {
        let d = UserTurnDriver::new();
        assert!(d.is_empty());
        let mut a = step();
        a.ty = "a".into();
        let mut b = step();
        b.ty = "b".into();
        d.push(a);
        d.push(b);
        assert_eq!(d.len(), 2);

        let first = d.poll(&ctx()).await.unwrap().expect("a queued step");
        assert_eq!(first.ty, "a"); // FIFO order
        assert_eq!(first.source.as_deref(), Some(SRC_USER)); // the driver stamps source (canon §7)
        assert_eq!(d.poll(&ctx()).await.unwrap().unwrap().ty, "b");
        assert!(d.poll(&ctx()).await.unwrap().is_none(), "drained → nothing to do now");
    }

    #[test]
    fn heartbeat_fires_once_per_interval() {
        let base = Instant::now();
        let d = HeartbeatDriver::new(Duration::from_secs(60), step());
        assert!(d.tick(base).is_some(), "first tick starts the cadence");
        assert!(d.tick(base).is_none(), "same instant → not due again");
        assert!(d.tick(base + Duration::from_secs(30)).is_none(), "30s < 60s → not due");
        let fired = d.tick(base + Duration::from_secs(60)).expect("due at 60s");
        assert_eq!(fired.source.as_deref(), Some(SRC_HEARTBEAT));
        assert!(d.tick(base + Duration::from_secs(61)).is_none(), "1s after firing → not due");
        assert!(d.tick(base + Duration::from_secs(120)).is_some(), "another interval elapsed → due");
    }

    #[tokio::test]
    async fn heartbeat_poll_uses_the_real_clock() {
        // a zero interval is always due → the async joint emits, stamping the heartbeat source.
        let d = HeartbeatDriver::new(Duration::from_millis(0), step());
        let s = d.poll(&ctx()).await.unwrap().expect("zero interval is always due");
        assert_eq!(s.source.as_deref(), Some(SRC_HEARTBEAT));
    }

    #[tokio::test]
    async fn watch_emits_only_on_change() {
        // a controlled probe sequence: 1, 1, 2, then nothing to observe.
        let seq = Arc::new(Mutex::new(VecDeque::from(vec![Some(1u64), Some(1), Some(2), None])));
        let seq2 = seq.clone();
        let d = WatchDriver::new(step(), move |_ctx| seq2.lock().unwrap().pop_front().flatten());

        let first = d.poll(&ctx()).await.unwrap().expect("first observation fires (unknown → known)");
        assert_eq!(first.source.as_deref(), Some(SRC_WATCH));
        assert!(d.poll(&ctx()).await.unwrap().is_none(), "unchanged (1 == 1) → no emit");
        assert!(d.poll(&ctx()).await.unwrap().is_some(), "changed (1 → 2) → emit");
        assert!(d.poll(&ctx()).await.unwrap().is_none(), "probe None → nothing to observe");
    }

    /// The §23 falsifier for the initiative seam: a single `Driver` joint expresses user work AND a
    /// perpetual heartbeat AND watch-on-change — all behind one `&dyn Driver`, each firing on first
    /// poll. If this couldn't hold, the abstraction would be wrong (canon §23).
    #[tokio::test]
    async fn one_poll_seam_expresses_all_three_modes() {
        let user = UserTurnDriver::new();
        user.push(step());
        let drivers: Vec<Box<dyn Driver>> = vec![
            Box::new(user),
            Box::new(HeartbeatDriver::new(Duration::from_secs(1), step())),
            Box::new(WatchDriver::new(step(), |_ctx| Some(7))),
        ];
        let mut sources: Vec<String> = Vec::new();
        for d in &drivers {
            let s = d.poll(&ctx()).await.unwrap().expect("each driver yields a Step on first poll");
            sources.push(s.source.unwrap());
        }
        let got: Vec<&str> = sources.iter().map(String::as_str).collect();
        assert_eq!(got, [SRC_USER, SRC_HEARTBEAT, SRC_WATCH]);
    }
}
