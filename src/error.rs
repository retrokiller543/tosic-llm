// tosic_llm/src/error.rs

use thiserror::Error;
use url::ParseError;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("An error occurred: {0}")]
    Generic(#[from] Box<dyn std::error::Error + Send>),
}
