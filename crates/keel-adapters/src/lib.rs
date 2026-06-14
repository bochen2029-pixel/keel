//! # keel-adapters — L2, the tiers (and senses, later)
//!
//! Thin shims under a protocol. Every cognition tier speaks **OpenAI Chat Completions**
//! (canon §3), so `openai` holds the shared request/response mapping once and each provider
//! module adds only its compat specifics. Imports only L0 contracts.
//!
//! - **landed:** `local_llama` (on-box: vision via `image_url`, the thinking toggle, GBNF/JSON
//!   constrained decode, $0) · `deepseek` (cheap-API: HTTPS `/chat/completions`,
//!   `thinking`+`reasoning_effort`, real cost + the prompt-cache lever).
//! - **next:** `anthropic` (frontier) · `whisper` (ears).

pub mod deepseek;
pub mod local_llama;
pub mod openai;

pub use deepseek::DeepSeek;
pub use local_llama::LocalLlama;
