use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
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

impl JsonRpcErrorResponse {
    pub fn new(error: JsonRpcError, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            error,
        }
    }

    pub fn from_error(err: &AppError, id: Option<Value>) -> Value {
        serde_json::to_value(Self::new(err.user_safe_format(), id))
            .expect("JsonRpcErrorResponse serialization failed")
    }
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

    #[error("IO error - {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Canvas name already exists")]
    CanvasNameExists,

    #[error("User already exists")]
    UserExists,

    #[error("Username already exists")]
    UsernameExists,

    #[error("User not found")]
    UserNotFound,

    #[error("Method not found - {0}")]
    MethodNotFound(String),

    #[error("Not a collaborator on this canvas")]
    NotCanvasCollaborator,

    #[error("Not canvas owner")]
    NotCanvasOwner,

    #[error("Pixel locked")]
    PixelLocked,

    #[error("Solana RPC error - {0}")]
    SolanaRpc(String),

    #[error("Transaction failed - {0}")]
    TransactionFailed(String),

    #[error("Cooldown active - {remaining_ms}ms remaining")]
    CooldownActive { remaining_ms: u64 },

    #[error("Bid too low - minimum is {min_lamports} lamports")]
    BidTooLow { min_lamports: u64 },

    #[error("TryInitError - {0}")]
    TryInitError(#[from] tracing_subscriber::util::TryInitError),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
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
            Self::SerializationError(_) => -32072,
            Self::Unauthorized => -32020,
            Self::TokenExpired => -32021,
            Self::InvalidSignature => -32012,
            Self::CanvasNameExists => -32037,
            Self::IoError(_) => -32080,
            Self::UserExists => -32010,
            Self::UsernameExists => -32013,
            Self::UserNotFound => -32011,
            Self::MethodNotFound(_) => -32601,
            Self::NotCanvasCollaborator => -32035,
            Self::NotCanvasOwner => -32034,
            Self::PixelLocked => -32040,
            Self::SolanaRpc(_) => -32061,
            Self::TransactionFailed(_) => -32060,

            Self::CooldownActive { .. } => -32042,
            Self::BidTooLow { .. } => -32041,
            Self::TryInitError(_) => -32080,
            Self::RateLimitExceeded => -32081,
        }
    }

    pub fn user_safe_format(&self) -> JsonRpcError {
        match self {
            Self::InvalidParams(msg) => JsonRpcError {
                code: self.code(),
                message: msg.clone(),
                data: None,
            },
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
            Self::TryInitError(error) => {
                tracing::error!(error = %error, "TryInitError");

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
            Self::SolanaRpc(error) => {
                tracing::error!(error = %error, "Solana RPC error");

                JsonRpcError {
                    code: self.code(),
                    message: "Network error connecting to Solana. Please try again.".to_string(),
                    data: None,
                }
            }
            Self::TransactionFailed(error) => {
                tracing::error!(error = %error, "Transaction failed");

                JsonRpcError {
                    code: self.code(),
                    message: "Transaction failed. Please try again.".to_string(),
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
            Self::InvalidCanvasStateTransition => JsonRpcError {
                code: self.code(),
                message: "Invalid canvas state transition".to_string(),
                data: None,
            },
            Self::IoError(error) => {
                tracing::error!(error = %error, "IO error");

                JsonRpcError {
                    code: self.code(),
                    message: "Service temporarily unavailable. Please try again later.".to_string(),
                    data: None,
                }
            }
            Self::UserExists => JsonRpcError {
                code: self.code(),
                message: "An account with this wallet already exists. Please login instead."
                    .to_string(),
                data: None,
            },
            Self::UsernameExists => JsonRpcError {
                code: self.code(),
                message: "This username is already taken. Please choose a different one."
                    .to_string(),
                data: None,
            },
            Self::UserNotFound => JsonRpcError {
                code: self.code(),
                message: "No account found for this wallet. Please register first.".to_string(),
                data: None,
            },
            Self::MethodNotFound(method) => JsonRpcError {
                code: self.code(),
                message: format!("Method '{}' not found", method),
                data: None,
            },
            Self::CooldownActive { remaining_ms } => JsonRpcError {
                code: self.code(),
                message: format!(
                    "Please wait {}s before placing another pixel.",
                    remaining_ms / 1000
                ),
                data: Some(serde_json::json!({ "remaining_ms": remaining_ms })),
            },
            Self::BidTooLow { min_lamports } => JsonRpcError {
                code: self.code(),
                message: format!(
                    "Bid too low. Minimum is {} SOL.",
                    (*min_lamports as f64) / 1_000_000_000.0
                ),
                data: Some(serde_json::json!({ "min_lamports": min_lamports })),
            },
            Self::RateLimitExceeded => JsonRpcError {
                code: self.code(),
                message: "Too many requests. Try again in a moment.".to_string(),
                data: None,
            },
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Unauthorized | Self::TokenExpired | Self::InvalidSignature => {
                StatusCode::UNAUTHORIZED
            }
            Self::UserNotFound | Self::CanvasNotFound => StatusCode::NOT_FOUND,
            Self::UserExists | Self::UsernameExists | Self::CanvasNameExists => {
                StatusCode::CONFLICT
            }
            Self::InvalidParams(_) | Self::InvalidCanvasStateTransition => StatusCode::BAD_REQUEST,
            Self::NotCanvasCollaborator | Self::NotCanvasOwner => StatusCode::FORBIDDEN,
            Self::CooldownActive { .. } | Self::BidTooLow { .. } | Self::PixelLocked => {
                StatusCode::TOO_MANY_REQUESTS
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let json_error = self.user_safe_format();
        let body = serde_json::to_string(&json_error)
            .unwrap_or_else(|_| r#"{"code":-32603,"message":"Internal server error"}"#.to_string());

        (status, [("content-type", "application/json")], body).into_response()
    }
}
