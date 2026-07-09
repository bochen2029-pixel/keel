//! keel-services::maintenance — the A7.4 memory-autopilot policy (canon §11 / the memory-autopilot
//! proposal). Decides WHEN the existing maintenance operations fire — consolidation (A6.2/A7.2) and
//! cold-eyes (I5) — so a KEEL consumer never manages memory by hand: no flags, no manual commands.
//! **Pure decision logic** — model-free, clock-free, unit-tested; the L5 wiring runs the turns and
//! owns the trigger inputs (turn counts, session end, budget pressure).
//!
//! Triggers (REEL §6.2, the model-free subset): every-N-turns · Ring-2 budget pressure · session
//! end. The significance-threshold trigger is model-dependent and deliberately NOT built (the
//! proposal's exclusion list). Cold-eyes runs on a consolidations-cadence — drift is checked
//! *periodically by default*, not on operator demand.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// What the policy asks the L5 wiring to run next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maintenance {
    /// Compress Ring-2 → the Ring-3 narrative + one appended episode (A7.2).
    Consolidate,
    /// Validate the narrative against the Tape (I5; the A7.5 correction loop rides the verdict).
    ColdEyes,
}

/// The trigger thresholds (keel.lock `memory.*` — config, not pins; 0 disables that trigger).
#[derive(Clone, Copy, Debug)]
pub struct MaintenancePolicy {
    /// Consolidate after this many new Tape turns since the last consolidation.
    pub consolidate_every_turns: usize,
    /// Session-end / budget-pressure consolidations need at least this many new turns (min 1 —
    /// there is never anything to consolidate from zero new turns).
    pub session_end_min_turns: usize,
    /// Run cold-eyes after this many consolidations since the last validation.
    pub cold_eyes_every: usize,
}

/// A snapshot of the observable memory state the policy decides over (the L5 wiring fills it).
#[derive(Clone, Copy, Debug, Default)]
pub struct MaintenanceStats {
    /// Total turns on the Tape now.
    pub turns_total: usize,
    /// Total turns on the Tape when the last consolidation ran (from [`MaintState`]).
    pub last_consolidated_turns: usize,
    /// Consolidations since the last cold-eyes pass (from [`MaintState`]).
    pub consolidations_since_cold_eyes: usize,
    /// Ring-2's working window currently exceeds its char budget (context pressure).
    pub ring2_over_budget: bool,
    /// This check runs at a session boundary (a one-shot CLI turn ending; a daemon shutdown).
    pub session_end: bool,
    /// A narrative exists to validate (cold-eyes is meaningless before the first consolidation).
    pub has_narrative: bool,
}

impl MaintenancePolicy {
    /// The next due maintenance, or `None`. Consolidation outranks cold-eyes (validate the NEW
    /// narrative, not the stale one). Every trigger needs at least one new turn — zero new turns
    /// never consolidates (nothing to compress) and never re-validates on an unchanged register.
    pub fn due(&self, s: &MaintenanceStats) -> Option<Maintenance> {
        let new_turns = s.turns_total.saturating_sub(s.last_consolidated_turns);
        let min = self.session_end_min_turns.max(1);
        let consolidate = (self.consolidate_every_turns > 0 && new_turns >= self.consolidate_every_turns)
            || (s.ring2_over_budget && new_turns >= min)
            || (s.session_end && new_turns >= min);
        if consolidate {
            return Some(Maintenance::Consolidate);
        }
        if self.cold_eyes_every > 0 && s.has_narrative && s.consolidations_since_cold_eyes >= self.cold_eyes_every {
            return Some(Maintenance::ColdEyes);
        }
        None
    }
}

/// The autopilot's durable cursor (a small JSON sidecar of the Tape, `<tape_stem>.maint.json`):
/// where the last consolidation left off + the cold-eyes cadence counter + the A7.5 drift flag.
/// Derived state — deleting it just resets the cadence (the registers themselves are untouched).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MaintState {
    #[serde(default)]
    pub last_consolidated_turns: usize,
    #[serde(default)]
    pub consolidations_since_cold_eyes: usize,
    /// A7.5: a drift correction ran and the next cold-eyes has not yet confirmed CONSISTENT.
    /// Drift while this is set is PERSISTENT drift — surfaced, never re-corrected in a loop.
    #[serde(default)]
    pub pending_drift: bool,
}

impl MaintState {
    /// Load from the sidecar; a missing/garbled file is a fresh default (never fatal).
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    /// Persist (best-effort dir creation; an error is the caller's to report — state loss only
    /// resets the cadence, never the registers).
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir)?;
            }
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> MaintenancePolicy {
        MaintenancePolicy { consolidate_every_turns: 24, session_end_min_turns: 4, cold_eyes_every: 4 }
    }

    #[test]
    fn consolidates_after_every_n_turns() {
        let p = policy();
        let mut s = MaintenanceStats { turns_total: 30, last_consolidated_turns: 10, ..Default::default() };
        assert_eq!(p.due(&s), None, "19 new turns < 24 and no other trigger");
        s.turns_total = 34;
        assert_eq!(p.due(&s), Some(Maintenance::Consolidate), "24 new turns -> consolidate");
    }

    #[test]
    fn session_end_and_budget_pressure_need_the_minimum_new_turns() {
        let p = policy();
        let mut s = MaintenanceStats { turns_total: 12, last_consolidated_turns: 10, session_end: true, ..Default::default() };
        assert_eq!(p.due(&s), None, "2 new turns < the session-end minimum of 4");
        s.turns_total = 14;
        assert_eq!(p.due(&s), Some(Maintenance::Consolidate), "session end + >=4 new turns");
        // budget pressure fires the same way without a session end.
        let s2 = MaintenanceStats { turns_total: 15, last_consolidated_turns: 10, ring2_over_budget: true, ..Default::default() };
        assert_eq!(p.due(&s2), Some(Maintenance::Consolidate));
        // zero new turns NEVER consolidates, whatever the flags say.
        let s3 = MaintenanceStats { turns_total: 10, last_consolidated_turns: 10, session_end: true, ring2_over_budget: true, ..Default::default() };
        assert_eq!(p.due(&s3), None);
    }

    #[test]
    fn cold_eyes_runs_on_the_consolidation_cadence_only_with_a_narrative() {
        let p = policy();
        let mut s = MaintenanceStats {
            turns_total: 11,
            last_consolidated_turns: 10,
            consolidations_since_cold_eyes: 4,
            has_narrative: true,
            ..Default::default()
        };
        assert_eq!(p.due(&s), Some(Maintenance::ColdEyes), "cadence reached + a narrative exists");
        s.has_narrative = false;
        assert_eq!(p.due(&s), None, "nothing to validate before the first consolidation");
        s.has_narrative = true;
        s.consolidations_since_cold_eyes = 3;
        assert_eq!(p.due(&s), None, "cadence not reached");
        // a due consolidation outranks a due cold-eyes (validate the NEW narrative).
        s.consolidations_since_cold_eyes = 9;
        s.turns_total = 40;
        assert_eq!(p.due(&s), Some(Maintenance::Consolidate));
    }

    #[test]
    fn disabled_triggers_and_state_roundtrip() {
        let off = MaintenancePolicy { consolidate_every_turns: 0, session_end_min_turns: 4, cold_eyes_every: 0 };
        let s = MaintenanceStats { turns_total: 1000, consolidations_since_cold_eyes: 99, has_narrative: true, ..Default::default() };
        assert_eq!(off.due(&s), None, "0 disables a trigger");

        let path = std::env::temp_dir().join(format!("keel-maint-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert_eq!(MaintState::load(&path).last_consolidated_turns, 0, "missing file -> fresh default");
        let st = MaintState { last_consolidated_turns: 42, consolidations_since_cold_eyes: 2, pending_drift: true };
        st.save(&path).unwrap();
        let back = MaintState::load(&path);
        assert_eq!(back.last_consolidated_turns, 42);
        assert!(back.pending_drift);
        let _ = std::fs::remove_file(&path);
    }
}
