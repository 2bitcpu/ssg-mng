use common::types::BoxError;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum AppError {
    Unauthorized(),
    AccountLocked(),
    Forbidden(),
    ServerBusy(),
    BadRequest(String),
    DataNotFound(String),
    DataConflict(String),
    Inconsistent(String),
    Unexpected(BoxError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Unauthorized() => write!(f, "Unauthorized"),
            AppError::AccountLocked() => write!(f, "Account locked"),
            AppError::Forbidden() => write!(f, "Forbidden"),
            AppError::ServerBusy() => write!(f, "Server busy"),
            AppError::BadRequest(reason) => write!(f, "Bad request: {}", reason),
            AppError::DataNotFound(reason) => write!(f, "Data not found: {}", reason),
            AppError::DataConflict(reason) => write!(f, "Data conflict: {}", reason),
            AppError::Inconsistent(reason) => write!(f, "Inconsistent: {}", reason),
            AppError::Unexpected(e) => {
                write!(f, "An unexpected infrastructure error occurred: {}", e)
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Unexpected(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<BoxError> for AppError {
    fn from(e: BoxError) -> Self {
        AppError::Unexpected(e)
    }
}
