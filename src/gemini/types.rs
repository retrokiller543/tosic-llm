// tosic_llm/src/gemini/types.rs

use crate::types::Bytes;
use crate::types::Role;
use crate::utils::SingleOrMultiple;
use derive_more::{Display, From, FromStr};
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::openapi::{KnownFormat, RefOr, Schema, SchemaFormat};
use utoipa::{PartialSchema, ToSchema, openapi};

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
)]
pub enum GeminiRole {
    #[display("user")]
    User,
    #[display("model")]
    Model,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, From,
)]
pub struct GeminiRequest {
    pub(crate) contents: Vec<GeminiContent>,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema, From,
)]
pub struct GeminiContent {
    role: Option<Role>,
    parts: Vec<GeminiPart>,
}

impl GeminiContent {
    pub fn new(role: Option<Role>, part: impl Into<GeminiPart>) -> Self {
        Self {
            role,
            parts: vec![part.into()],
        }
    }

    pub fn from_iter(
        role: Option<Role>,
        parts: impl IntoIterator<Item = impl Into<GeminiPart>>,
    ) -> Self {
        Self {
            role,
            parts: parts.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T: Into<GeminiPart>> From<SingleOrMultiple<T>> for GeminiContent {
    fn from(value: SingleOrMultiple<T>) -> Self {
        match value {
            SingleOrMultiple::Single(item) => Self::new(None, item),
            SingleOrMultiple::Multiple(items) => Self::from_iter(None, items),
        }
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, From, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema,
)]
#[serde(rename_all = "camelCase", untagged)]
pub enum GeminiPart {
    Text {
        text: String,
    },
    InlineData {
        inline_data: GeminiBlob,
    },
    FileData {
        file_data: GeminiFileData,
    },
    ExecutableCode {
        executable_code: ExecutableCode,
    },
    CodeExecutionResult {
        code_execution_result: CodeExecutionResult,
    },
}

#[derive(
    Serialize, Deserialize, Debug, Clone, From, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema,
)]
pub struct GeminiBlob {
    #[serde(rename = "mimeType")]
    pub(crate) mime_type: String,
    /// A padded, base64-encoded string of bytes, encoded with a URL and filename safe alphabet (sometimes referred to as "web-safe" or "base64url"). Defined by [RFC4648](https://datatracker.ietf.org/doc/html/rfc4648).
    pub(crate) data: String,
}

impl PartialSchema for Bytes {
    fn schema() -> RefOr<Schema> {
        openapi::ObjectBuilder::new()
            .schema_type(openapi::Type::Array)
            .format(Some(SchemaFormat::KnownFormat(KnownFormat::Byte)))
            .into()
    }
}

impl ToSchema for Bytes {}

#[derive(
    Serialize, Deserialize, Debug, Clone, From, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema,
)]
pub struct GeminiFileData {
    #[serde(rename = "mimeType")]
    pub(crate) mime_type: String,
    pub(crate) data: Url,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, From, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema,
)]
pub struct ExecutableCode {
    language: GeminiCodeLanguage,
    code: String,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, From, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema,
)]
pub struct CodeExecutionResult {
    outcome: GeminiCodeOutcome,
    output: String,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, FromStr, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiCodeOutcome {
    OutcomeUnspecified,
    OutcomeOk,
    OutcomeFailed,
    OutcomeDeadlineExceeded,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, FromStr, Hash, Eq, PartialEq, Ord, PartialOrd, ToSchema,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum GeminiCodeLanguage {
    Python,
}
