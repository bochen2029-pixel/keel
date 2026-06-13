//! # keel-contracts — L0, the frozen joints
//!
//! The genome's load-bearing surface: the core types, the §18 error taxonomy, and the
//! **ten contracts** every layer above is built against. Get the joints right and the
//! bones can be swapped.
//!
//! These are **frozen** (canon §7). The accompanying golden cases (`tests/golden/golden.json`)
//! are the *language-neutral conformance layer*: any KEEL implementation, in any language,
//! must pass them — which is what makes the implementation language the most reversible
//! decision in the system (canon §20 #5).
//!
//! Layer rule (canon §6): contracts depend on nothing. Everything depends on contracts.

pub mod errors;
pub mod traits;
pub mod types;

pub use errors::{KeelError, Result};
pub use traits::*;
pub use types::*;
