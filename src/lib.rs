// tosic_llm/src/lib.rs

pub use gemini::*;

pub mod error;
pub mod gemini;
mod utils;
pub mod traits;
pub mod types;

type Result<T, E = error::LlmError> = core::result::Result<T, E>;

