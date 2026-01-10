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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct CreateCanvasParams {
    pub access_token: String,
    pub name: String,

    #[serde(default)]
    pub initial_color: Option<i16>,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct CanvasResponse {
    pub id: String,
    pub name: String,
    pub invite_code: String,
    pub state: String,
    pub owner_id: String,
    pub canvas_pda: Option<String>,
    pub mint_address: Option<String>,
}

#[derive(Deserialize)]
pub struct GetCanvasParams {
    pub access_token: String,
    pub canvas_id: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Deserialize)]
pub struct ListCanvasParams {
    pub access_token: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct ListCanvasResponse {
    pub owned: Vec<CanvasResponse>,
    pub collaborating: Vec<CanvasResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedPixelInfo {
    pub x: i16,
    pub y: i16,
    pub owner_id: String,
    pub price_lamports: i64,
}

#[derive(Debug, Serialize)]
pub struct CanvasWithPixelsResponse {
    pub canvas: CanvasResponse,

    /// Base64-encoded 1024-byte array where byte at index (y*32 + x) is the color.
    pub pixel_colors: String,
    pub owned_pixels: Vec<OwnedPixelInfo>,
}

#[derive(Deserialize)]
pub struct JoinCanvasParams {
    pub access_token: String,
    pub invite_code: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct JoinCanvasResponse {
    pub success: bool,
    pub canvas_id: String,
}

#[derive(Deserialize)]
pub struct PublishCanvasParams {
    pub access_token: String,
    pub canvas_id: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct PublishCanvasResponse {
    pub success: bool,
    pub state: String,
    pub pixel_colors_packed: String,
}

#[derive(Deserialize)]
pub struct ConfirmPublishCanvasParams {
    pub access_token: String,
    pub canvas_id: String,
    pub signature: String,
    pub canvas_pda: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct ConfirmPublishCanvasResponse {
    pub success: bool,
    pub state: String,
    pub canvas_pda: String,
}

#[derive(Deserialize)]
pub struct CancelPublishCanvasParams {
    pub access_token: String,
    pub canvas_id: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct CancelPublishCanvasResponse {
    pub success: bool,
    pub state: String,
}

#[derive(Deserialize)]
pub struct DeleteCanvasParams {
    pub access_token: String,
    pub canvas_id: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct DeleteCanvasResponse {
    pub success: bool,
}

#[derive(Deserialize)]
pub struct PlacePixelBidParams {
    pub access_token: String,
    pub canvas_id: String,
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub bid_lamports: Option<i64>,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct PlacePixelBidResponse {
    pub success: bool,
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub requires_confirmation: bool,
    pub previous_owner_wallet: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfirmPixelBidParams {
    pub access_token: String,
    pub canvas_id: String,
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub bid_lamports: Option<i64>,
    pub signature: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct ConfirmPixelBidResponse {
    pub success: bool,
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub owner_id: Option<String>,
    pub price_lamports: i64,
}

#[derive(Deserialize)]
pub struct CancelPixelBidParams {
    pub access_token: String,
    pub canvas_id: String,
    pub x: i16,
    pub y: i16,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct CancelPixelBidResponse {
    pub success: bool,
}

#[derive(Deserialize)]
pub struct PaintPixelParams {
    pub access_token: String,
    pub canvas_id: String,
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub signature: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct PaintPixelResponse {
    pub success: bool,
    pub x: i16,
    pub y: i16,
    pub color: i16,
}
