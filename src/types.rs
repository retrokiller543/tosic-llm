// tosic_llm/src/types.rs

use derive_more::{AsMut, AsRef, Deref, DerefMut, Display, From, Into, IsVariant, TryUnwrap, Unwrap};
use serde::{Deserialize, Serialize};
use tosic_utils::wrap_external_type;
use url::Url;
use utoipa::ToSchema;

wrap_external_type! {
    #[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, AsRef, AsMut)]
    pub(crate) struct Bytes(bytes::Bytes);
}

#[derive(
    Serialize, Deserialize, Debug, Clone, From, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, Into
)]
pub struct Blob {
    pub(crate) mime_type: String,
    /// A padded, base64-encoded string of bytes, encoded with a URL and filename safe alphabet (sometimes referred to as "web-safe" or "base64url"). Defined by [RFC4648](https://datatracker.ietf.org/doc/html/rfc4648).
    pub(crate) data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, IsVariant)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LlmMessagePart {
    Text {
        text: String,
    },
    Image(ImageMessagePart),
    Audio {
        data: String,
        format: MediaFormat
    },
    Blob(Blob),
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, IsVariant)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ImageMessagePart {
    Base64 {
        data: String,
        media_type: String, // Maybe restrict this?
    },
    Url {
        url: Url,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, IsVariant,
    Unwrap,
    TryUnwrap, Display)]
#[serde(rename_all = "lowercase")]
pub enum MediaFormat {
    #[display("audio/wav")]
    Wav,
    #[display("audio/mpeg")]
    Mp3
}

#[derive(
    Debug,
    Display,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    ToSchema,
    IsVariant,
    Unwrap,
    TryUnwrap
)]
pub enum Role {
    #[display("user")]
    User,
    #[display("model")]
    Model,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, IsVariant)]
#[serde(untagged)]
pub enum LlmMessage {
    Text {
        role: Role,
        text: String,
    },
    Detailed {
        role: Role,
        parts: Vec<LlmMessagePart>,
    }
}

#[derive(AsRef, AsMut, Deref, DerefMut, Serialize, Deserialize, Default, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, Into, From)]
pub struct LlmMessages(pub Vec<LlmMessage>);

// TODO: Create general configurations that should be exposed at the endpoint level 