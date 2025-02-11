#![cfg(any(doc, doctest, test, feature = "doc-utils"))]
#![doc(hidden)]

#[doc(hidden)]
#[macro_export]
macro_rules! mocked_llm_client {
    () => {
        struct SimpleClient;
        #[derive(Debug, ::serde::Serialize)]
        struct SimpleInput(String);
        
        impl From<$crate::types::LlmMessages> for SimpleInput {
            fn from(value: $crate::types::LlmMessages) -> Self {
                todo!()
            }
        }
        
        #[derive(Debug, ::serde::Deserialize)]
        struct SimpleOutput(String);
        #[derive(Debug, ::derive_more::Error, ::derive_more::Display)]
        struct SimpleError;
        struct SimpleConfig;
        
        #[::async_trait::async_trait]
        impl $crate::traits::LlmClient for SimpleClient {
            type Error = SimpleError;
            type Input = SimpleInput;
            type Output = SimpleOutput;
            type StreamedOutput = String;
            type Config = SimpleConfig;
            
            async fn chat_completion(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
                Ok(SimpleOutput("response".to_string()))
            }
            
            async fn stream_chat_completion(&self, input: Self::Input)
            -> Result<impl ::futures_util::Stream<Item = Result<Self::StreamedOutput, Self::Error>>, Self::Error> {
                Ok(::futures_util::stream::empty())
            }
        }
    };
}