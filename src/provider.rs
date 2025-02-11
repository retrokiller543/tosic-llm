use crate::traits::LlmClient;
use derive_more::{AsMut, AsRef, Deref, DerefMut, Display, From, IsVariant, TryUnwrap, Unwrap};
use futures_util::Stream;
use std::fmt::Debug;

/// Represents either a static value or a stream of values.
///
/// # Variants
///
/// * `Static` - Contains a single value of type `T`
/// * `Stream` - Contains a stream of values implementing the `Stream` trait, does not have to be of type `T`
#[derive(
    Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Display, IsVariant, Unwrap, TryUnwrap,
)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum MaybeStream<T, ST: Stream> {
    Static(T),
    Stream(ST),
}

/// A wrapper around an LLM client implementation that provides unified interface for
/// both streaming and non-streaming operations.
///
/// This struct implements various traits for ergonomic usage including `Debug`,
/// `Clone`, `Copy`, `Ord`, `PartialOrd`, `Eq`, `PartialEq`, `Hash` `AsRef`, `AsMut`, `Deref`, `DerefMut`, and `From`.
///
/// # Type Parameters
///
/// * `T` - The underlying LLM client type implementing the `LlmClient` trait
#[derive(
    Debug,
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Display,
    AsRef,
    AsMut,
    Deref,
    DerefMut,
    From,
)]
pub struct LlmProvider<T> {
    inner: T,
}

impl<T: LlmClient> LlmProvider<T> {
    #[inline(always)]
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Generates a response from the LLM client, either as a static value or a stream.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to send to the LLM client
    /// * `stream` - Whether to stream the response
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either:
    /// * `MaybeStream::Static` with the complete response
    /// * `MaybeStream::Stream` with a stream of partial responses
    ///
    /// # Errors
    ///
    /// Returns an error if either the streaming or non-streaming operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use derive_more::{Display, Error};
    /// # use futures_util::Stream;
    /// # use tosic_llm::{LlmProvider, traits::LlmClient};
    /// # use serde::{Serialize, Deserialize};
    /// # use tosic_llm::types::LlmMessages;
    /// #
    /// # // Example minimal LlmClient implementation
    /// # struct SimpleClient;
    /// # #[derive(Debug, Serialize)]
    /// # struct SimpleInput(String);
    ///
    /// # impl From<LlmMessages> for SimpleInput {
    /// #
    /// #     fn from(value: LlmMessages) -> Self {
    /// #         todo!()
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug, Deserialize)]
    /// # struct SimpleOutput(String);
    /// # #[derive(Debug, Error, Display)]
    /// # struct SimpleError;
    /// # struct SimpleConfig;
    /// #
    /// # #[async_trait::async_trait]
    /// # impl LlmClient for SimpleClient {
    /// #     type Error = SimpleError;
    /// #     type Input = SimpleInput;
    /// #     type Output = SimpleOutput;
    /// #     type StreamedOutput = String;
    /// #     type Config = SimpleConfig;
    /// #
    /// #     async fn chat_completion(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
    /// #         Ok(SimpleOutput("response".to_string()))
    /// #     }
    /// #
    /// #     async fn stream_chat_completion(&self, input: Self::Input)
    /// #         -> Result<impl Stream<Item = Result<Self::StreamedOutput, Self::Error>>, Self::Error> {
    /// #         Ok(futures_util::stream::empty())
    /// #     }
    /// # }
    /// #
    /// # async fn example() -> Result<(), SimpleError> {
    /// let client = SimpleClient; // Any type implementing LlmClient
    /// let provider = LlmProvider::new(client);
    ///
    /// // Create input for the LLM
    /// let input = SimpleInput("What is Rust?".to_string());
    ///
    /// // Handle static response
    /// let response = provider.generate(input, false).await?;
    /// if let Ok(output) = response.try_unwrap_static() {
    ///     println!("Got response: {:?}", output);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[tracing::instrument(skip(self, input))]
    pub async fn generate(
        &self,
        input: T::Input,
        stream: bool,
    ) -> Result<
        MaybeStream<T::Output, impl Stream<Item = Result<T::StreamedOutput, T::Error>>>,
        T::Error,
    > {
        if stream {
            Ok(MaybeStream::Stream(
                self.stream_chat_completion(input).await?,
            ))
        } else {
            Ok(MaybeStream::Static(self.chat_completion(input).await?))
        }
    }
}
