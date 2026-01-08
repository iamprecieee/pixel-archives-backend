use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,

    #[serde(default)]
    pub params: Value,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse<T: Serialize> {
    pub jsonrpc: &'static str,
    pub result: T,
    pub id: Option<Value>,
}

impl<T: Serialize> JsonRpcResponse<T> {
    pub fn new(result: T, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            result,
            id,
        }
    }
}

pub enum AuthOperation {
    Login,
    Register,
}

#[derive(serde::Deserialize)]
pub struct AuthParams {
    pub wallet: String,
    pub message: String,
    pub signature: String,
    pub username: Option<String>,
    #[serde(skip)]
    pub state: Option<AppState>,
    #[serde(skip)]
    pub operation: Option<AuthOperation>,
}

#[derive(serde::Deserialize)]
pub struct SessionParams {
    pub access_token: String,
    pub refresh_token: Option<String>,
    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub wallet_address: String,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
}
