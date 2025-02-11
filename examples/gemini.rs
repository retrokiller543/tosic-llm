use std::io::Write;
use thiserror::Error;
use tokio_stream::StreamExt;
use tosic_llm::gemini::{GeminiClient, GeminiContent, GeminiModel};
use tosic_llm::{ensure, LlmProvider};
use tosic_llm::error::{LlmError, WithContext};
use tosic_llm::types::Role;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Llm(#[from] LlmError),
    #[error("{0}")]
    Generic(String),
}

use serde_json::{Result as JsonResult, Value};

#[derive(Debug)]
pub struct GeminiResponseParser {
    has_started: bool,
    has_finished: bool,
}

impl GeminiResponseParser {
    pub fn new() -> Self {
        Self {
            has_started: false,
            has_finished: false,
        }
    }

    pub fn parse_chunk(&mut self, chunk: &[u8]) -> JsonResult<Option<String>> {
        // Convert bytes to string
        let chunk_str = String::from_utf8_lossy(chunk);

        // Handle the start and end markers
        if chunk_str == "[" {
            self.has_started = true;
            return Ok(None);
        } else if chunk_str == "]" || chunk_str.is_empty() {
            self.has_finished = true;
            return Ok(None);
        }

        // Remove leading comma if present (subsequent chunks start with ,\r\n)
        let cleaned_chunk = if chunk_str.starts_with(",") {
            chunk_str.trim_start_matches(",").trim_start()
        } else {
            &chunk_str
        };

        // Parse the JSON object
        let v: Value = serde_json::from_str(cleaned_chunk)?;

        // Extract the text from the nested structure
        let text = v
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .map(String::from);

        Ok(text)
    }
}

async fn ask_ai() -> Result<(), Error> {
    let client = GeminiClient::new(GeminiModel::Gemini2Flash).context("Failed to create the Gemini Client")?;
    let provider = LlmProvider::new(client);

    let req = GeminiContent::new(Some(Role::User), "Hi my name is Emil and i like to write complex rust libraries".to_string());

    let res = provider.generate(vec![req], true).await.context("Failed to get response from LLM")?;

    ensure!(res.is_stream(), Error::Generic("Response is not stream".into()));

    let mut stream = res.unwrap_stream();

    // Stream to STDOUT
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let mut parser = GeminiResponseParser::new();
    let mut written_len = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read response from LLM")?;

        if let Ok(Some(text)) = parser.parse_chunk(&chunk) {
            let bytes = text.as_bytes();
            written_len += bytes.len();
            stdout.write_all(bytes).context("Failed to write to stdout")?;
            stdout.flush().context("Failed to flush stdout")?;
        }
    }


    println!("\nWrote {} bytes", written_len);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    ask_ai().await
}