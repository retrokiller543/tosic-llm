use crate::traits::LlmClient;
use derive_more::{AsMut, AsRef, Deref, DerefMut, From, IsVariant, TryUnwrap, Unwrap};
use futures_util::Stream;
use std::fmt::Debug;

#[derive(IsVariant, Unwrap, TryUnwrap, Debug)]
pub enum MaybeStream<T, ST: Stream> {
    Static(T),
    Stream(ST),
}

#[derive(Debug, Clone, AsRef, AsMut, Deref, DerefMut, From)]
pub struct LlmProvider<T> {
    inner: T,
}

impl<T: LlmClient> LlmProvider<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

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
