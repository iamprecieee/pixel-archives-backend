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

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Canvas name already exists")]
    CanvasNameExists,
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
            Self::Unauthorized => -32020,
            Self::TokenExpired => -32021,
            Self::InvalidSignature => -32012,
            Self::CanvasNameExists => -32037,
        }
    }

    pub fn user_safe_format(&self) -> JsonRpcError {
        match self {
            Self::RedisError(error) => {
                tracing::error!(error = %error, "Redis error");

                JsonRpcError {
                    code: self.code(),
                    message: "Service temporarily unavailable. Please try again later.".to_string(),
                    data: None,
                }
            }
            Self::SerializationError(error) => {
                tracing::error!(error = %error, "Serialization error");

                JsonRpcError {
                    code: self.code(),
                    message: "Internal server error".to_string(),
                    data: None,
                }
            }
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

                let error_string = error.to_string();

                if error_string.contains("canvases_name_key") {
                    return JsonRpcError {
                        code: AppError::CanvasNameExists.code(),
                        message: "A canvas with this name already exists. Please choose a different name.".to_string(),
                        data: None,
                    };
                }

                JsonRpcError {
                    code: self.code(),
                    message: "Service temporarily unavailable. Please try again later.".to_string(),
                    data: None,
                }
            }
            Self::Unauthorized => JsonRpcError {
                code: self.code(),
                message: "Please login to continue.".to_string(),
                data: None,
            },
            Self::TokenExpired => JsonRpcError {
                code: self.code(),
                message: "Session has expired. Please login again.".to_string(),
                data: None,
            },
            Self::InvalidSignature => JsonRpcError {
                code: self.code(),
                message: "Wallet signature verification failed. Please try signing again."
                    .to_string(),
                data: None,
            },
            Self::CanvasNotFound => JsonRpcError {
                code: self.code(),
                message: "Canvas not found".to_string(),
                data: None,
            },
            Self::InvalidCanvasStateTransition => JsonRpcError {
                code: self.code(),
                message: "Invalid canvas state transition".to_string(),
                data: None,
            },
            Self::CanvasNameExists => {
                tracing::error!("Canvas name already exists");

                JsonRpcError {
                    code: self.code(),
                    message: "Canvas name already exists".to_string(),
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
