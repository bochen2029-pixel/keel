//! KEEL error taxonomy (canon §18). Typed, with stable codes that match the canon table.

use std::fmt;

/// Every fallible contract method returns `Result<T>` over this taxonomy.
pub type Result<T> = std::result::Result<T, KeelError>;

#[derive(Clone, Debug)]
pub enum KeelError {
    /// No tier clears the bar within budget.
    RouteNoTier(String),
    /// Provider/server down or OOM.
    TierUnavailable(String),
    /// Output failed an oracle.
    OracleFail(String),
    /// Tests + code agree, a golden disagrees. Surfaced to the operator immediately.
    JointWrong(String),
    /// Task cost cap hit (I4 hard-stop).
    BudgetExceeded(String),
    /// All tiers failed the oracle.
    EscalationExhausted(String),
    /// Undo cost unstatable in one sentence (the reversibility gate).
    ReversibilityBlock(String),
    /// A claim cannot be grounded in ground truth (routes to human).
    InsufficientSource(String),
    /// A sense returned below its confidence floor (routes to review).
    PerceptLowConfidence(String),
    /// No inference layer found or launchable.
    SubstrateUnresolved(String),
    /// Constrained decode produced invalid output (should be impossible; adapter bug on repeat).
    GrammarViolation(String),
    /// Anything else, carried with a message.
    Other(String),
}

impl KeelError {
    /// The stable code string (canon §18).
    pub fn code(&self) -> &'static str {
        use KeelError::*;
        match self {
            RouteNoTier(_) => "ROUTE_NO_TIER",
            TierUnavailable(_) => "TIER_UNAVAILABLE",
            OracleFail(_) => "ORACLE_FAIL",
            JointWrong(_) => "JOINT_WRONG",
            BudgetExceeded(_) => "BUDGET_EXCEEDED",
            EscalationExhausted(_) => "ESCALATION_EXHAUSTED",
            ReversibilityBlock(_) => "REVERSIBILITY_BLOCK",
            InsufficientSource(_) => "INSUFFICIENT_SOURCE",
            PerceptLowConfidence(_) => "PERCEPT_LOW_CONFIDENCE",
            SubstrateUnresolved(_) => "SUBSTRATE_UNRESOLVED",
            GrammarViolation(_) => "GRAMMAR_VIOLATION",
            Other(_) => "KEEL_ERROR",
        }
    }

    /// The carried human-readable message.
    pub fn message(&self) -> &str {
        use KeelError::*;
        match self {
            RouteNoTier(s) | TierUnavailable(s) | OracleFail(s) | JointWrong(s) | BudgetExceeded(s)
            | EscalationExhausted(s) | ReversibilityBlock(s) | InsufficientSource(s)
            | PerceptLowConfidence(s) | SubstrateUnresolved(s) | GrammarViolation(s) | Other(s) => s,
        }
    }
}

impl fmt::Display for KeelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code(), self.message())
    }
}

impl std::error::Error for KeelError {}
