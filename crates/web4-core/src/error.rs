use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidArgument,
    Unauthorized,
    Forbidden,
    NotFound,
    ConsentRequired,
    RateLimited,
    EffectsBlocked,
    InternalError,
    #[serde(untagged)]
    Extension(String),
}

impl ErrorCode {
    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Self::InvalidArgument => Cow::Borrowed("INVALID_ARGUMENT"),
            Self::Unauthorized => Cow::Borrowed("UNAUTHORIZED"),
            Self::Forbidden => Cow::Borrowed("FORBIDDEN"),
            Self::NotFound => Cow::Borrowed("NOT_FOUND"),
            Self::ConsentRequired => Cow::Borrowed("CONSENT_REQUIRED"),
            Self::RateLimited => Cow::Borrowed("RATE_LIMITED"),
            Self::EffectsBlocked => Cow::Borrowed("EFFECTS_BLOCKED"),
            Self::InternalError => Cow::Borrowed("INTERNAL_ERROR"),
            Self::Extension(code) => Cow::Borrowed(code.as_str()),
        }
    }

    pub fn http_status(&self) -> StatusCode {
        match self {
            Self::InvalidArgument => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::ConsentRequired => StatusCode::FORBIDDEN,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::EffectsBlocked => StatusCode::FORBIDDEN,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Extension(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl RuntimeError {
    pub fn new(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.as_str().into_owned(),
            message: message.into(),
            retryable,
            details: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn status_code(&self) -> StatusCode {
        ErrorCode::from(self.code.as_str()).http_status()
    }
}

impl From<&str> for ErrorCode {
    fn from(value: &str) -> Self {
        match value {
            "INVALID_ARGUMENT" => Self::InvalidArgument,
            "UNAUTHORIZED" => Self::Unauthorized,
            "FORBIDDEN" => Self::Forbidden,
            "NOT_FOUND" => Self::NotFound,
            "CONSENT_REQUIRED" => Self::ConsentRequired,
            "RATE_LIMITED" => Self::RateLimited,
            "EFFECTS_BLOCKED" => Self::EffectsBlocked,
            "INTERNAL_ERROR" => Self::InternalError,
            other => Self::Extension(other.to_string()),
        }
    }
}

impl From<String> for ErrorCode {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub error: RuntimeError,
}

#[derive(Debug, Error)]
pub enum Web4Error {
    #[error("{0:?}")]
    Runtime(RuntimeError),
    #[error("internal error: {0}")]
    Internal(String),
}

impl Web4Error {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Runtime(runtime) => ErrorCode::from(runtime.code.as_str()),
            Self::Internal(_) => ErrorCode::InternalError,
        }
    }

    pub fn into_runtime(self) -> RuntimeError {
        match self {
            Self::Runtime(runtime) => runtime,
            Self::Internal(message) => RuntimeError::new(ErrorCode::InternalError, message, false),
        }
    }

    pub fn into_envelope(self) -> ErrorEnvelope {
        ErrorEnvelope {
            error: self.into_runtime(),
        }
    }
}
