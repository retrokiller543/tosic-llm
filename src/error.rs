// tosic_llm/src/error.rs

use std::fmt::{Debug, Display, Formatter};
use derive_more::IsVariant;
use thiserror::Error;
use url::ParseError;

#[derive(Debug, Error, IsVariant)]
pub enum LlmErrorType {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("An error occurred: {0}")]
    Generic(#[from] Box<dyn std::error::Error + Send>),
    #[error("{message}")]
    Context {
        message: String,
        #[source]
        source: Box<LlmErrorType>,
    },
    #[error("Value was empty. {value}, expected one of: {expected:?}")]
    Empty {
        value: String,
        expected: &'static [&'static str],
    },
}

#[derive(Debug)]
pub struct ErrorContext {
    pub message: String,
    pub file: &'static str,
    pub line: u32,
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at {}:{})", self.message, self.file, self.line)
    }
}

pub struct LlmError {
    pub error_type: LlmErrorType,
}

impl Debug for LlmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.error_type {
            LlmErrorType::Context { .. } => write!(f, "{}", self), // Avoid using debug print for context to avoid hard to read message due to recursive type.
            _ => write!(f, "{:?}", self.error_type),
        }
    }
}

impl LlmError {
    pub fn new(error_type: LlmErrorType) -> Self {
        Self { error_type }
    }
    
    pub fn add_context(self, message: impl Display) -> Self {
        let source = Box::new(self.error_type);
        
        Self::new(LlmErrorType::Context { message: message.to_string(), source })
    }
    
    pub fn new_context(message: impl Display, source: Box<LlmErrorType>) -> Self {
        let source = source;
        
        Self::new(LlmErrorType::Context {
            message: message.to_string(),
            source
        })
    }
}

impl<E: Into<LlmErrorType>> From<E> for LlmError {
    fn from(value: E) -> Self {
        Self {
            error_type: value.into(),
        }
    }
}

impl LlmErrorType {
    pub(crate) fn fmt_context(&self, writer: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmErrorType::Context { message, source } => {
                writeln!(writer, "\t- {message}")?;
                source.fmt_context(writer)
            }
            _ => Ok(())
        }
    }
}

impl Display for LlmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", self.error_type)?;
        match &self.error_type {
            LlmErrorType::Context { message, source } => {
                writeln!(f, "\nContext:")?;
                writeln!(f, "\t- {}", message)?;
                source.fmt_context(f)
            }
            _ => Ok(())
        }
    }
}

impl std::error::Error for LlmError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.error_type {
            LlmErrorType::Context { source, .. } => Some(source),
            _ => {
                Some(&self.error_type)
            }
        }
    }
}

pub trait WithContext<T> {
    fn context<C: Display>(self, context: C) -> Result<T, LlmError>;
    fn with_context<C, F>(self, context_fn: F) -> Result<T, LlmError>
    where
        F: FnOnce() -> C,
        C: Display;
}

impl<T> WithContext<T> for Result<T, LlmError> {
    fn context<C: Display>(self, context: C) -> Result<T, LlmError> {
        self.map_err(|error| {
            error.add_context(context)
        })
    }

    fn with_context<C, F>(self, context_fn: F) -> Result<T, LlmError>
    where
        F: FnOnce() -> C,
        C: Display,
    {
        self.map_err(|error| {
            error.add_context(context_fn())
        })
    }
}

impl<T, E> WithContext<T> for Result<T, E>
where
    E: Into<LlmErrorType>,
{
    fn context<C: Display>(self, context: C) -> Result<T, LlmError> {
        self.map_err(|error| {
            let llm_error = error.into();
            
            LlmError::new_context(context, Box::new(llm_error))
        })
    }

    fn with_context<C, F>(self, context_fn: F) -> Result<T, LlmError>
    where
        F: FnOnce() -> C,
        C: Display,
    {
        self.map_err(|error| {
            let llm_error = error.into();
            
            LlmError::new_context(context_fn(), Box::new(llm_error))
        })
    }
}

impl<T: Debug> WithContext<T> for Option<T> {
    fn context<C: Display>(self, context: C) -> Result<T, LlmError> {
        match self {
            Some(val) => Ok(val),
            None => {
                let llm_error = LlmErrorType::Empty {
                    value: format!("{self:?}"),
                    expected: &[],
                };
                
                Err(LlmError::new_context(context, Box::new(llm_error)))
            }
        }
    }

    fn with_context<C, F>(self, context_fn: F) -> Result<T, LlmError>
    where
        F: FnOnce() -> C,
        C: Display
    {
        match self {
            Some(val) => Ok(val),
            None => {
                let llm_error = LlmErrorType::Empty {
                    value: format!("{self:?}"),
                    expected: &[],
                };
                
                Err(LlmError::new_context(context_fn(), Box::new(llm_error)))
            }
        }
    }
}

#[macro_export]
macro_rules! error_context {
    ($msg:expr) => {
        $crate::error::ErrorContext {
            message: $msg.to_string(),
            file: file!(),
            line: line!(),
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::ErrorContext {
            message: format!($fmt, $($arg)*),
            file: file!(),
            line: line!(),
        }
    };
}

#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    }
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $expr:expr) => {
        if !$cond {
            return Err($expr.into());
        }
    }
}