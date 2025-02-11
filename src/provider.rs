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
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
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
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Display, AsRef, AsMut, Deref, DerefMut, From)]
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
    /// ```no_run
    /// # use tosic_llm::{GeminiClient, GeminiModel};
    /// # use tosic_llm::provider::LlmProvider;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = GeminiClient::new(GeminiModel::Gemini2FlashLite)?; // could be any type that implements `LlmClient`
    /// let provider = LlmProvider::new(client);
    ///
    /// let input = Vec::new(); // Replace with the actual input type of your client
    /// 
    /// // Handle static response
    /// match provider.generate(input, false).await {
    ///     Ok(maybe_stream) => {
    ///         if let Ok(response) = maybe_stream.try_unwrap_static() { // a ref or a ref mut could also be unwrapped if ownership is not desired
    ///             println!("Got response: {:?}", response);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Error: {}", e),
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
