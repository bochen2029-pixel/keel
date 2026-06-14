//! # keel-adapters — L2, the tiers (and senses, later)
//!
//! Thin shims under a protocol. Every cognition tier speaks **OpenAI Chat Completions**
//! (canon §3), so `openai` holds the shared request/response mapping once and each provider
//! module adds only its compat specifics. Imports only L0 contracts.
//!
//! - **landed:** `local_llama` (on-box: vision via `image_url`, the thinking toggle, GBNF/JSON
//!   constrained decode, $0) · `deepseek` (cheap-API: HTTPS `/chat/completions`,
//!   `thinking`+`reasoning_effort`, real cost) · `anthropic` (frontier: the Messages API — its own
//!   protocol, the canonical thin gateway) · `whisper` (the ears — a local, sovereign transcription
//!   organ, NOT a tier; canon §12).
//! - **next:** the `mcp` tool host · the embedder/reranker (Memory organs) · the privacy classifier.

pub mod anthropic;
pub mod deepseek;
pub mod local_llama;
pub mod openai;
pub mod whisper;

pub use anthropic::Anthropic;
pub use deepseek::DeepSeek;
pub use local_llama::LocalLlama;
pub use whisper::Whisper;
