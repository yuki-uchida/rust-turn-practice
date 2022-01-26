use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("turn: AllocateResponse has not ATTR_ERROR_CODE")]
    ErrAllocateResponseIncludeNoErrorCodeAttribute,
    #[allow(non_camel_case_types)]
    #[error("{0}")]
    new(String),
}

impl Error {}
