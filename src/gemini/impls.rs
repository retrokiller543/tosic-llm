use crate::types::{ImageMessagePart, LlmMessage, LlmMessagePart, LlmMessages};
use crate::{GeminiBlob, GeminiContent, GeminiFileData, GeminiPart, GeminiRequest};

impl From<LlmMessagePart> for GeminiPart {
    fn from(part: LlmMessagePart) -> Self {
        match part {
            LlmMessagePart::Text { text } => Self::Text { text },
            LlmMessagePart::Blob(blob) => Self::InlineData {
                inline_data: GeminiBlob {
                    mime_type: blob.mime_type,
                    data: blob.data,
                },
            },
            LlmMessagePart::Image(img) => match img {
                ImageMessagePart::Base64 { data, media_type } => Self::InlineData {
                    inline_data: GeminiBlob {
                        mime_type: media_type,
                        data,
                    },
                },
                ImageMessagePart::Url { url } => Self::FileData {
                    file_data: GeminiFileData {
                        mime_type: "image/*".to_string(),
                        data: url,
                    },
                },
            },
            LlmMessagePart::Audio { data, format } => Self::InlineData {
                inline_data: GeminiBlob {
                    mime_type: format.to_string(),
                    data,
                },
            },
        }
    }
}

impl From<LlmMessage> for GeminiContent {
    fn from(msg: LlmMessage) -> Self {
        match msg {
            LlmMessage::Text { role, text } => Self::new(Some(role), GeminiPart::Text { text }),
            LlmMessage::Detailed { role, parts } => Self::from_iter(Some(role), parts),
        }
    }
}

impl From<LlmMessages> for Vec<GeminiContent> {
    fn from(msg: LlmMessages) -> Self {
        msg.0.into_iter().map(Into::into).collect()
    }
}

impl From<LlmMessages> for GeminiRequest {
    fn from(msgs: LlmMessages) -> Self {
        Self {
            contents: msgs.0.into_iter().map(Into::into).collect(),
        }
    }
}
