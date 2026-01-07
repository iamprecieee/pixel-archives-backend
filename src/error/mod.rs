use deadpool_redis::redis;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: &'static str,
    pub id: Option<Value>,
    pub error: JsonRpcError,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid params - {0}")]
    InvalidParams(String),

    #[error("Internal server error - {0}")]
    InternalServerError(String),

    #[error("Database error - {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Canvas not found")]
    CanvasNotFound,

    #[error("Invalid canvas state transition")]
    InvalidCanvasStateTransition,

    #[error("Redis error - {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error - {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl AppError {
    pub fn code(&self) -> i32 {
        match self {
            Self::InvalidParams(_) => -32602,
            Self::InternalServerError(_) => -32603,
            Self::DatabaseError(_) => -32070,
            Self::CanvasNotFound => -32030,
            Self::InvalidCanvasStateTransition => -32031,
            Self::RedisError(_) => -32071,
            Self::SerializationError(_) => -32601,
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
            Self::DatabaseError(error) => {
                tracing::error!(error = %error, "Database error");

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
