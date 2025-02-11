use async_trait::async_trait;
use futures_util::Stream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::types::LlmMessages;

#[async_trait]
pub trait LlmClient: Send + Sync {
    type Error: std::error::Error + Send;
    type Input: Serialize + From<LlmMessages>;
    type Output: DeserializeOwned;
    type StreamedOutput: DeserializeOwned;
    type Config;

    async fn chat_completion(
        &self,
        messages: Self::Input,
    ) -> Result<Self::Output, Self::Error>;

    async fn stream_chat_completion(
        &self,
        messages: Self::Input,
    ) -> Result<impl Stream<Item = Result<Self::StreamedOutput, Self::Error>>, Self::Error>;
}

