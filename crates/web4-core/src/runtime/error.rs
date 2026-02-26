use crate::error::{ErrorCode, RuntimeError, Web4Error};

pub fn invalid(message: impl Into<String>) -> Web4Error {
    Web4Error::Runtime(RuntimeError::new(
        ErrorCode::InvalidArgument,
        message,
        false,
    ))
}

pub fn internal(message: impl Into<String>) -> Web4Error {
    Web4Error::Runtime(RuntimeError::new(ErrorCode::InternalError, message, false))
}
