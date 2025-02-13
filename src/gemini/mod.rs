// tosic_llm/src/gemini/mod.rs

mod impls;
mod types;

use crate::error::LlmError;
use crate::traits::LlmClient;
use crate::utils::SingleOrMultiple;
use bytes::Bytes;
use derive_more::{AsMut, AsRef, Display, From};
use futures_util::{Stream, TryStreamExt};
use reqwest::{Client, Response};
use serde::Serialize;
use serde_json::Value;
use std::sync::LazyLock;
use tosic_utils::env::env_util;
pub use types::*;
use url::Url;

pub const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
pub const GEMINI_STREAM_ENDPOINT: &str = ":streamGenerateContent";
pub const GEMINI_ENDPOINT: &str = ":generateContent";

/// Lazily fetched env variable of the API key to Gemini.
///
/// Variable: `GEMINI_API_KEY`.
///
/// # Panics
///
/// Will panic if the environment variable is not set but attempted to initialize.
pub static GEMINI_KEY: LazyLock<String> = LazyLock::new(|| env_util!("GEMINI_API_KEY"));

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Display)]
pub enum GeminiModel {
    #[display("models/gemini-2.0-flash")]
    Gemini2Flash,
    #[display("models/gemini-2.0-flash-lite-preview-02-05")]
    Gemini2FlashLite,
}

#[derive(Debug, Clone, AsRef, AsMut, From)]
pub struct GeminiClient {
    model: GeminiModel,
    client: Client,
}

impl GeminiClient {
    pub fn new(model: GeminiModel) -> crate::Result<Self> {
        let client = Client::builder().build()?;

        Ok(Self { model, client })
    }

    #[tracing::instrument(skip(endpoint, extra_query))]
    fn endpoint_url(
        &self,
        endpoint: impl AsRef<str>,
        extra_query: Option<&str>,
    ) -> crate::Result<Url> {
        let query = if let Some(query) = extra_query {
            format!("?{}&key={}", query, *GEMINI_KEY)
        } else {
            format!("?key={}", *GEMINI_KEY)
        };

        Url::parse(&format!(
            "{GEMINI_BASE_URL}/{}{}{query}",
            self.model,
            endpoint.as_ref()
        ))
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(request, endpoint))]
    async fn send_request(
        &self,
        request: impl Serialize,
        endpoint: (impl AsRef<str>, Option<&str>),
    ) -> crate::Result<Response> {
        let url = self.endpoint_url(endpoint.0, endpoint.1)?;

        self.client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(Into::into)
    }

    async fn stream_generate_content_inner<T: Into<GeminiContent>>(
        &self,
        input: impl Into<SingleOrMultiple<T>>,
    ) -> crate::Result<impl Stream<Item = crate::Result<Bytes>>> {
        let request = GeminiRequest {
            contents: input.into().into_iter().map(Into::into).collect(),
        };

        let response = self
            .send_request(request, (GEMINI_STREAM_ENDPOINT, None))
            .await?;

        let stream = response.bytes_stream().map_err(Into::into);

        Ok(stream)
    }

    async fn generate_content_inner<T: Into<GeminiContent>>(
        &self,
        input: impl Into<SingleOrMultiple<T>>,
    ) -> crate::Result<Value> {
        let request = GeminiRequest {
            contents: input.into().into_iter().map(Into::into).collect(),
        };

        let response = self.send_request(request, (GEMINI_ENDPOINT, None)).await?;

        let response: Value = response.json().await?;

        Ok(response)
    }

    #[tracing::instrument(skip(input))]
    pub async fn stream_generate_content(
        &self,
        input: impl Into<GeminiContent>,
    ) -> crate::Result<impl Stream<Item = crate::Result<Bytes>>> {
        self.stream_generate_content_inner(input).await
    }

    #[tracing::instrument(skip(input))]
    pub async fn generate_content(&self, input: impl Into<GeminiContent>) -> crate::Result<Value> {
        self.generate_content_inner(input).await
    }

    #[tracing::instrument(skip(input))]
    pub async fn stream_generate_content_iter(
        &self,
        input: impl IntoIterator<Item = impl Into<GeminiContent>>,
    ) -> crate::Result<impl Stream<Item = crate::Result<Bytes>>> {
        self.stream_generate_content_inner::<GeminiContent>(
            input.into_iter().map(Into::into).collect::<Vec<_>>(),
        )
        .await
    }

    #[tracing::instrument(skip(input))]
    pub async fn generate_content_iter(
        &self,
        input: impl IntoIterator<Item = impl Into<GeminiContent>>,
    ) -> crate::Result<Value> {
        self.generate_content_inner::<GeminiContent>(
            input.into_iter().map(Into::into).collect::<Vec<_>>(),
        )
        .await
    }
}

#[async_trait::async_trait]
impl LlmClient for GeminiClient {
    type Error = LlmError;
    type Input = Vec<GeminiContent>;
    type Output = Value;
    type StreamedOutput = Bytes;
    type Config = ();

    async fn chat_completion(&self, messages: Self::Input) -> Result<Self::Output, Self::Error> {
        self.generate_content_iter(messages).await
    }

    async fn stream_chat_completion(
        &self,
        messages: Self::Input,
    ) -> Result<impl Stream<Item = Result<Self::StreamedOutput, Self::Error>>, Self::Error> {
        self.stream_generate_content_iter(messages).await
    }
}
