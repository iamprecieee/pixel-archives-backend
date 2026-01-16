use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

impl SuccessResponse {
    pub fn ok() -> Self {
        Self { success: true }
    }
}

#[derive(Debug, Serialize)]
pub struct StateChangeResponse {
    pub success: bool,
    pub state: String,
}

impl StateChangeResponse {
    pub fn new(state: impl Into<String>) -> Self {
        Self {
            success: true,
            state: state.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelCoords {
    pub x: i16,
    pub y: i16,
    pub color: i16,
}

#[derive(Deserialize)]
pub struct CanvasActionParams {
    pub access_token: String,
    pub canvas_id: Uuid,

    #[serde(skip)]
    pub state: Option<AppState>,
}
