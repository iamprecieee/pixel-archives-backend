use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

use super::common::{CanvasActionParams, StateChangeResponse, SuccessResponse};

// Type aliases for canvas action params
pub type GetCanvasParams = CanvasActionParams;
pub type PublishCanvasParams = CanvasActionParams;
pub type CancelPublishCanvasParams = CanvasActionParams;
pub type DeleteCanvasParams = CanvasActionParams;

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

#[derive(Debug, Serialize)]
pub struct PublishCanvasResponse {
    pub success: bool,
    pub state: String,
    pub pixel_colors_packed: String,
}

#[derive(Deserialize)]
pub struct ConfirmPublishCanvasParams {
    pub access_token: String,
    pub canvas_id: Uuid,
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

pub type CancelPublishCanvasResponse = StateChangeResponse;
pub type DeleteCanvasResponse = SuccessResponse;
