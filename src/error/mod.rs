use thiserror::Error;

use crate::error::types::JsonRpcError;

pub mod types;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid params - {0}")]
    InvalidParams(String),

    #[error("Internal server error - {0}")]
    InternalServerError(String),
}

impl AppError {
    pub fn code(&self) -> i32 {
        match self {
            Self::InvalidParams(_) => -32602,
            Self::InternalServerError(_) => -32603,
        }
    }

    pub fn user_safe_format(&self) -> JsonRpcError {
        match self {
            Self::InternalServerError(error) => {
                tracing::error!(error = %error, "Internal server error");

                JsonRpcError {
                    code: self.code(),
                    message: "Internal server error".to_string(),
                    data: None,
                }
            }
            _ => JsonRpcError {
                code: self.code(),
                message: self.to_string(),
                data: None,
            },
        }
    }
}

impl From<&AppError> for JsonRpcError {
    fn from(error: &AppError) -> Self {
        error.user_safe_format()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
