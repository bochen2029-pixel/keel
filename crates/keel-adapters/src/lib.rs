//! # keel-adapters — L2, the tiers (and senses, later)
//!
//! Thin shims under a protocol. Every cognition tier speaks **OpenAI Chat Completions**
//! (canon §3), so `openai` holds the shared request/response mapping once and each provider
//! module adds only its compat specifics. Imports only L0 contracts.
//!
//! - **landed:** `local_llama` (HTTP → llama-server: vision via `image_url`, the per-step thinking
//!   toggle, GBNF/JSON-schema constrained decode, $0 local cost).
//! - **next:** `deepseek` (cheap-API) · `anthropic` (frontier) · `whisper` (ears).

pub mod local_llama;
pub mod openai;

pub use local_llama::LocalLlama;
