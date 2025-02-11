use std::fmt::Debug;
use derive_more::{AsMut, AsRef, Deref, DerefMut, From, IsVariant, TryUnwrap, Unwrap};
use futures_util::Stream;
use futures_util::stream::LocalBoxStream;
use crate::traits::LlmClient;

#[derive(IsVariant, Unwrap, TryUnwrap)]
pub enum MaybeStream<T, ST> {
    Static(T),
    Stream(LocalBoxStream<'static, ST>)
}

#[derive(IsVariant, Unwrap, TryUnwrap)]
pub enum MaybeStream2<T, ST: Stream> {
    Static(T),
    Stream(ST)
}

impl<T: Debug, ST> Debug for MaybeStream<T, ST> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            MaybeStream::Static(val) => f.debug_tuple("MaybeStream::Static").field(val).finish(),
            MaybeStream::Stream(_) => f.debug_tuple("MaybeStream::Stream").finish()
        }
    }
}

#[derive(Debug, AsRef, AsMut, Deref, DerefMut, From)]
pub struct LlmProvider<T> {
    inner: T,
}

impl<T: Clone> Clone for LlmProvider<T> {
    fn clone(&self) -> Self {
        LlmProvider { inner: self.inner.clone() }
    }
}

impl<T: LlmClient> LlmProvider<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub async fn generate(&'static self, input: T::Input, stream: bool) -> Result<MaybeStream<T::Output, Result<T::StreamedOutput, T::Error>>, T::Error> {
        if stream {
            Ok(MaybeStream::Stream(Box::pin(self.stream_chat_completion(input).await?)))
        } else {
            Ok(MaybeStream::Static(self.chat_completion(input).await?))
        }
    }

    pub async fn generate2(&'static self, input: T::Input, stream: bool) -> Result<MaybeStream2<T::Output, impl Stream<Item = Result<T::StreamedOutput, T::Error>>>, T::Error> {
        if stream {
            Ok(MaybeStream2::Stream(self.stream_chat_completion(input).await?))
        } else {
            Ok(MaybeStream2::Static(self.chat_completion(input).await?))
        }
    }
}

