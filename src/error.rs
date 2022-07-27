use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;
use std::time::SystemTimeError;
#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum Error {
    #[error("turn: AllocateResponse has not ATTR_ERROR_CODE")]
    ErrAllocateResponseIncludeNoErrorCodeAttribute,
    #[error("turn: RequestType is UNKNOWN")]
    ErrRequestTypeUnknown,
    #[error("turn: Receiver is closed")]
    ErrReceiverClosed,
    #[error("turn: duplicated NONCE generated, discarding request")]
    ErrDuplicatedNonce,
    #[error("{0}")]
    Other(String),
}

// SystemTimeErrorなど、ライブラリのエラーなども、独自定義Errorにラッピングしてあげる
impl From<SystemTimeError> for Error {
    fn from(e: SystemTimeError) -> Self {
        Error::Other(e.to_string())
    }
}
