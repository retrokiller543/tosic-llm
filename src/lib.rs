// tosic_llm/src/lib.rs

pub use provider::*;

pub mod error;
pub mod gemini;
pub mod provider;
pub mod traits;
pub mod types;
mod utils;

pub type Result<T, E = error::LlmError> = core::result::Result<T, E>;
